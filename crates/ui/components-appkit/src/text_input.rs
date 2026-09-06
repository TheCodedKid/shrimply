use objc2::ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_setAssociatedObject};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSControlTextEditingDelegate, NSScrollView, NSTextDelegate, NSTextField, NSTextFieldDelegate,
    NSTextView, NSTextViewDelegate,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSRect, NSSize, NSString,
};
use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;

static SINGLE_TARGET_KEY: u8 = 0;
static MULTILINE_TARGET_KEY: u8 = 0;

struct SingleTargetIvars {
    commit: RefCell<shrimply_component_core::text::TextCommit>,
    max_length: Option<usize>,
    on_change: Rc<dyn Fn(String)>,
    on_commit: Rc<dyn Fn(String)>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SingleTargetIvars]
    struct SingleTarget;

    unsafe impl NSObjectProtocol for SingleTarget {}
    unsafe impl NSTextFieldDelegate for SingleTarget {}
    unsafe impl NSControlTextEditingDelegate for SingleTarget {
        #[unsafe(method(controlTextDidChange:))]
        fn changed(&self, notification: &NSNotification) {
            let field = notification.object().expect("text notification sender")
                .downcast::<NSTextField>().expect("text notification must contain a text field");
            let entered = field.stringValue().to_string();
            let text = shrimply_component_core::text::limited_text(&entered, self.ivars().max_length);
            if text != entered { field.setStringValue(&NSString::from_str(&text)); }
            self.ivars().commit.borrow_mut().changed(text.clone());
            (self.ivars().on_change)(text);
        }

        #[unsafe(method(controlTextDidEndEditing:))]
        fn ended(&self, notification: &NSNotification) {
            let field = notification.object().expect("text notification sender")
                .downcast::<NSTextField>().expect("text notification must contain a text field");
            self.commit(&field);
        }
    }

    impl SingleTarget {
        #[unsafe(method(commit:))]
        fn commit_action(&self, sender: &NSTextField) { self.commit(sender); }
    }
);

impl SingleTarget {
    fn commit(&self, field: &NSTextField) {
        if self.ivars().commit.borrow_mut().take_commit() {
            (self.ivars().on_commit)(field.stringValue().to_string());
        }
    }
}

pub struct SingleLineTextInput {
    view: Retained<NSTextField>,
}

impl SingleLineTextInput {
    pub fn new(
        value: &str,
        placeholder: Option<&str>,
        max_length: Option<usize>,
        on_change: impl Fn(String) + 'static,
        on_commit: impl Fn(String) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let field = NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::ZERO);
        field.setStringValue(&NSString::from_str(
            &shrimply_component_core::text::limited_text(value, max_length),
        ));
        field.setPlaceholderString(placeholder.map(NSString::from_str).as_deref());
        let target = SingleTarget::alloc(mtm).set_ivars(SingleTargetIvars {
            commit: RefCell::new(shrimply_component_core::text::TextCommit::new(value)),
            max_length,
            on_change: Rc::new(on_change),
            on_commit: Rc::new(on_commit),
        });
        let target: Retained<SingleTarget> = unsafe { msg_send![super(target), init] };
        unsafe {
            field.setTarget(Some(&target));
            field.setAction(Some(sel!(commit:)));
            field.setDelegate(Some(ProtocolObject::from_ref(&*target)));
            retain_target(&*field, &target, &SINGLE_TARGET_KEY);
        }
        Self { view: field }
    }

    pub fn view(&self) -> &NSTextField {
        &self.view
    }
}

struct MultilineTargetIvars {
    commit: RefCell<shrimply_component_core::text::TextCommit>,
    syncing: Cell<bool>,
    max_length: Option<usize>,
    on_change: Rc<dyn Fn(String) -> bool>,
    on_commit: Rc<dyn Fn()>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MultilineTargetIvars]
    struct MultilineTarget;

    unsafe impl NSObjectProtocol for MultilineTarget {}
    unsafe impl NSTextDelegate for MultilineTarget {
        #[unsafe(method(textDidChange:))]
        fn changed(&self, notification: &NSNotification) {
            if self.ivars().syncing.get() {
                return;
            }
            let view = notification
                .object()
                .expect("text notification sender")
                .downcast::<NSTextView>()
                .expect("text notification must contain a text view");
            let entered = view.string().to_string();
            let text =
                shrimply_component_core::text::limited_text(&entered, self.ivars().max_length);
            if text != entered {
                self.ivars().syncing.set(true);
                view.setString(&NSString::from_str(&text));
                self.ivars().syncing.set(false);
            }
            let marks = shrimply_component_core::text::typo_marks(&text);
            view.setToolTip(
                (!marks.is_empty())
                    .then(|| {
                        NSString::from_str(
                            &marks
                                .into_iter()
                                .map(|mark| mark.message)
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    })
                    .as_deref(),
            );
            if (self.ivars().on_change)(text.clone()) {
                self.ivars().commit.borrow_mut().changed(text);
            }
        }

        #[unsafe(method(textDidEndEditing:))]
        fn ended(&self, _notification: &NSNotification) {
            self.commit();
        }
    }
    unsafe impl NSTextViewDelegate for MultilineTarget {}
);

impl MultilineTarget {
    fn commit(&self) {
        if self.ivars().commit.borrow_mut().take_commit() {
            (self.ivars().on_commit)();
        }
    }
}

pub struct MultilineTextInput {
    scroll: Retained<NSScrollView>,
    view: Retained<NSTextView>,
}

impl MultilineTextInput {
    pub fn new(
        value: &str,
        min_content_height: f64,
        max_length: Option<usize>,
        on_change: impl Fn(String) -> bool + 'static,
        on_commit: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let view = NSTextView::initWithFrame(
            NSTextView::alloc(mtm),
            NSRect::new(
                objc2_foundation::NSPoint::ZERO,
                NSSize::new(400.0, min_content_height),
            ),
        );
        view.setString(&NSString::from_str(
            &shrimply_component_core::text::limited_text(value, max_length),
        ));
        view.setRichText(false);
        view.setContinuousSpellCheckingEnabled(true);
        let target = MultilineTarget::alloc(mtm).set_ivars(MultilineTargetIvars {
            commit: RefCell::new(shrimply_component_core::text::TextCommit::new(value)),
            syncing: Cell::new(false),
            max_length,
            on_change: Rc::new(on_change),
            on_commit: Rc::new(on_commit),
        });
        let target: Retained<MultilineTarget> = unsafe { msg_send![super(target), init] };
        unsafe {
            view.setDelegate(Some(ProtocolObject::from_ref(&*target)));
            retain_target(&*view, &target, &MULTILINE_TARGET_KEY);
        }
        let scroll = NSScrollView::initWithFrame(
            NSScrollView::alloc(mtm),
            NSRect::new(
                objc2_foundation::NSPoint::ZERO,
                NSSize::new(400.0, min_content_height),
            ),
        );
        scroll.setHasVerticalScroller(true);
        scroll.setBorderType(objc2_app_kit::NSBorderType::BezelBorder);
        scroll.setDocumentView(Some(&view));
        scroll
            .heightAnchor()
            .constraintGreaterThanOrEqualToConstant(min_content_height)
            .setActive(true);
        Self { scroll, view }
    }

    pub fn view(&self) -> &NSScrollView {
        &self.scroll
    }

    pub fn set_text(&self, text: &str) {
        self.view.setString(&NSString::from_str(text));
    }
}

unsafe fn retain_target<T: objc2::Message, U: objc2::Message>(
    object: &T,
    target: &Retained<U>,
    key: &'static u8,
) {
    unsafe {
        objc_setAssociatedObject(
            std::ptr::from_ref(object).cast_mut().cast(),
            std::ptr::from_ref(key).cast::<c_void>(),
            Retained::as_ptr(target).cast_mut().cast(),
            OBJC_ASSOCIATION_RETAIN_NONATOMIC,
        );
    }
}
