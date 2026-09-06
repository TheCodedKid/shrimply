#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSTrackingArea, NSTrackingAreaOptions, NSView};
use objc2_foundation::{
    MainThreadMarker, NSObjectProtocol, NSRect, NSRunLoop, NSRunLoopCommonModes, NSSize, NSTimer,
};
use shrimply_framegraph_core::{
    FrameGraphComponentAction, FrameGraphInputResult, FrameGraphKey, FrameGraphModifiers,
    FrameGraphPointerButton, FrameGraphPointerPosition, FrameGraphScrollInput,
    SharedFrameGraphState,
};
use shrimply_skia_adw_core::canvas::TimelinePainter;
use shrimply_skia_metal::Renderer;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub type FrameGraphActionHandler = Rc<dyn Fn(FrameGraphComponentAction)>;
const FRAME_INTERVAL_SECONDS: f64 = 1.0 / 60.0;

pub struct GraphViewIvars {
    renderer: RefCell<Renderer>,
    state: SharedFrameGraphState,
    on_action: FrameGraphActionHandler,
    tracking_area: RefCell<Option<Retained<NSTrackingArea>>>,
    pointer: Cell<Option<(f64, f64)>>,
    animation_timer: RefCell<Option<Retained<NSTimer>>>,
}

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = GraphViewIvars]
    pub struct FrameGraphView;

    unsafe impl NSObjectProtocol for FrameGraphView {}

    impl FrameGraphView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool { true }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool { true }

        #[unsafe(method(updateTrackingAreas))]
        fn update_tracking_areas(&self) {
            unsafe { let _: () = msg_send![super(self), updateTrackingAreas]; }
            if let Some(previous) = self.ivars().tracking_area.borrow_mut().take() {
                self.removeTrackingArea(&previous);
            }
            let options = NSTrackingAreaOptions::MouseEnteredAndExited
                | NSTrackingAreaOptions::MouseMoved
                | NSTrackingAreaOptions::ActiveInKeyWindow
                | NSTrackingAreaOptions::InVisibleRect;
            let area = unsafe {
                NSTrackingArea::initWithRect_options_owner_userInfo(
                    NSTrackingArea::alloc(),
                    NSRect::ZERO,
                    options,
                    Some(self),
                    None,
                )
            };
            self.addTrackingArea(&area);
            self.ivars().tracking_area.replace(Some(area));
        }

        #[unsafe(method(layout))]
        fn layout(&self) {
            unsafe { let _: () = msg_send![super(self), layout]; }
            self.render();
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, event: &NSEvent) { self.pointer_moved(event); }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) { self.pointer_moved(event); }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            self.ivars().pointer.set(None);
            self.finish(self.ivars().state.pointer_left());
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.begin_pointer(event, FrameGraphPointerButton::Primary);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) { self.update_pointer(event); }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, _event: &NSEvent) { self.end_pointer(); }

        #[unsafe(method(otherMouseDown:))]
        fn other_mouse_down(&self, event: &NSEvent) {
            if event.buttonNumber() == 2 {
                self.begin_pointer(event, FrameGraphPointerButton::Middle);
            }
        }

        #[unsafe(method(otherMouseDragged:))]
        fn other_mouse_dragged(&self, event: &NSEvent) {
            if event.buttonNumber() == 2 { self.update_pointer(event); }
        }

        #[unsafe(method(otherMouseUp:))]
        fn other_mouse_up(&self, event: &NSEvent) {
            if event.buttonNumber() == 2 { self.end_pointer(); }
        }

        #[unsafe(method(rightMouseDown:))]
        fn right_mouse_down(&self, event: &NSEvent) {
            self.begin_pointer(event, FrameGraphPointerButton::Secondary);
            self.end_pointer();
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            let (x, y) = self.point(event);
            let size = self.bounds().size;
            let result = self.ivars().state.scroll(
                -event.scrollingDeltaX(),
                -event.scrollingDeltaY(),
                FrameGraphPointerPosition {
                    x,
                    y,
                    width: size.width.max(1.0),
                    height: size.height.max(1.0),
                },
                event.modifierFlags().contains(NSEventModifierFlags::Control),
                if event.hasPreciseScrollingDeltas() {
                    FrameGraphScrollInput::Surface
                } else {
                    FrameGraphScrollInput::Wheel
                },
            );
            let handled = result.handled;
            self.finish(result);
            if !handled {
                unsafe { let _: () = msg_send![super(self), scrollWheel: event]; }
            }
        }

        #[unsafe(method(magnifyWithEvent:))]
        fn magnify(&self, event: &NSEvent) {
            let (x, y) = self.point(event);
            let size = self.bounds().size;
            self.finish(self.ivars().state.magnify(
                event.magnification(),
                FrameGraphPointerPosition {
                    x,
                    y,
                    width: size.width.max(1.0),
                    height: size.height.max(1.0),
                },
            ));
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            let command = event.modifierFlags().contains(NSEventModifierFlags::Command);
            let key = match event.keyCode() {
                49 => Some(FrameGraphKey::TogglePlayback),
                123 => Some(FrameGraphKey::PreviousFrame),
                124 => Some(FrameGraphKey::NextFrame),
                115 => Some(FrameGraphKey::Start),
                119 => Some(FrameGraphKey::End),
                51 | 117 => Some(FrameGraphKey::Delete),
                8 if command => Some(FrameGraphKey::Copy),
                9 if command => Some(FrameGraphKey::Paste),
                24 => Some(FrameGraphKey::ZoomIn),
                27 => Some(FrameGraphKey::ZoomOut),
                _ => None,
            };
            let Some(key) = key else {
                unsafe { let _: () = msg_send![super(self), keyDown: event]; }
                return;
            };
            self.finish(self.ivars().state.key(key));
        }

        #[unsafe(method(cancelOperation:))]
        fn cancel_operation(&self, _sender: &objc2_foundation::NSObject) {
            self.end_pointer();
        }

        #[unsafe(method(viewDidMoveToWindow))]
        fn moved_to_window(&self) {
            unsafe { let _: () = msg_send![super(self), viewDidMoveToWindow]; }
            if self.window().is_none()
                && let Some(timer) = self.ivars().animation_timer.borrow_mut().take()
            {
                timer.invalidate();
            }
        }

        #[unsafe(method(renderAnimation:))]
        fn render_animation(&self, timer: &NSTimer) {
            if self.window().is_none() || !self.ivars().state.is_animating() {
                timer.invalidate();
                self.ivars().animation_timer.borrow_mut().take();
                return;
            }
            self.render();
        }
    }
);

impl FrameGraphView {
    fn point(&self, event: &NSEvent) -> (f64, f64) {
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        (point.x, point.y)
    }

    fn modifiers(event: &NSEvent) -> FrameGraphModifiers {
        let modifiers = event.modifierFlags();
        FrameGraphModifiers {
            control: modifiers.contains(NSEventModifierFlags::Command),
            shift: modifiers.contains(NSEventModifierFlags::Shift),
        }
    }

    fn pointer_moved(&self, event: &NSEvent) {
        let point = self.point(event);
        self.ivars().pointer.set(Some(point));
        self.finish(self.ivars().state.pointer_moved(point.0, point.1));
    }

    fn begin_pointer(&self, event: &NSEvent, button: FrameGraphPointerButton) {
        let (x, y) = self.point(event);
        let size = self.bounds().size;
        self.finish(self.ivars().state.begin_pointer(
            button,
            FrameGraphPointerPosition {
                x,
                y,
                width: size.width.max(1.0),
                height: size.height.max(1.0),
            },
            Self::modifiers(event),
        ));
    }

    fn update_pointer(&self, event: &NSEvent) {
        let (x, y) = self.point(event);
        let size = self.bounds().size;
        self.finish(
            self.ivars()
                .state
                .update_pointer(FrameGraphPointerPosition {
                    x,
                    y,
                    width: size.width.max(1.0),
                    height: size.height.max(1.0),
                }),
        );
    }

    fn end_pointer(&self) {
        self.finish(self.ivars().state.end_pointer());
    }

    fn finish(&self, result: FrameGraphInputResult) {
        if result.focus {
            self.window()
                .expect("frame graph must be attached before input")
                .makeFirstResponder(Some(self));
        }
        for action in result.actions {
            (self.ivars().on_action)(action);
        }
        if result.redraw {
            self.render();
        }
        self.start_animation_if_needed();
    }

    fn start_animation_if_needed(&self) {
        if !self.ivars().state.is_animating() || self.ivars().animation_timer.borrow().is_some() {
            return;
        }
        let timer = unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                FRAME_INTERVAL_SECONDS,
                self,
                sel!(renderAnimation:),
                None,
                true,
            )
        };
        unsafe {
            NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes);
        }
        self.ivars().animation_timer.replace(Some(timer));
    }

    pub fn render(&self) {
        let size = self.bounds().size;
        if self.window().is_none() || size.width <= 0.0 || size.height <= 0.0 {
            return;
        }
        let scale = self
            .window()
            .expect("frame graph attached")
            .backingScaleFactor();
        let mut renderer = self.ivars().renderer.borrow_mut();
        renderer.layer().setContentsScale(scale);
        renderer.layer().setDrawableSize(NSSize::new(
            (size.width * scale).ceil(),
            (size.height * scale).ceil(),
        ));
        renderer.draw(|canvas| {
            canvas.clear(shrimply_cross_ui_theme::current().view_bg);
            canvas.scale((scale as f32, scale as f32));
            let painter = TimelinePainter::new(canvas);
            self.ivars().state.draw(&painter, size.width, size.height);
        });
    }
}

pub fn frame_graph_view(
    state: SharedFrameGraphState,
    on_action: FrameGraphActionHandler,
    mtm: MainThreadMarker,
) -> Retained<FrameGraphView> {
    let view = FrameGraphView::alloc(mtm).set_ivars(GraphViewIvars {
        renderer: RefCell::new(Renderer::default()),
        state,
        on_action,
        tracking_area: RefCell::new(None),
        pointer: Cell::new(None),
        animation_timer: RefCell::new(None),
    });
    let view: Retained<FrameGraphView> =
        unsafe { msg_send![super(view), initWithFrame: NSRect::ZERO] };
    view.setLayer(Some(view.ivars().renderer.borrow().layer()));
    view.setWantsLayer(true);
    view
}
