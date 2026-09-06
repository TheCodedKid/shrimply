use crate::action;
use block2::RcBlock;
use objc2::ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_setAssociatedObject};
use objc2::{ClassType, DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2::rc::{Retained, Weak};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSCellImagePosition, NSColor, NSColorWell, NSControlStateValueOff,
    NSControlStateValueOn, NSEvent, NSFont, NSGlassEffectView, NSGlassEffectViewStyle, NSImage,
    NSLayoutAttribute, NSLayoutConstraintOrientation, NSLayoutPriorityDefaultLow,
    NSLayoutPriorityRequired, NSPasteboard, NSPasteboardTypeString, NSPopover, NSPopoverBehavior,
    NSProgressIndicator, NSProgressIndicatorStyle, NSScrollView, NSSearchField,
    NSSegmentedControl, NSStackView, NSStackViewDistribution, NSSwitch, NSTextAlignment, NSTextField,
    NSUserInterfaceLayoutOrientation, NSView, NSViewController, NSAutoresizingMaskOptions,
};
use objc2_foundation::{
    MainThreadMarker, NSObjectProtocol, NSPoint, NSRect, NSRectEdge, NSSize, NSString, NSTimer,
};
use shrimply_math_color::Color;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

pub use shrimply_component_core::selector::StringChoice;

const CONTROL_HEIGHT: f64 = 28.0;
const SEARCH_FIELD_OUTER_INSET: f64 = 10.0;
const SEARCH_FIELD_VERTICAL_INSET: f64 = 8.0;
const SEARCH_ROW_HORIZONTAL_INSET: f64 = 8.0;
const SEARCH_VISIBLE_ROWS: usize = 8;
static SEARCH_POPOVER_KEY: u8 = 0;

pub fn row_stack(spacing: f64, mtm: MainThreadMarker) -> Retained<NSStackView> {
    let view = NSStackView::initWithFrame(NSStackView::alloc(mtm), NSRect::ZERO);
    view.setOrientation(NSUserInterfaceLayoutOrientation::Horizontal);
    view.setAlignment(NSLayoutAttribute::CenterY);
    view.setSpacing(spacing);
    view.setDistribution(NSStackViewDistribution::Fill);
    view
}

pub fn column_stack(spacing: f64, mtm: MainThreadMarker) -> Retained<NSStackView> {
    let view = NSStackView::initWithFrame(NSStackView::alloc(mtm), NSRect::ZERO);
    view.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
    view.setAlignment(NSLayoutAttribute::Leading);
    view.setSpacing(spacing);
    view.setDistribution(NSStackViewDistribution::Fill);
    view
}

pub fn column_append(column: &NSStackView, child: &NSView) {
    column.addArrangedSubview(child);
    child.setTranslatesAutoresizingMaskIntoConstraints(false);
    child.setContentHuggingPriority_forOrientation(
        NSLayoutPriorityDefaultLow,
        NSLayoutConstraintOrientation::Horizontal,
    );
    child
        .widthAnchor()
        .constraintEqualToAnchor(&column.widthAnchor())
        .setActive(true);
}

pub fn control_row(label: &str, child: &NSView, mtm: MainThreadMarker) -> Retained<NSView> {
    control_row_with_suffix(label, child, None, mtm)
}

pub fn control_row_with_suffix(
    label: &str,
    child: &NSView,
    suffix: Option<&NSView>,
    mtm: MainThreadMarker,
) -> Retained<NSView> {
    let row = NSView::new(mtm);
    let label = NSTextField::labelWithString(&NSString::from_str(label), mtm);
    let column_measure = NSTextField::labelWithString(
        &NSString::from_str(
            &"0".repeat(shrimply_component_core::layout::CONTROL_ROW_LABEL_COLUMNS),
        ),
        mtm,
    );
    let label_width = label
        .intrinsicContentSize()
        .width
        .max(column_measure.intrinsicContentSize().width);
    label.setTextColor(Some(&NSColor::secondaryLabelColor()));
    label.setAlignment(NSTextAlignment::Left);
    label.setTranslatesAutoresizingMaskIntoConstraints(false);
    child.setTranslatesAutoresizingMaskIntoConstraints(false);
    row.addSubview(&label);
    row.addSubview(child);
    for constraint in [
        label.leadingAnchor().constraintEqualToAnchor(&row.leadingAnchor()),
        label.widthAnchor().constraintEqualToConstant(label_width),
        label.centerYAnchor().constraintEqualToAnchor(&row.centerYAnchor()),
        child.leadingAnchor().constraintEqualToAnchor_constant(
            &label.trailingAnchor(),
            f64::from(shrimply_component_core::layout::CONTROL_ROW_GAP),
        ),
        child.topAnchor().constraintEqualToAnchor(&row.topAnchor()),
        child.bottomAnchor().constraintEqualToAnchor(&row.bottomAnchor()),
        child.heightAnchor().constraintGreaterThanOrEqualToConstant(CONTROL_HEIGHT),
    ] {
        constraint.setActive(true);
    }
    if let Some(suffix) = suffix {
        suffix.setTranslatesAutoresizingMaskIntoConstraints(false);
        suffix.setContentHuggingPriority_forOrientation(
            NSLayoutPriorityRequired,
            NSLayoutConstraintOrientation::Horizontal,
        );
        suffix.setContentCompressionResistancePriority_forOrientation(
            NSLayoutPriorityRequired,
            NSLayoutConstraintOrientation::Horizontal,
        );
        row.addSubview(suffix);
        let suffix_width = suffix.fittingSize().width;
        for constraint in [
            suffix.leadingAnchor().constraintEqualToAnchor_constant(
                &child.trailingAnchor(),
                f64::from(shrimply_component_core::layout::CONTROL_ROW_GAP),
            ),
            suffix.trailingAnchor().constraintEqualToAnchor(&row.trailingAnchor()),
            suffix.centerYAnchor().constraintEqualToAnchor(&row.centerYAnchor()),
            suffix.widthAnchor().constraintEqualToConstant(suffix_width),
        ] {
            constraint.setActive(true);
        }
    } else {
        child
            .trailingAnchor()
            .constraintEqualToAnchor(&row.trailingAnchor())
            .setActive(true);
    }
    row
}

pub struct StringSelector {
    view: Retained<NSView>,
}

impl StringSelector {
    pub fn new(
        value: &str,
        choices: Vec<StringChoice>,
        on_change: impl Fn(String) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        if shrimply_component_core::selector::searchable(choices.len()) {
            let selected = shrimply_component_core::selector::selected_index(value, &choices);
            let title = choices[selected].label.clone();
            let button = search_menu_button(
                &title,
                None,
                "Search choices",
                choices,
                true,
                on_change,
                mtm,
            );
            Self {
                view: button.into_super().into_super(),
            }
        } else {
            let popup = objc2_app_kit::NSPopUpButton::initWithFrame_pullsDown(
                objc2_app_kit::NSPopUpButton::alloc(mtm),
                NSRect::ZERO,
                false,
            );
            for choice in &choices {
                popup.addItemWithTitle(&NSString::from_str(&choice.label));
            }
            popup.selectItemAtIndex(shrimply_component_core::selector::selected_index(
                value, &choices,
            ) as isize);
            let choices = Rc::new(choices);
            action::attach(
                &popup,
                move |control| {
                    let index = control
                        .downcast_ref::<objc2_app_kit::NSPopUpButton>()
                        .expect("selector sender")
                        .indexOfSelectedItem();
                    if let Ok(index) = usize::try_from(index)
                        && let Some(choice) = choices.get(index)
                    {
                        on_change(choice.value.clone());
                    }
                },
                mtm,
            );
            Self {
                view: popup.into_super().into_super().into_super(),
            }
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
}

type SearchSelection = Rc<dyn Fn(String)>;

struct SearchFieldIvars {
    on_navigation: Box<dyn Fn(u16) -> bool>,
}

define_class!(
    #[unsafe(super(NSSearchField))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SearchFieldIvars]
    struct SearchField;

    unsafe impl NSObjectProtocol for SearchField {}

    impl SearchField {
        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            if !(self.ivars().on_navigation)(event.keyCode()) {
                unsafe { let _: () = msg_send![super(self), keyDown: event]; }
            }
        }
    }
);

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ()]
    struct SearchResultsView;

    unsafe impl NSObjectProtocol for SearchResultsView {}

    impl SearchResultsView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool { true }
    }
);

struct SearchResultIvars {
    list: Weak<NSView>,
    popover: Weak<NSPopover>,
}

define_class!(
    #[unsafe(super(NSButton))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SearchResultIvars]
    struct SearchResult;

    unsafe impl NSObjectProtocol for SearchResult {}

    impl SearchResult {
        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            match event.keyCode() {
                125 => self.focus_sibling(1),
                126 => self.focus_sibling(-1),
                53 => {
                    if let Some(popover) = self.ivars().popover.load() { popover.close(); }
                }
                _ => unsafe { let _: () = msg_send![super(self), keyDown: event]; },
            }
        }
    }
);

impl SearchResult {
    fn focus_sibling(&self, offset: isize) {
        let Some(list) = self.ivars().list.load() else { return };
        let children = list.subviews();
        let Some(index) = children
            .iter()
            .position(|view| {
                std::ptr::eq(
                    &*view,
                    self.as_super().as_super().as_super(),
                )
            })
        else {
            return;
        };
        let Some(next) = index.checked_add_signed(offset) else { return };
        let Some(view) = children.iter().nth(next) else { return };
        let button = view
            .downcast::<NSButton>()
            .expect("search result list only contains buttons");
        button
            .window()
            .expect("search result must be attached")
            .makeFirstResponder(Some(&button));
    }
}

fn search_menu_button(
    title: &str,
    icon: Option<&str>,
    placeholder: &str,
    choices: Vec<StringChoice>,
    update_title: bool,
    on_select: impl Fn(String) + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSButton> {
    let button = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str(title), None, None, mtm)
    };
    button.setAlignment(NSTextAlignment::Left);
    if let Some(icon) = icon {
        button.setImage(Some(&symbol(icon, title)));
        button.setImagePosition(NSCellImagePosition::ImageLeading);
        button.setBordered(false);
    } else {
        button.setImage(Some(&symbol("chevron.up.chevron.down", "Open choices")));
        button.setImagePosition(NSCellImagePosition::ImageRight);
    }

    let choices = Rc::new(choices);
    let selected = Rc::new(RefCell::new(
        choices
            .iter()
            .find(|choice| choice.label == title)
            .map(|choice| choice.value.clone()),
    ));
    let on_select = Rc::new(on_select) as SearchSelection;
    let parts = search_popover(
        placeholder,
        choices,
        selected,
        Some(Weak::new(&*button)),
        update_title,
        on_select,
        mtm,
    );
    action::attach(
        &button,
        move |control| {
            (parts.refresh)();
            let button = control
                .downcast_ref::<NSButton>()
                .expect("search menu button sender");
            parts.popover.showRelativeToRect_ofView_preferredEdge(
                button.bounds(),
                button,
                NSRectEdge::MaxY,
            );
            button
                .window()
                .expect("search menu must be attached")
                .makeFirstResponder(Some(&parts.search));
        },
        mtm,
    );
    button
}

struct SearchPopoverParts {
    popover: Retained<NSPopover>,
    search: Retained<SearchField>,
    refresh: Rc<dyn Fn()>,
}

fn search_popover(
    placeholder: &str,
    choices: Rc<Vec<StringChoice>>,
    selected: Rc<RefCell<Option<String>>>,
    main_button: Option<Weak<NSButton>>,
    update_title: bool,
    on_select: SearchSelection,
    mtm: MainThreadMarker,
) -> SearchPopoverParts {
    let popover = NSPopover::new(mtm);
    popover.setBehavior(NSPopoverBehavior::Transient);
    let content = column_stack(0.0, mtm);
    let list = SearchResultsView::alloc(mtm).set_ivars(());
    let list: Retained<SearchResultsView> =
        unsafe { msg_send![super(list), initWithFrame: NSRect::ZERO] };
    let search = SearchField::alloc(mtm).set_ivars(SearchFieldIvars {
        on_navigation: {
            let list = Weak::new(&*list);
            let popover = Weak::new(&*popover);
            Box::new(move |key_code| {
                let Some(list) = list.load() else { return false };
                match key_code {
                    36 | 76 => first_result(&list).is_some_and(|button| {
                        unsafe { button.performClick(None) };
                        true
                    }),
                    125 => focus_result(&list, false),
                    126 => focus_result(&list, true),
                    53 => {
                        if let Some(popover) = popover.load() { popover.close(); }
                        true
                    }
                    _ => false,
                }
            })
        },
    });
    let search: Retained<SearchField> =
        unsafe { msg_send![super(search), initWithFrame: NSRect::ZERO] };
    search.setPlaceholderString(Some(&NSString::from_str(placeholder)));
    search.setSendsSearchStringImmediately(true);
    search.setTranslatesAutoresizingMaskIntoConstraints(false);
    let search_container = NSView::new(mtm);
    search_container.addSubview(&search);
    for constraint in [
        search
            .leadingAnchor()
            .constraintEqualToAnchor_constant(
                &search_container.leadingAnchor(),
                SEARCH_FIELD_OUTER_INSET,
            ),
        search
            .trailingAnchor()
            .constraintEqualToAnchor_constant(
                &search_container.trailingAnchor(),
                -SEARCH_FIELD_OUTER_INSET,
            ),
        search
            .topAnchor()
            .constraintEqualToAnchor_constant(
                &search_container.topAnchor(),
                SEARCH_FIELD_VERTICAL_INSET,
            ),
        search
            .bottomAnchor()
            .constraintEqualToAnchor_constant(
                &search_container.bottomAnchor(),
                -SEARCH_FIELD_VERTICAL_INSET,
            ),
    ] {
        constraint.setActive(true);
    }
    let scroll = NSScrollView::initWithFrame(
        NSScrollView::alloc(mtm),
        NSRect::new(NSPoint::ZERO, NSSize::new(280.0, 240.0)),
    );
    scroll.setHasVerticalScroller(true);
    scroll.setAutohidesScrollers(true);
    scroll.setBorderType(objc2_app_kit::NSBorderType::NoBorder);
    scroll.setDocumentView(Some(&list));
    list.setFrameSize(NSSize::new(280.0, 240.0));
    list.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
    let results_height = CONTROL_HEIGHT * SEARCH_VISIBLE_ROWS as f64;
    scroll
        .heightAnchor()
        .constraintEqualToConstant(results_height)
        .setActive(true);
    column_append(&content, &search_container);
    column_append(&content, &scroll);
    let controller = NSViewController::new(mtm);
    controller.setView(&content);
    controller.setPreferredContentSize(NSSize::new(
        292.0,
        CONTROL_HEIGHT + SEARCH_FIELD_VERTICAL_INSET * 2.0 + results_height,
    ));
    popover.setContentViewController(Some(&controller));

    let weak_button = main_button;
    let weak_popover = Weak::new(&*popover);
    populate_search_results(
        &list,
        "",
        &choices,
        &selected,
        &weak_button,
        &weak_popover,
        update_title,
        &on_select,
        mtm,
    );
    action::attach(
        &search,
        {
            let list = list.clone();
            let choices = choices.clone();
            let selected = selected.clone();
            let weak_button = weak_button.clone();
            let weak_popover = weak_popover.clone();
            let on_select = on_select.clone();
            move |control| {
                let query = control
                    .downcast_ref::<NSSearchField>()
                    .expect("search menu sender")
                    .stringValue()
                    .to_string();
                populate_search_results(
                    &list,
                    &query,
                    &choices,
                    &selected,
                    &weak_button,
                    &weak_popover,
                    update_title,
                    &on_select,
                    mtm,
                );
            }
        },
        mtm,
    );
    let refresh = Rc::new({
        let search = search.clone();
        let list = list.clone();
        let choices = choices.clone();
        let selected = selected.clone();
        let weak_button = weak_button.clone();
        let weak_popover = weak_popover.clone();
        let on_select = on_select.clone();
        move || {
            search.setStringValue(&NSString::new());
            populate_search_results(
                &list,
                "",
                &choices,
                &selected,
                &weak_button,
                &weak_popover,
                update_title,
                &on_select,
                mtm,
            );
        }
    }) as Rc<dyn Fn()>;
    SearchPopoverParts { popover, search, refresh }
}

pub(crate) fn show_searchable_popover_at(
    view: &NSView,
    point: NSPoint,
    placeholder: &str,
    choices: Vec<StringChoice>,
    selected: String,
    on_select: impl Fn(String) + 'static,
    mtm: MainThreadMarker,
) {
    let parts = search_popover(
        placeholder,
        Rc::new(choices),
        Rc::new(RefCell::new(Some(selected))),
        None,
        false,
        Rc::new(on_select),
        mtm,
    );
    (parts.refresh)();
    parts.popover.showRelativeToRect_ofView_preferredEdge(
        NSRect::new(point, NSSize::new(1.0, 1.0)),
        view,
        NSRectEdge::MaxY,
    );
    view.window()
        .expect("search popover host must be attached")
        .makeFirstResponder(Some(&parts.search));
    unsafe {
        objc_setAssociatedObject(
            std::ptr::from_ref(view).cast_mut().cast(),
            std::ptr::from_ref(&SEARCH_POPOVER_KEY).cast::<c_void>(),
            Retained::as_ptr(&parts.popover).cast_mut().cast(),
            OBJC_ASSOCIATION_RETAIN_NONATOMIC,
        );
    }
}

fn first_result(list: &NSView) -> Option<Retained<NSButton>> {
    list.subviews()
        .iter()
        .next()
        .and_then(|view| view.downcast::<NSButton>().ok())
}

fn focus_result(list: &NSView, last: bool) -> bool {
    let children = list.subviews();
    let result = if last {
        children.iter().last()
    } else {
        children.iter().next()
    };
    result
        .and_then(|view| view.downcast::<NSButton>().ok())
        .is_some_and(|button| {
            button
                .window()
                .expect("search result must be attached")
                .makeFirstResponder(Some(&button));
            true
        })
}

#[allow(clippy::too_many_arguments)]
fn populate_search_results(
    list: &NSView,
    query: &str,
    choices: &Rc<Vec<StringChoice>>,
    selected: &Rc<RefCell<Option<String>>>,
    main_button: &Option<Weak<NSButton>>,
    popover: &Weak<NSPopover>,
    update_title: bool,
    on_select: &SearchSelection,
    mtm: MainThreadMarker,
) {
    let children = list.subviews();
    for child in children.iter() {
        child.removeFromSuperview();
    }
    let matches = choices.iter().filter(|choice| {
        shrimply_component_core::selector::matches_query(&choice.label, query)
    });
    let mut row_index = 0usize;
    for choice in matches {
        let row = SearchResult::alloc(mtm).set_ivars(SearchResultIvars {
            list: Weak::new(list),
            popover: popover.clone(),
        });
        let row: Retained<SearchResult> = unsafe {
            msg_send![
                super(row),
                initWithFrame: NSRect::new(
                    NSPoint::new(
                        SEARCH_ROW_HORIZONTAL_INSET,
                        row_index as f64 * CONTROL_HEIGHT,
                    ),
                    NSSize::new(
                        280.0 - SEARCH_ROW_HORIZONTAL_INSET * 2.0,
                        CONTROL_HEIGHT,
                    ),
                )
            ]
        };
        row.setTitle(&NSString::from_str(&choice.label));
        row.setAlignment(NSTextAlignment::Left);
        row.setBordered(false);
        row.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
        if selected.borrow().as_ref() == Some(&choice.value) {
            row.setImage(Some(&symbol("checkmark", "Selected")));
            row.setImagePosition(NSCellImagePosition::ImageTrailing);
        }
        let value = choice.value.clone();
        let label = choice.label.clone();
        let selected = selected.clone();
        let main_button = main_button.clone();
        let popover = popover.clone();
        let on_select = on_select.clone();
        action::attach(
            &row,
            move |_| {
                *selected.borrow_mut() = Some(value.clone());
                if update_title
                    && let Some(button) = main_button.as_ref().and_then(Weak::load)
                {
                    button.setTitle(&NSString::from_str(&label));
                }
                on_select(value.clone());
                if let Some(popover) = popover.load() {
                    popover.close();
                }
            },
            mtm,
        );
        list.addSubview(&row);
        row_index += 1;
    }
    list.setFrameSize(NSSize::new(
        280.0,
        (row_index.max(SEARCH_VISIBLE_ROWS) as f64) * CONTROL_HEIGHT,
    ));
}

pub fn switch_row(
    label: &str,
    tooltip: Option<&str>,
    active: bool,
    on_change: impl Fn(bool) + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSView> {
    let toggle = NSSwitch::new(mtm);
    toggle.setState(if active {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    });
    toggle.setToolTip(tooltip.map(NSString::from_str).as_deref());
    action::attach(
        &toggle,
        move |control| {
            on_change(
                control
                    .downcast_ref::<NSSwitch>()
                    .expect("switch sender")
                    .state()
                    == NSControlStateValueOn,
            )
        },
        mtm,
    );
    let aligned = NSView::new(mtm);
    toggle.setTranslatesAutoresizingMaskIntoConstraints(false);
    aligned.addSubview(&toggle);
    for constraint in [
        toggle.trailingAnchor().constraintEqualToAnchor(&aligned.trailingAnchor()),
        toggle.centerYAnchor().constraintEqualToAnchor(&aligned.centerYAnchor()),
        aligned.heightAnchor().constraintGreaterThanOrEqualToAnchor(&toggle.heightAnchor()),
    ] {
        constraint.setActive(true);
    }
    control_row(label, &aligned, mtm)
}

pub struct ColorPicker {
    view: Retained<NSColorWell>,
}

impl ColorPicker {
    pub fn new(
        color: Color<u8>,
        on_change: impl Fn(Color<u8>) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let well = NSColorWell::initWithFrame(NSColorWell::alloc(mtm), NSRect::ZERO);
        well.setColor(&native_color(color));
        action::attach(
            &well,
            move |control| {
                let color = control
                    .downcast_ref::<NSColorWell>()
                    .expect("color sender")
                    .color()
                    .colorUsingColorSpace(&objc2_app_kit::NSColorSpace::sRGBColorSpace())
                    .expect("convert color to sRGB");
                on_change(Color::new(
                    channel(color.redComponent()),
                    channel(color.greenComponent()),
                    channel(color.blueComponent()),
                    channel(color.alphaComponent()),
                ));
            },
            mtm,
        );
        Self { view: well }
    }

    pub fn view(&self) -> &NSColorWell {
        &self.view
    }
}

fn native_color(color: Color<u8>) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
        f64::from(color.a) / 255.0,
    )
}

fn channel(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn split_button(
    primary: &str,
    secondary: &str,
    on_primary: impl Fn() + 'static,
    on_secondary: impl Fn() + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSStackView> {
    let row = row_stack(0.0, mtm);
    let primary = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str(primary), None, None, mtm)
    };
    primary.setBezelStyle(NSBezelStyle::Push);
    primary.setContentHuggingPriority_forOrientation(
        NSLayoutPriorityDefaultLow,
        NSLayoutConstraintOrientation::Horizontal,
    );
    action::attach(&primary, move |_| on_primary(), mtm);
    let disclosure = unsafe {
        NSButton::buttonWithImage_target_action(
            &symbol("chevron.down", secondary),
            None,
            None,
            mtm,
        )
    };
    disclosure.setBezelStyle(NSBezelStyle::Push);
    disclosure.setToolTip(Some(&NSString::from_str(secondary)));
    disclosure.setContentHuggingPriority_forOrientation(
        NSLayoutPriorityRequired,
        NSLayoutConstraintOrientation::Horizontal,
    );
    let popover = NSPopover::new(mtm);
    popover.setBehavior(NSPopoverBehavior::Transient);
    let secondary_button = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str(secondary), None, None, mtm)
    };
    secondary_button.setBordered(false);
    secondary_button.setAlignment(NSTextAlignment::Left);
    let controller = NSViewController::new(mtm);
    controller.setView(&secondary_button);
    let size = secondary_button.intrinsicContentSize();
    controller.setPreferredContentSize(NSSize::new(size.width.max(120.0), CONTROL_HEIGHT));
    popover.setContentViewController(Some(&controller));
    action::attach(
        &secondary_button,
        {
            let popover = popover.clone();
            move |_| {
                popover.close();
                on_secondary();
            }
        },
        mtm,
    );
    action::attach(
        &disclosure,
        move |control| {
            let disclosure = control
                .downcast_ref::<NSButton>()
                .expect("split disclosure sender");
            popover.showRelativeToRect_ofView_preferredEdge(
                disclosure.bounds(),
                disclosure,
                NSRectEdge::MaxY,
            );
        },
        mtm,
    );
    row.addArrangedSubview(&primary);
    row.addArrangedSubview(&disclosure);
    row
}

#[derive(Clone, Copy)]
pub enum ProgressButtonState {
    Idle,
    Indeterminate,
    Progress(f64),
}

pub struct ProgressButton {
    button: Retained<NSButton>,
    indicator: Retained<NSProgressIndicator>,
}

impl ProgressButton {
    pub fn new(label: &str, mtm: MainThreadMarker) -> Self {
        let indicator =
            NSProgressIndicator::initWithFrame(NSProgressIndicator::alloc(mtm), NSRect::ZERO);
        indicator.setStyle(NSProgressIndicatorStyle::Bar);
        indicator.setDisplayedWhenStopped(false);
        indicator.setHidden(true);
        let button = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(label), None, None, mtm)
        };
        button.setContentHuggingPriority_forOrientation(
            NSLayoutPriorityRequired,
            NSLayoutConstraintOrientation::Horizontal,
        );
        indicator.setTranslatesAutoresizingMaskIntoConstraints(false);
        button.addSubview(&indicator);
        for constraint in [
            indicator.leadingAnchor().constraintEqualToAnchor_constant(
                &button.leadingAnchor(),
                5.0,
            ),
            indicator.trailingAnchor().constraintEqualToAnchor_constant(
                &button.trailingAnchor(),
                -5.0,
            ),
            indicator.bottomAnchor().constraintEqualToAnchor_constant(
                &button.bottomAnchor(),
                -2.0,
            ),
            indicator.heightAnchor().constraintEqualToConstant(2.0),
        ] {
            constraint.setActive(true);
        }
        Self { button, indicator }
    }

    pub fn view(&self) -> &NSButton {
        &self.button
    }

    pub fn set_state(&self, state: ProgressButtonState) {
        match state {
            ProgressButtonState::Idle => {
                unsafe {
                    self.indicator.stopAnimation(None);
                }
                self.indicator.setHidden(true);
            }
            ProgressButtonState::Indeterminate => {
                self.indicator.setIndeterminate(true);
                self.indicator.setHidden(false);
                unsafe {
                    self.indicator.startAnimation(None);
                }
            }
            ProgressButtonState::Progress(value) => {
                unsafe {
                    self.indicator.stopAnimation(None);
                }
                self.indicator.setIndeterminate(false);
                self.indicator.setMinValue(0.0);
                self.indicator.setMaxValue(1.0);
                self.indicator.setDoubleValue(value.clamp(0.0, 1.0));
                self.indicator.setHidden(false);
            }
        }
    }
}

pub struct ReadOnlyField {
    root: Retained<NSStackView>,
}

impl ReadOnlyField {
    pub fn new(value: &str, right_aligned: bool, mtm: MainThreadMarker) -> Self {
        let root = row_stack(4.0, mtm);
        let field = NSTextField::labelWithString(&NSString::from_str(value), mtm);
        field.setSelectable(true);
        field.setAlignment(if right_aligned {
            NSTextAlignment::Right
        } else {
            NSTextAlignment::Left
        });
        field.setContentHuggingPriority_forOrientation(
            NSLayoutPriorityDefaultLow,
            NSLayoutConstraintOrientation::Horizontal,
        );
        root.addArrangedSubview(&field);
        Self { root }
    }

    pub fn with_action(
        value: &str,
        action_label: &str,
        on_action: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let this = Self::new(value, true, mtm);
        let button = unsafe {
            NSButton::buttonWithImage_target_action(
                &symbol("folder", action_label),
                None,
                None,
                mtm,
            )
        };
        button.setToolTip(Some(&NSString::from_str(action_label)));
        button.setContentHuggingPriority_forOrientation(
            NSLayoutPriorityRequired,
            NSLayoutConstraintOrientation::Horizontal,
        );
        action::attach(&button, move |_| on_action(), mtm);
        this.root.addArrangedSubview(&button);
        this
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }
}

pub struct Tabs {
    root: Retained<NSView>,
}

impl Tabs {
    pub fn new(pages: Vec<(&str, Retained<NSView>)>, mtm: MainThreadMarker) -> Self {
        assert!(!pages.is_empty(), "tabs need at least one page");
        let root = NSView::new(mtm);
        let selector = unsafe {
            NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &objc2_foundation::NSArray::from_retained_slice(
                    &pages
                        .iter()
                        .map(|(label, _)| NSString::from_str(label))
                        .collect::<Vec<_>>(),
                ),
                objc2_app_kit::NSSegmentSwitchTracking::SelectOne,
                None,
                None,
                mtm,
            )
        };
        selector.setSelectedSegment(0);
        for (index, (label, _)) in pages.iter().enumerate() {
            let icon = match *label {
                "General" => "gearshape",
                "Info" => "info.circle",
                "Log" => "terminal",
                _ => "square.grid.2x2",
            };
            selector.setImage_forSegment(Some(&symbol(icon, label)), index as isize);
        }
        let content = NSView::initWithFrame(NSView::alloc(mtm), NSRect::ZERO);
        let page_views = Rc::new(pages.into_iter().map(|(_, view)| view).collect::<Vec<_>>());
        for (index, page) in page_views.iter().enumerate() {
            page.setTranslatesAutoresizingMaskIntoConstraints(false);
            page.setHidden(index != 0);
            content.addSubview(page);
            for constraint in [
                page.leadingAnchor()
                    .constraintEqualToAnchor(&content.leadingAnchor()),
                page.trailingAnchor()
                    .constraintEqualToAnchor(&content.trailingAnchor()),
                page.topAnchor()
                    .constraintEqualToAnchor(&content.topAnchor()),
                page.bottomAnchor()
                    .constraintEqualToAnchor(&content.bottomAnchor()),
            ] {
                constraint.setActive(true);
            }
        }
        let callback_pages = page_views.clone();
        action::attach(
            &selector,
            move |control| {
                let selected = control
                    .downcast_ref::<NSSegmentedControl>()
                    .expect("tabs sender")
                    .selectedSegment();
                for (index, page) in callback_pages.iter().enumerate() {
                    page.setHidden(index as isize != selected);
                }
            },
            mtm,
        );
        selector.setTranslatesAutoresizingMaskIntoConstraints(false);
        content.setTranslatesAutoresizingMaskIntoConstraints(false);
        root.addSubview(&selector);
        root.addSubview(&content);
        for constraint in [
            selector.leadingAnchor().constraintEqualToAnchor_constant(&root.leadingAnchor(), 16.0),
            selector.trailingAnchor().constraintEqualToAnchor_constant(&root.trailingAnchor(), -16.0),
            selector.topAnchor().constraintEqualToAnchor_constant(&root.topAnchor(), 12.0),
            content.leadingAnchor().constraintEqualToAnchor(&root.leadingAnchor()),
            content.trailingAnchor().constraintEqualToAnchor(&root.trailingAnchor()),
            content.topAnchor().constraintEqualToAnchor_constant(&selector.bottomAnchor(), 8.0),
            content.bottomAnchor().constraintEqualToAnchor(&root.bottomAnchor()),
        ] {
            constraint.setActive(true);
        }
        Self { root }
    }

    pub fn view(&self) -> &NSView {
        &self.root
    }
}

struct PlaybackSurfaceIvars {
    on_toggle: Box<dyn Fn()>,
    on_speed: Box<dyn Fn()>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = PlaybackSurfaceIvars]
    struct PlaybackSurface;

    unsafe impl NSObjectProtocol for PlaybackSurface {}

    impl PlaybackSurface {
        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool { true }

        #[unsafe(method(hitTest:))]
        fn hit_test(&self, point: NSPoint) -> Option<&NSView> {
            let hit: Option<&NSView> = unsafe { msg_send![super(self), hitTest: point] };
            hit.map(|_| self.as_super())
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, _event: &NSEvent) {
            self.window()
                .expect("playback shortcut surface must be attached")
                .makeFirstResponder(Some(self));
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            match event.keyCode() {
                49 => (self.ivars().on_toggle)(),
                37 => (self.ivars().on_speed)(),
                _ => unsafe { let _: () = msg_send![super(self), keyDown: event]; },
            }
        }
    }
);

pub fn playback_shortcuts(
    on_toggle: impl Fn() + 'static,
    on_speed: impl Fn() + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSView> {
    let surface = PlaybackSurface::alloc(mtm).set_ivars(PlaybackSurfaceIvars {
        on_toggle: Box::new(on_toggle),
        on_speed: Box::new(on_speed),
    });
    let surface: Retained<PlaybackSurface> =
        unsafe { msg_send![super(surface), initWithFrame: NSRect::ZERO] };
    let label = NSTextField::labelWithString(
        &NSString::from_str("Click, then press Space or L"),
        mtm,
    );
    label.setTextColor(Some(&NSColor::secondaryLabelColor()));
    label.setTranslatesAutoresizingMaskIntoConstraints(false);
    surface.addSubview(&label);
    for constraint in [
        label.centerXAnchor().constraintEqualToAnchor(&surface.centerXAnchor()),
        label.centerYAnchor().constraintEqualToAnchor(&surface.centerYAnchor()),
        surface.heightAnchor().constraintEqualToConstant(44.0),
    ] {
        constraint.setActive(true);
    }
    surface.into_super()
}

pub fn modifier_menu(
    choices: Vec<StringChoice>,
    on_change: impl Fn(String) + 'static,
    mtm: MainThreadMarker,
) -> StringSelector {
    let row = NSView::new(mtm);
    let button = search_menu_button(
        "Add modifier",
        Some("plus"),
        "Search modifiers",
        choices,
        false,
        on_change,
        mtm,
    );
    button.setTranslatesAutoresizingMaskIntoConstraints(false);
    row.addSubview(&button);
    for constraint in [
        button.centerXAnchor().constraintEqualToAnchor(&row.centerXAnchor()),
        button.topAnchor().constraintEqualToAnchor(&row.topAnchor()),
        button.bottomAnchor().constraintEqualToAnchor(&row.bottomAnchor()),
        button.heightAnchor().constraintGreaterThanOrEqualToConstant(CONTROL_HEIGHT),
    ] {
        constraint.setActive(true);
    }
    StringSelector {
        view: row,
    }
}

pub fn live_performance(mtm: MainThreadMarker) -> Retained<NSGlassEffectView> {
    let root = NSGlassEffectView::new(mtm);
    root.setStyle(NSGlassEffectViewStyle::Regular);
    root.setCornerRadius(8.0);
    let vertical = column_stack(0.0, mtm);
    let header = row_stack(4.0, mtm);
    header.setEdgeInsets(objc2_foundation::NSEdgeInsets {
        top: 6.0,
        left: 8.0,
        bottom: 6.0,
        right: 8.0,
    });
    let disclosure = unsafe {
        NSButton::buttonWithImage_target_action(
            &symbol("chevron.right", "Expand Live Performance"),
            None,
            None,
            mtm,
        )
    };
    disclosure.setBordered(false);
    let title = NSTextField::labelWithString(&NSString::from_str("Live Performance"), mtm);
    let clear = unsafe {
        NSButton::buttonWithImage_target_action(&symbol("trash", "Clear"), None, None, mtm)
    };
    clear.setBordered(false);
    clear.setToolTip(Some(&NSString::from_str("Clear")));
    let copy = unsafe {
        NSButton::buttonWithImage_target_action(&symbol("doc.on.doc", "Copy JSON"), None, None, mtm)
    };
    copy.setBordered(false);
    copy.setToolTip(Some(&NSString::from_str("Copy JSON")));
    header.addArrangedSubview(&disclosure);
    header.addArrangedSubview(&title);
    header.addArrangedSubview(&NSView::new(mtm));
    header.addArrangedSubview(&clear);
    header.addArrangedSubview(&copy);
    let rows = column_stack(4.0, mtm);
    rows.setEdgeInsets(objc2_foundation::NSEdgeInsets {
        top: 4.0,
        left: 12.0,
        bottom: 12.0,
        right: 12.0,
    });
    rows.setHidden(true);
    column_append(&vertical, &header);
    column_append(&vertical, &rows);
    root.setContentView(Some(&vertical));

    let performance = Rc::new(RefCell::new(
        shrimply_component_core::performance::PerformanceRows::default(),
    ));
    action::attach(
        &disclosure,
        {
            let rows = rows.clone();
            let performance = performance.clone();
            move |control| {
                let expanded = rows.isHidden();
                rows.setHidden(!expanded);
                control
                    .downcast_ref::<NSButton>()
                    .expect("live performance disclosure sender")
                    .setImage(Some(&symbol(
                        if expanded { "chevron.down" } else { "chevron.right" },
                        if expanded { "Collapse" } else { "Expand" },
                    )));
                if expanded {
                    refresh_performance(&rows, &performance, mtm);
                } else {
                    clear_stack(&rows);
                    *performance.borrow_mut() = Default::default();
                }
            }
        },
        mtm,
    );
    action::attach(
        &clear,
        {
            let rows = rows.clone();
            let performance = performance.clone();
            move |_| {
                shrimply_component_core::performance::clear();
                *performance.borrow_mut() = Default::default();
                if !rows.isHidden() {
                    refresh_performance(&rows, &performance, mtm);
                }
            }
        },
        mtm,
    );
    action::attach(
        &copy,
        move |_| {
            let pasteboard = NSPasteboard::generalPasteboard();
            pasteboard.clearContents();
            assert!(
                pasteboard.setString_forType(
                    &NSString::from_str(&shrimply_component_core::performance::report_json()),
                    unsafe { NSPasteboardTypeString },
                ),
                "could not copy the performance report",
            );
        },
        mtm,
    );
    let weak_root = Weak::new(&*root);
    let weak_rows = Weak::new(&*rows);
    let timer_performance = performance.clone();
    let timer = RcBlock::new(move |timer: std::ptr::NonNull<NSTimer>| {
        let Some(root) = weak_root.load() else {
            unsafe { timer.as_ref() }.invalidate();
            return;
        };
        let Some(rows) = weak_rows.load() else {
            unsafe { timer.as_ref() }.invalidate();
            return;
        };
        if root.window().is_some() && !rows.isHidden() {
            refresh_performance(&rows, &timer_performance, mtm);
        }
    });
    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_repeats_block(
            shrimply_component_core::performance::REFRESH_INTERVAL.as_secs_f64(),
            true,
            &timer,
        );
    }
    root
}

fn refresh_performance(
    rows: &NSStackView,
    performance: &RefCell<shrimply_component_core::performance::PerformanceRows>,
    mtm: MainThreadMarker,
) {
    let entries = {
        let mut performance = performance.borrow_mut();
        if !performance.refresh() {
            return;
        }
        performance.rows().to_vec()
    };
    clear_stack(rows);
    if entries.is_empty() {
        column_append(rows, &NSTextField::labelWithString(
            &NSString::from_str("No performance samples"),
            mtm,
        ));
        return;
    }
    for entry in entries {
        let item = column_stack(1.0, mtm);
        column_append(&item, &NSTextField::labelWithString(
            &NSString::from_str(&entry.title),
            mtm,
        ));
        let subtitle = NSTextField::labelWithString(&NSString::from_str(&entry.subtitle), mtm);
        subtitle.setTextColor(Some(&NSColor::secondaryLabelColor()));
        subtitle.setFont(Some(&NSFont::systemFontOfSize(NSFont::smallSystemFontSize())));
        column_append(&item, &subtitle);
        column_append(rows, &item);
    }
}

fn clear_stack(stack: &NSStackView) {
    let children = stack.arrangedSubviews();
    for child in children.iter() {
        stack.removeArrangedSubview(&child);
        child.removeFromSuperview();
    }
}

fn symbol(name: &str, label: &str) -> Retained<NSImage> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&NSString::from_str(label)),
    )
    .unwrap_or_else(|| panic!("macOS must provide the {name} system symbol"))
}
