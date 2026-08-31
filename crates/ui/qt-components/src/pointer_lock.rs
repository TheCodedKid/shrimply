use shrimply_wayland_pointer_lock::WaylandPointerLock;
use std::{cell::RefCell, ffi::c_void};

thread_local! {
    static POINTER_LOCK: RefCell<Option<WaylandPointerLock>> = const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_number_begin_pointer_lock(
    display: *mut c_void,
    surface: *mut c_void,
    seat: *mut c_void,
) -> bool {
    POINTER_LOCK.with_borrow_mut(|current| {
        *current = unsafe { WaylandPointerLock::new(display, surface, seat) };
        current.is_some()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_number_poll_pointer_lock(
    delta_x: *mut f64,
    delta_y: *mut f64,
) -> bool {
    if delta_x.is_null() || delta_y.is_null() {
        return false;
    }
    POINTER_LOCK.with_borrow_mut(|current| {
        let Some((x, y)) = current.as_mut().and_then(WaylandPointerLock::poll) else {
            return false;
        };
        unsafe {
            *delta_x = x;
            *delta_y = y;
        }
        true
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_number_end_pointer_lock() {
    POINTER_LOCK.with_borrow_mut(Option::take);
}
