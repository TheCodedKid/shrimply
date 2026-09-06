use crate::{action, stack};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::NSControlTextEditingDelegate;
use objc2_app_kit::{
    NSButton, NSControlStateValueOn, NSCursor, NSEvent, NSStackView, NSTextField,
    NSTextFieldDelegate, NSView,
};
use objc2_core_graphics::{CGAssociateMouseAndMouseCursorPosition, CGError};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObjectProtocol, NSPoint, NSRect, NSString,
};
use shrimply_component_core::number::{
    DEFAULT_MAXIMUM, DEFAULT_MINIMUM, NumberConfig, NumberDrag, accepted_value, format_value,
    locked_pair, locked_triple, pair_ratio, parse_fraction, positive_fraction_or, triple_ratios,
};
use shrimply_math_core::{
    FRACTION_ZERO, Fraction, fraction_as_f64, fraction_from_f64, fraction_from_integer,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type NumberCallback = Rc<dyn Fn(Fraction)>;
type PairCallback = Box<dyn Fn([f64; 2], usize)>;
type ScalarCallback = Box<dyn Fn(f64)>;

struct NumberPickerIvars {
    config: NumberConfig,
    value: Cell<Fraction>,
    display: Retained<NSTextField>,
    entry: Retained<NSTextField>,
    prefix: String,
    suffix: String,
    on_change: NumberCallback,
    on_commit: NumberCallback,
    drag: Cell<NumberDrag>,
    pointer_locked: Cell<bool>,
    editing: Cell<bool>,
}

impl Drop for NumberPickerIvars {
    fn drop(&mut self) {
        if self.pointer_locked.replace(false) {
            let result = CGAssociateMouseAndMouseCursorPosition(true);
            assert_eq!(
                result,
                CGError::Success,
                "could not release the number picker pointer"
            );
            NSCursor::unhide();
        }
    }
}

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = NumberPickerIvars]
    struct NumberPickerView;

    unsafe impl NSObjectProtocol for NumberPickerView {}
    unsafe impl NSControlTextEditingDelegate for NumberPickerView {}
    unsafe impl NSTextFieldDelegate for NumberPickerView {}

    impl NumberPickerView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool { true }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool { true }

        #[unsafe(method(hitTest:))]
        fn hit_test(&self, point: NSPoint) -> Option<&NSView> {
            let hit: Option<&NSView> = unsafe { msg_send![super(self), hitTest: point] };
            if hit.is_none() || self.ivars().editing.get() {
                return hit;
            }
            Some(self.as_super())
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, _event: &NSEvent) {
            if self.ivars().editing.get() { return; }
            self.ivars().drag.set(NumberDrag::begin(self.ivars().value.get()));
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            if self.ivars().editing.get() { return; }
            let mut drag = self.ivars().drag.get();
            if !drag.update_relative(event.deltaX()) { return; }
            self.ivars().drag.set(drag);
            self.lock_pointer();
            self.set_value(drag.value(&self.ivars().config));
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, _event: &NSEvent) {
            if self.ivars().editing.get() { return; }
            if self.ivars().drag.get().moved() {
                self.release_pointer();
                (self.ivars().on_commit)(self.ivars().value.get());
            } else {
                self.begin_edit();
            }
        }

        #[unsafe(method(commitText:))]
        fn commit_text(&self, _sender: &NSTextField) { self.commit_edit(); }

        #[unsafe(method(controlTextDidEndEditing:))]
        fn text_did_end(&self, _notification: &NSNotification) { self.commit_edit(); }

        #[unsafe(method(cancelOperation:))]
        fn cancel_operation(&self, _sender: &objc2_foundation::NSObject) {
            self.release_pointer();
            self.end_edit_without_change();
        }

        #[unsafe(method(viewDidMoveToWindow))]
        fn moved_to_window(&self) {
            unsafe { let _: () = msg_send![super(self), viewDidMoveToWindow]; }
            if self.window().is_none() { self.release_pointer(); }
        }
    }
);

impl NumberPickerView {
    fn text(&self) -> String {
        format!(
            "{}{}{}",
            self.ivars().prefix,
            format_value(&self.ivars().config, self.ivars().value.get()),
            self.ivars().suffix,
        )
    }

    fn refresh(&self) {
        self.ivars()
            .display
            .setStringValue(&NSString::from_str(&self.text()));
    }

    fn set_value(&self, value: Fraction) -> bool {
        let value = accepted_value(&self.ivars().config, value);
        if self.ivars().value.replace(value) == value {
            return false;
        }
        self.refresh();
        (self.ivars().on_change)(value);
        true
    }

    fn lock_pointer(&self) {
        if self.ivars().pointer_locked.replace(true) {
            return;
        }
        let result = CGAssociateMouseAndMouseCursorPosition(false);
        assert_eq!(
            result,
            CGError::Success,
            "could not capture the number picker pointer"
        );
        NSCursor::hide();
    }

    fn release_pointer(&self) {
        if !self.ivars().pointer_locked.replace(false) {
            return;
        }
        let result = CGAssociateMouseAndMouseCursorPosition(true);
        assert_eq!(
            result,
            CGError::Success,
            "could not release the number picker pointer"
        );
        NSCursor::unhide();
    }

    fn begin_edit(&self) {
        self.ivars().editing.set(true);
        self.ivars()
            .entry
            .setStringValue(&NSString::from_str(&format_value(
                &self.ivars().config,
                self.ivars().value.get(),
            )));
        self.ivars().display.setHidden(true);
        self.ivars().entry.setHidden(false);
        self.window()
            .expect("number picker must be attached before editing")
            .makeFirstResponder(Some(&self.ivars().entry));
        unsafe {
            self.ivars().entry.selectText(None);
        }
    }

    fn commit_edit(&self) {
        if !self.ivars().editing.get() {
            return;
        }
        let changed = parse_fraction(self.ivars().entry.stringValue().to_string().trim())
            .is_some_and(|value| self.set_value(value));
        if changed {
            (self.ivars().on_commit)(self.ivars().value.get());
        }
        self.end_edit_without_change();
    }

    fn end_edit_without_change(&self) {
        if !self.ivars().editing.replace(false) {
            return;
        }
        self.ivars().entry.setHidden(true);
        self.ivars().display.setHidden(false);
        self.refresh();
    }
}

pub struct NumberPicker;

impl NumberPicker {
    pub fn builder(value: f64) -> NumberPickerBuilder {
        Self::fraction_builder(fraction_from_f64(value))
    }

    pub fn fraction_builder(value: Fraction) -> NumberPickerBuilder {
        NumberPickerBuilder {
            value,
            minimum: fraction_from_integer(DEFAULT_MINIMUM),
            maximum: fraction_from_integer(DEFAULT_MAXIMUM),
            drag_step: fraction_from_integer(1),
            digits: 2,
            prefix: String::new(),
            suffix: String::new(),
            on_change: None,
            on_commit: None,
        }
    }
}

pub struct NumberPickerBuilder {
    value: Fraction,
    minimum: Fraction,
    maximum: Fraction,
    drag_step: Fraction,
    digits: usize,
    prefix: String,
    suffix: String,
    on_change: Option<Box<dyn Fn(Fraction)>>,
    on_commit: Option<Box<dyn Fn(Fraction)>>,
}

impl NumberPickerBuilder {
    pub fn accepted_range(mut self, minimum: f64, maximum: f64) -> Self {
        self.minimum = fraction_from_f64(minimum);
        self.maximum = fraction_from_f64(maximum);
        self
    }

    pub fn minimum(mut self, value: f64) -> Self {
        self.minimum = fraction_from_f64(value);
        self
    }
    pub fn maximum(mut self, value: f64) -> Self {
        self.maximum = fraction_from_f64(value);
        self
    }
    pub fn drag_step(mut self, value: f64) -> Self {
        self.drag_step = positive_fraction_or(fraction_from_f64(value), self.drag_step);
        self
    }
    pub fn digits(mut self, value: usize) -> Self {
        self.digits = value;
        self
    }
    pub fn prefix(mut self, value: impl Into<String>) -> Self {
        self.prefix = format!("{} ", value.into());
        self
    }
    pub fn unit_name(mut self, value: impl Into<String>) -> Self {
        self.suffix = format!(" {}", value.into());
        self
    }
    pub fn on_change(mut self, callback: impl Fn(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(move |value| callback(fraction_as_f64(value))));
        self
    }
    pub fn on_change_fraction(mut self, callback: impl Fn(Fraction) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    pub fn on_commit(mut self, callback: impl Fn(f64) + 'static) -> Self {
        self.on_commit = Some(Box::new(move |value| callback(fraction_as_f64(value))));
        self
    }
    pub fn on_commit_fraction(mut self, callback: impl Fn(Fraction) + 'static) -> Self {
        self.on_commit = Some(Box::new(callback));
        self
    }

    pub fn build(self, mtm: MainThreadMarker) -> Retained<NSView> {
        self.build_with_handle(mtm).widget
    }

    pub fn build_with_handle(self, mtm: MainThreadMarker) -> NumberPickerParts {
        let config = NumberConfig {
            minimum: self.minimum,
            maximum: self.maximum,
            drag_step: self.drag_step,
            drag_pixels: shrimply_component_core::number::DEFAULT_DRAG_PIXELS,
            digits: self.digits,
            fallback: if self.value == FRACTION_ZERO {
                FRACTION_ZERO
            } else {
                self.value
            },
        };
        let value = accepted_value(&config, self.value);
        let display = NSTextField::labelWithString(&NSString::from_str(""), mtm);
        display.setAlignment(objc2_app_kit::NSTextAlignment::Right);
        display.setBordered(true);
        display.setBezeled(true);
        display.setToolTip(Some(&NSString::from_str(
            "Click to type, drag horizontally to adjust",
        )));
        let entry = NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::ZERO);
        entry.setAlignment(objc2_app_kit::NSTextAlignment::Right);
        entry.setHidden(true);
        let view = NumberPickerView::alloc(mtm).set_ivars(NumberPickerIvars {
            config,
            value: Cell::new(value),
            display: display.clone(),
            entry: entry.clone(),
            prefix: self.prefix,
            suffix: self.suffix,
            on_change: self
                .on_change
                .map_or_else(|| Rc::new(|_| {}) as NumberCallback, Rc::from),
            on_commit: self
                .on_commit
                .map_or_else(|| Rc::new(|_| {}) as NumberCallback, Rc::from),
            drag: Cell::new(NumberDrag::begin(value)),
            pointer_locked: Cell::new(false),
            editing: Cell::new(false),
        });
        let view: Retained<NumberPickerView> =
            unsafe { msg_send![super(view), initWithFrame: NSRect::ZERO] };
        unsafe {
            entry.setTarget(Some(&view));
            entry.setAction(Some(sel!(commitText:)));
            entry.setDelegate(Some(ProtocolObject::from_ref(&*view)));
        }
        for child in [&*display, &*entry] {
            child.setTranslatesAutoresizingMaskIntoConstraints(false);
            view.addSubview(child);
            for constraint in [
                child
                    .leadingAnchor()
                    .constraintEqualToAnchor(&view.leadingAnchor()),
                child
                    .trailingAnchor()
                    .constraintEqualToAnchor(&view.trailingAnchor()),
                child.topAnchor().constraintEqualToAnchor(&view.topAnchor()),
                child
                    .bottomAnchor()
                    .constraintEqualToAnchor(&view.bottomAnchor()),
            ] {
                constraint.setActive(true);
            }
        }
        view.widthAnchor()
            .constraintGreaterThanOrEqualToConstant(100.0)
            .setActive(true);
        view.heightAnchor()
            .constraintEqualToConstant(28.0)
            .setActive(true);
        view.refresh();
        NumberPickerParts {
            widget: view.clone().into_super(),
            handle: NumberPickerHandle { view },
        }
    }
}

pub struct NumberPickerParts {
    pub widget: Retained<NSView>,
    pub handle: NumberPickerHandle,
}

#[derive(Clone)]
pub struct NumberPickerHandle {
    view: Retained<NumberPickerView>,
}

impl NumberPickerHandle {
    pub fn set_f64(&self, value: f64) {
        if !self.view.ivars().editing.get() {
            self.view.set_value(fraction_from_f64(value));
        }
    }
    pub fn value(&self) -> Fraction {
        self.view.ivars().value.get()
    }
}

pub struct Number2Picker;

impl Number2Picker {
    pub fn builder(first: f64, second: f64) -> Number2PickerBuilder {
        Number2PickerBuilder {
            first: NumberPicker::builder(first),
            second: NumberPicker::builder(second),
            initial: [fraction_from_f64(first), fraction_from_f64(second)],
            lock: false,
            on_change: None,
        }
    }
}

pub struct Number2PickerBuilder {
    first: NumberPickerBuilder,
    second: NumberPickerBuilder,
    initial: [Fraction; 2],
    lock: bool,
    on_change: Option<PairCallback>,
}

impl Number2PickerBuilder {
    pub fn minimum(mut self, value: f64) -> Self {
        self.first = self.first.minimum(value);
        self.second = self.second.minimum(value);
        self
    }
    pub fn maximum(mut self, value: f64) -> Self {
        self.first = self.first.maximum(value);
        self.second = self.second.maximum(value);
        self
    }
    pub fn digits(mut self, value: usize) -> Self {
        self.first = self.first.digits(value);
        self.second = self.second.digits(value);
        self
    }
    pub fn first_prefix(mut self, value: impl Into<String>) -> Self {
        self.first = self.first.prefix(value);
        self
    }
    pub fn second_prefix(mut self, value: impl Into<String>) -> Self {
        self.second = self.second.prefix(value);
        self
    }
    pub fn unit_name(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.first = self.first.unit_name(value.clone());
        self.second = self.second.unit_name(value);
        self
    }
    pub fn enable_lock(mut self) -> Self {
        self.lock = true;
        self
    }
    pub fn on_change(mut self, callback: impl Fn([f64; 2], usize) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn build_with_handles(self, mtm: MainThreadMarker) -> Number2PickerParts {
        let handles = Rc::new(RefCell::new(None::<[NumberPickerHandle; 2]>));
        let locked = Rc::new(Cell::new(self.lock));
        let ratio = Rc::new(Cell::new(pair_ratio(self.initial[0], self.initial[1])));
        let callback: Rc<dyn Fn([f64; 2], usize)> = match self.on_change {
            Some(callback) => Rc::from(callback),
            None => Rc::new(|_, _| {}),
        };
        let first = self
            .first
            .on_change_fraction({
                let handles = handles.clone();
                let locked = locked.clone();
                let ratio = ratio.clone();
                let callback = callback.clone();
                move |value| {
                    if let Some(handles) = handles.borrow().as_ref() {
                        if locked.get() {
                            handles[1]
                                .set_f64(fraction_as_f64(locked_pair(0, value, ratio.get())[1]));
                        }
                        callback(
                            handles
                                .clone()
                                .map(|handle| fraction_as_f64(handle.value())),
                            0,
                        );
                    }
                }
            })
            .build_with_handle(mtm);
        let second = self
            .second
            .on_change_fraction({
                let handles = handles.clone();
                let locked = locked.clone();
                let ratio = ratio.clone();
                let callback = callback.clone();
                move |value| {
                    if let Some(handles) = handles.borrow().as_ref() {
                        if locked.get() {
                            handles[0]
                                .set_f64(fraction_as_f64(locked_pair(1, value, ratio.get())[0]));
                        }
                        callback(
                            handles
                                .clone()
                                .map(|handle| fraction_as_f64(handle.value())),
                            1,
                        );
                    }
                }
            })
            .build_with_handle(mtm);
        let pair_handles = [first.handle.clone(), second.handle.clone()];
        handles.replace(Some(pair_handles.clone()));
        let row = stack(false, 4.0, mtm);
        row.addArrangedSubview(&first.widget);
        row.addArrangedSubview(&second.widget);
        if self.lock {
            let button = unsafe {
                NSButton::checkboxWithTitle_target_action(
                    &NSString::from_str("Lock"),
                    None,
                    None,
                    mtm,
                )
            };
            button.setState(NSControlStateValueOn);
            action::attach(
                &button,
                move |control| {
                    let active = control
                        .downcast_ref::<NSButton>()
                        .expect("pair lock sender")
                        .state()
                        == NSControlStateValueOn;
                    locked.set(active);
                    if active && let Some(handles) = handles.borrow().as_ref() {
                        ratio.set(pair_ratio(handles[0].value(), handles[1].value()));
                    }
                },
                mtm,
            );
            row.addArrangedSubview(&button);
        }
        Number2PickerParts {
            widget: row,
            first: pair_handles[0].clone(),
            second: pair_handles[1].clone(),
        }
    }
}

pub struct Number2PickerParts {
    pub widget: Retained<NSStackView>,
    pub first: NumberPickerHandle,
    pub second: NumberPickerHandle,
}

pub struct Number3Picker;

impl Number3Picker {
    pub fn builder(values: [f64; 3]) -> Number3PickerBuilder {
        Number3PickerBuilder {
            builders: values.map(NumberPicker::builder),
            initial: values.map(fraction_from_f64),
            prefixes: [String::new(), String::new(), String::new()],
            lock: false,
            callbacks: [None, None, None],
        }
    }
}

pub struct Number3PickerBuilder {
    builders: [NumberPickerBuilder; 3],
    initial: [Fraction; 3],
    prefixes: [String; 3],
    lock: bool,
    callbacks: [Option<ScalarCallback>; 3],
}

impl Number3PickerBuilder {
    pub fn prefixes(mut self, values: [&str; 3]) -> Self {
        self.prefixes = values.map(str::to_string);
        self
    }
    pub fn enable_lock(mut self) -> Self {
        self.lock = true;
        self
    }
    pub fn on_change(mut self, component: usize, callback: impl Fn(f64) + 'static) -> Self {
        *self
            .callbacks
            .get_mut(component)
            .expect("number3 component") = Some(Box::new(callback));
        self
    }
    pub fn build_with_handles(mut self, mtm: MainThreadMarker) -> Number3PickerParts {
        let shared = Rc::new(RefCell::new(None::<[NumberPickerHandle; 3]>));
        let locked = Rc::new(Cell::new(self.lock));
        let ratios = Rc::new(Cell::new(triple_ratios(self.initial)));
        let callbacks = self
            .callbacks
            .map(|callback| callback.map_or_else(|| Rc::new(|_| {}) as Rc<dyn Fn(f64)>, Rc::from));
        let mut parts = Vec::new();
        for component in 0..3 {
            let shared = shared.clone();
            let locked = locked.clone();
            let ratios = ratios.clone();
            let callbacks = callbacks.clone();
            let builder =
                std::mem::replace(&mut self.builders[component], NumberPicker::builder(0.0))
                    .prefix(self.prefixes[component].clone())
                    .on_change_fraction(move |value| {
                        if let Some(handles) = shared.borrow().as_ref() {
                            if locked.get() {
                                let next = locked_triple(component, value, ratios.get());
                                for index in 0..3 {
                                    if index != component {
                                        handles[index].set_f64(fraction_as_f64(next[index]));
                                    }
                                }
                            }
                            callbacks[component](fraction_as_f64(value));
                        }
                    })
                    .build_with_handle(mtm);
            parts.push(builder);
        }
        let [first, second, third]: [NumberPickerParts; 3] =
            parts.try_into().ok().expect("three number parts");
        let handles = [
            first.handle.clone(),
            second.handle.clone(),
            third.handle.clone(),
        ];
        shared.replace(Some(handles.clone()));
        let row = stack(false, 4.0, mtm);
        row.addArrangedSubview(&first.widget);
        row.addArrangedSubview(&second.widget);
        row.addArrangedSubview(&third.widget);
        if self.lock {
            let button = unsafe {
                NSButton::checkboxWithTitle_target_action(
                    &NSString::from_str("Lock"),
                    None,
                    None,
                    mtm,
                )
            };
            button.setState(NSControlStateValueOn);
            action::attach(
                &button,
                move |control| {
                    let active = control
                        .downcast_ref::<NSButton>()
                        .expect("vector lock sender")
                        .state()
                        == NSControlStateValueOn;
                    locked.set(active);
                    if active && let Some(handles) = shared.borrow().as_ref() {
                        ratios.set(triple_ratios(handles.clone().map(|handle| handle.value())));
                    }
                },
                mtm,
            );
            row.addArrangedSubview(&button);
        }
        Number3PickerParts {
            widget: row,
            handles,
        }
    }
}

pub struct Number3PickerParts {
    pub widget: Retained<NSStackView>,
    pub handles: [NumberPickerHandle; 3],
}
