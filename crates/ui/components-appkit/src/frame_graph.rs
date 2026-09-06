use crate::{action, stack};
use objc2::ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_setAssociatedObject};
use objc2::rc::{Retained, Weak};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSButton, NSImage, NSLayoutConstraint, NSMenu, NSMenuItem, NSStackView};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSPoint, NSString};
use shrimply_interpolation::Interpolation;
use shrimply_keyframe_graph_core::{
    FrameGraphAction, FrameGraphComponentAction, FrameGraphComponents, FrameGraphState,
    FrameGraphStatus,
};
use shrimply_math_core::Time;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

pub use shrimply_component_metal::SharedFrameGraphState;

type ActionHandler = Rc<dyn Fn(FrameGraphComponentAction)>;
type StatusHandler = Rc<dyn Fn(FrameGraphStatus)>;

static MENU_TARGET_KEY: u8 = 0;

struct MenuTargetIvars {
    callback: RefCell<Box<dyn FnMut()>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MenuTargetIvars]
    struct MenuTarget;

    unsafe impl NSObjectProtocol for MenuTarget {}

    impl MenuTarget {
        #[unsafe(method(invoke:))]
        fn invoke(&self, _sender: &NSMenuItem) { (self.ivars().callback.borrow_mut())(); }
    }
);

#[derive(Clone)]
pub struct FrameGraph {
    root: Retained<NSStackView>,
    view: Retained<shrimply_component_metal::FrameGraphView>,
    state: SharedFrameGraphState,
    on_action: ActionHandler,
    sync: Rc<dyn Fn()>,
    status_handlers: Rc<RefCell<Vec<StatusHandler>>>,
    height: Retained<NSLayoutConstraint>,
}

impl FrameGraph {
    pub fn new(state: FrameGraphState, mtm: MainThreadMarker) -> Self {
        Self::with_actions(state, |_| {}, mtm)
    }

    pub fn with_actions(
        state: FrameGraphState,
        on_action: impl Fn(FrameGraphAction) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        Self::with_components(
            FrameGraphComponents::single(state),
            move |action| on_action(action.action),
            mtm,
        )
    }

    pub fn with_components(
        state: FrameGraphComponents,
        on_action: impl Fn(FrameGraphComponentAction) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        Self::with_shared_components(Rc::new(RefCell::new(state)), on_action, mtm)
    }

    pub fn with_shared_components(
        state: SharedFrameGraphState,
        on_action: impl Fn(FrameGraphComponentAction) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let on_action: ActionHandler = Rc::new(on_action);
        let status_handlers = Rc::new(RefCell::new(Vec::<StatusHandler>::new()));
        let root = stack(true, 0.0, mtm);
        let controls = stack(false, 6.0, mtm);
        let spacer = objc2_app_kit::NSView::new(mtm);
        controls.addArrangedSubview(&spacer);
        let previous = graph_button("backward.end", "Previous keyframe", mtm);
        let toggle = graph_button("plus", "Add keyframe at playhead", mtm);
        let next = graph_button("forward.end", "Next keyframe", mtm);
        controls.addArrangedSubview(&previous);
        controls.addArrangedSubview(&toggle);
        controls.addArrangedSubview(&next);
        root.addArrangedSubview(&controls);

        let view_slot = Rc::new(RefCell::new(
            None::<Weak<shrimply_component_metal::FrameGraphView>>,
        ));
        let action_state = state.clone();
        let action_handler = on_action.clone();
        let action_view = view_slot.clone();
        let view = shrimply_component_metal::frame_graph_view(
            state.clone(),
            Rc::new(move |component_action| {
                if let FrameGraphAction::InterpolationRequested {
                    owner_id,
                    interpolation,
                    x,
                    y,
                } = component_action.action
                {
                    if let Some(view) = action_view.borrow().as_ref().and_then(Weak::load) {
                        show_interpolation_menu(
                            &view,
                            action_state.clone(),
                            action_handler.clone(),
                            component_action.component,
                            owner_id,
                            interpolation,
                            NSPoint::new(x, y),
                            mtm,
                        );
                    }
                } else {
                    action_handler(component_action);
                }
            }),
            mtm,
        );
        view_slot.replace(Some(Weak::new(&view)));
        let height = view
            .heightAnchor()
            .constraintEqualToConstant(f64::from(state.borrow().preferred_height()));
        height.setActive(true);
        root.addArrangedSubview(&view);

        let sync = {
            let state = state.clone();
            let previous = previous.clone();
            let toggle = toggle.clone();
            let next = next.clone();
            let handlers = status_handlers.clone();
            Rc::new(move || {
                let status = state.borrow().status();
                previous.setEnabled(status.can_previous);
                next.setEnabled(status.can_next);
                toggle.setImage(Some(&symbol(
                    if status.key_at_playhead {
                        "minus"
                    } else {
                        "plus"
                    },
                    if status.key_at_playhead {
                        "Delete keyframe"
                    } else {
                        "Add keyframe"
                    },
                )));
                toggle.setToolTip(Some(&NSString::from_str(if status.key_at_playhead {
                    "Delete keyframe at playhead"
                } else {
                    "Add keyframe at playhead"
                })));
                for handler in handlers.borrow().iter() {
                    handler(status);
                }
            }) as Rc<dyn Fn()>
        };

        attach_graph_button(
            &previous,
            &view,
            &state,
            &on_action,
            &sync,
            |state| state.previous_key(),
            mtm,
        );
        attach_graph_button(
            &toggle,
            &view,
            &state,
            &on_action,
            &sync,
            |state| state.toggle_key(),
            mtm,
        );
        attach_graph_button(
            &next,
            &view,
            &state,
            &on_action,
            &sync,
            |state| state.next_key(),
            mtm,
        );
        sync();
        Self {
            root,
            view,
            state,
            on_action,
            sync,
            status_handlers,
            height,
        }
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }

    pub(crate) fn retained_view(&self) -> Retained<NSStackView> {
        self.root.clone()
    }
    pub fn graph_view(&self) -> &shrimply_component_metal::FrameGraphView {
        &self.view
    }
    pub fn state(&self) -> SharedFrameGraphState {
        self.state.clone()
    }

    pub fn edit_value(&self, value: f64) {
        let actions = self
            .state
            .borrow_mut()
            .active_actions(|state| state.set_value(value));
        self.dispatch(actions);
    }

    pub fn edit_component_values(&self, active_component: usize, values: &[(usize, f64)]) {
        let actions = self
            .state
            .borrow_mut()
            .set_component_values(active_component, values);
        self.dispatch(actions);
    }

    pub fn activate_component(&self, component: usize) {
        self.state.borrow_mut().activate(component);
        self.refresh();
    }

    pub fn set_playhead(&self, playhead: Time) {
        self.state.borrow_mut().set_playhead(playhead);
        self.refresh();
    }

    pub fn replace_state(&self, state: FrameGraphState) {
        self.replace_components(FrameGraphComponents::single(state));
    }

    pub fn replace_components(&self, states: FrameGraphComponents) {
        *self.state.borrow_mut() = states;
        self.refresh();
    }

    pub fn refresh(&self) {
        self.height
            .setConstant(f64::from(self.state.borrow().preferred_height()));
        (self.sync)();
        self.view.render();
    }

    pub fn connect_status(&self, handler: impl Fn(FrameGraphStatus) + 'static) {
        self.status_handlers.borrow_mut().push(Rc::new(handler));
    }

    fn dispatch(&self, actions: Vec<FrameGraphComponentAction>) {
        for action in actions {
            (self.on_action)(action);
        }
        self.refresh();
    }
}

fn graph_button(icon: &str, tooltip: &str, mtm: MainThreadMarker) -> Retained<NSButton> {
    let button =
        unsafe { NSButton::buttonWithImage_target_action(&symbol(icon, tooltip), None, None, mtm) };
    button.setBordered(false);
    button.setToolTip(Some(&NSString::from_str(tooltip)));
    button
}

fn symbol(name: &str, label: &str) -> Retained<NSImage> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&NSString::from_str(label)),
    )
    .unwrap_or_else(|| panic!("macOS must provide the {name} system symbol"))
}

fn attach_graph_button(
    button: &NSButton,
    view: &shrimply_component_metal::FrameGraphView,
    state: &SharedFrameGraphState,
    handler: &ActionHandler,
    sync: &Rc<dyn Fn()>,
    action_fn: impl Fn(&mut FrameGraphState) -> Vec<FrameGraphAction> + 'static,
    mtm: MainThreadMarker,
) {
    let view = Weak::new(view);
    let state = state.clone();
    let handler = handler.clone();
    let sync = sync.clone();
    action::attach(
        button,
        move |_| {
            let actions = state.borrow_mut().active_actions(|state| action_fn(state));
            for action in actions {
                handler(action);
            }
            sync();
            if let Some(view) = view.load() {
                view.window()
                    .expect("frame graph attached")
                    .makeFirstResponder(Some(&view));
                view.render();
            }
        },
        mtm,
    );
}

#[allow(clippy::too_many_arguments)]
fn show_interpolation_menu(
    view: &shrimply_component_metal::FrameGraphView,
    state: SharedFrameGraphState,
    handler: ActionHandler,
    component: usize,
    owner_id: uuid::Uuid,
    selected: Interpolation,
    point: NSPoint,
    mtm: MainThreadMarker,
) {
    let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str("Interpolation"));
    for interpolation in Interpolation::KEYFRAME {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(interpolation.label()),
                Some(sel!(invoke:)),
                &NSString::new(),
            )
        };
        item.setState(if interpolation == selected { 1 } else { 0 });
        let state = state.clone();
        let handler = handler.clone();
        let view = Weak::new(view);
        let target = MenuTarget::alloc(mtm).set_ivars(MenuTargetIvars {
            callback: RefCell::new(Box::new(move || {
                state
                    .borrow_mut()
                    .set_interpolation(owner_id, interpolation);
                handler(FrameGraphComponentAction {
                    component,
                    action: FrameGraphAction::InterpolationRequested {
                        owner_id,
                        interpolation,
                        x: point.x,
                        y: point.y,
                    },
                });
                if let Some(view) = view.load() {
                    view.render();
                }
            })),
        });
        let target: Retained<MenuTarget> = unsafe { msg_send![super(target), init] };
        unsafe {
            item.setTarget(Some(&target));
            objc_setAssociatedObject(
                std::ptr::from_ref(&*item).cast_mut().cast(),
                std::ptr::from_ref(&MENU_TARGET_KEY).cast::<c_void>(),
                Retained::as_ptr(&target).cast_mut().cast(),
                OBJC_ASSOCIATION_RETAIN_NONATOMIC,
            );
        }
        menu.addItem(&item);
    }
    menu.popUpMenuPositioningItem_atLocation_inView(None, point, Some(view));
}
