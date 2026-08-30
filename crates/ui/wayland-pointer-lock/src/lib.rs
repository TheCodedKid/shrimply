use std::ffi::c_void;
use std::time::{Duration, Instant};

use wayland_client::backend::{Backend, ObjectId};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_pointer, wl_registry, wl_seat, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols::wp::pointer_constraints::zv1::client::{
    zwp_locked_pointer_v1, zwp_pointer_constraints_v1,
};
use wayland_protocols::wp::relative_pointer::zv1::client::{
    zwp_relative_pointer_manager_v1, zwp_relative_pointer_v1,
};

const SLOW_POINTER_LOCK_LOG_THRESHOLD: Duration = Duration::from_millis(16);

pub struct WaylandPointerLock {
    event_queue: EventQueue<PointerLockState>,
    state: PointerLockState,
    locked_pointer: zwp_locked_pointer_v1::ZwpLockedPointerV1,
    relative_pointer: zwp_relative_pointer_v1::ZwpRelativePointerV1,
    pointer: wl_pointer::WlPointer,
    pointer_constraints: zwp_pointer_constraints_v1::ZwpPointerConstraintsV1,
    relative_manager: zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1,
    surface: wl_surface::WlSurface,
    _seat: wl_seat::WlSeat,
}

impl WaylandPointerLock {
    /// Creates a pointer lock around native Wayland objects.
    ///
    /// # Safety
    ///
    /// The pointers must belong to the same live Wayland connection and remain valid
    /// until the returned lock is dropped.
    pub unsafe fn new(
        wl_display: *mut c_void,
        wl_surface: *mut c_void,
        wl_seat: *mut c_void,
    ) -> Option<Self> {
        if wl_display.is_null() || wl_surface.is_null() || wl_seat.is_null() {
            return None;
        }
        let backend = unsafe { Backend::from_foreign_display(wl_display.cast()) };
        let conn = Connection::from_backend(backend);
        let (globals, event_queue) = registry_queue_init::<PointerLockState>(&conn).ok()?;
        let qh = event_queue.handle();
        let pointer_constraints: zwp_pointer_constraints_v1::ZwpPointerConstraintsV1 =
            globals.bind(&qh, 1..=1, ()).ok()?;
        let relative_manager: zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1 =
            globals.bind(&qh, 1..=1, ()).ok()?;
        let surface_id =
            unsafe { ObjectId::from_ptr(wl_surface::WlSurface::interface(), wl_surface.cast()) }
                .ok()?;
        let seat_id =
            unsafe { ObjectId::from_ptr(wl_seat::WlSeat::interface(), wl_seat.cast()) }.ok()?;
        let surface = wl_surface::WlSurface::from_id(&conn, surface_id).ok()?;
        let seat = wl_seat::WlSeat::from_id(&conn, seat_id).ok()?;
        let pointer = seat.get_pointer(&qh, ());
        let relative_pointer = relative_manager.get_relative_pointer(&pointer, &qh, ());
        let locked_pointer = pointer_constraints.lock_pointer(
            &surface,
            &pointer,
            None,
            zwp_pointer_constraints_v1::Lifetime::Persistent,
            &qh,
            (),
        );
        let mut lock = Self {
            event_queue,
            state: PointerLockState::default(),
            locked_pointer,
            relative_pointer,
            pointer,
            pointer_constraints,
            relative_manager,
            surface,
            _seat: seat,
        };
        lock.event_queue.roundtrip(&mut lock.state).ok()?;
        lock.state.take_delta();
        lock.state.lock_origin = Some(lock.state.surface_position?);
        Some(lock)
    }

    pub fn poll(&mut self) -> Option<(f64, f64)> {
        self.state.begin_tick();
        let dispatch_started = Instant::now();
        let _ = self.event_queue.dispatch_pending(&mut self.state);
        let dispatch_elapsed = dispatch_started.elapsed();
        let flush_started = Instant::now();
        let _ = self.event_queue.flush();
        let flush_elapsed = flush_started.elapsed();
        self.state.log_tick(dispatch_elapsed, flush_elapsed);
        self.state.take_delta()
    }

    pub fn restore_cursor_at(&mut self, x: f64, y: f64) {
        self.locked_pointer.set_cursor_position_hint(x, y);
        self.surface.commit();
        let _ = self.event_queue.flush();
    }

    pub fn restore_cursor_with_offset(&mut self, x: f64, y: f64) {
        let (origin_x, origin_y) = self
            .state
            .lock_origin
            .expect("locked Wayland pointer must have a surface position");
        self.restore_cursor_at(origin_x + x, origin_y + y);
    }
}

impl Drop for WaylandPointerLock {
    fn drop(&mut self) {
        self.locked_pointer.destroy();
        self.relative_pointer.destroy();
        self.pointer.release();
        self.pointer_constraints.destroy();
        self.relative_manager.destroy();
        let _ = self.event_queue.flush();
    }
}

#[derive(Default)]
struct PointerLockState {
    surface_position: Option<(f64, f64)>,
    lock_origin: Option<(f64, f64)>,
    tick_motion_count: usize,
    tick_delta_x: f64,
    tick_delta_y: f64,
}

impl PointerLockState {
    fn begin_tick(&mut self) {
        self.tick_motion_count = 0;
        self.tick_delta_x = 0.0;
        self.tick_delta_y = 0.0;
    }

    fn log_tick(&self, dispatch_elapsed: Duration, flush_elapsed: Duration) {
        if dispatch_elapsed < SLOW_POINTER_LOCK_LOG_THRESHOLD
            && flush_elapsed < SLOW_POINTER_LOCK_LOG_THRESHOLD
        {
            return;
        }
        tracing::debug!(
            "pointer_lock: dispatch_tick motions={} delta=({:.3}, {:.3}) dispatch_elapsed_us={} flush_elapsed_us={}",
            self.tick_motion_count,
            self.tick_delta_x,
            self.tick_delta_y,
            dispatch_elapsed.as_micros(),
            flush_elapsed.as_micros(),
        );
    }

    fn take_delta(&mut self) -> Option<(f64, f64)> {
        if self.tick_delta_x == 0.0 && self.tick_delta_y == 0.0 {
            return None;
        }
        let delta = (self.tick_delta_x, self.tick_delta_y);
        self.tick_delta_x = 0.0;
        self.tick_delta_y = 0.0;
        Some(delta)
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for PointerLockState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for PointerLockState {
    fn event(
        state: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            }
            | wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => state.surface_position = Some((surface_x, surface_y)),
            _ => {}
        }
    }
}

impl Dispatch<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, ()> for PointerLockState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_pointer_constraints_v1::ZwpPointerConstraintsV1,
        _event: zwp_pointer_constraints_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_locked_pointer_v1::ZwpLockedPointerV1, ()> for PointerLockState {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_locked_pointer_v1::ZwpLockedPointerV1,
        _event: zwp_locked_pointer_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1, ()>
    for PointerLockState
{
    fn event(
        _state: &mut Self,
        _proxy: &zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1,
        _event: zwp_relative_pointer_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_relative_pointer_v1::ZwpRelativePointerV1, ()> for PointerLockState {
    fn event(
        state: &mut Self,
        _proxy: &zwp_relative_pointer_v1::ZwpRelativePointerV1,
        event: zwp_relative_pointer_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zwp_relative_pointer_v1::Event::RelativeMotion { dx, dy, .. } = event {
            state.tick_motion_count += 1;
            state.tick_delta_x += dx;
            state.tick_delta_y += dy;
        }
    }
}
