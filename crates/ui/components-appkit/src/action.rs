use objc2::ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_setAssociatedObject};
use objc2::rc::Retained;
use objc2::{MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::NSControl;
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use std::cell::RefCell;
use std::ffi::c_void;

static ACTION_TARGET_KEY: u8 = 0;

struct ActionIvars {
    callback: RefCell<Box<dyn FnMut(&NSControl)>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ActionIvars]
    struct ActionTarget;

    unsafe impl NSObjectProtocol for ActionTarget {}

    impl ActionTarget {
        #[unsafe(method(invoke:))]
        fn invoke(&self, sender: &NSControl) {
            (self.ivars().callback.borrow_mut())(sender);
        }
    }
);

pub fn attach(control: &NSControl, callback: impl FnMut(&NSControl) + 'static, mtm: MainThreadMarker) {
    let target = ActionTarget::alloc(mtm).set_ivars(ActionIvars {
        callback: RefCell::new(Box::new(callback)),
    });
    let target: Retained<ActionTarget> = unsafe { msg_send![super(target), init] };
    unsafe {
        control.setTarget(Some(&target));
        control.setAction(Some(sel!(invoke:)));
        objc_setAssociatedObject(
            std::ptr::from_ref(control).cast_mut().cast(),
            std::ptr::from_ref(&ACTION_TARGET_KEY).cast::<c_void>(),
            Retained::as_ptr(&target).cast_mut().cast(),
            OBJC_ASSOCIATION_RETAIN_NONATOMIC,
        );
    }
}

