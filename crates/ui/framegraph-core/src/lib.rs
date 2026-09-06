use std::{cell::RefCell, rc::Rc};

use shrimply_interpolation::Interpolation;
pub use shrimply_keyframe_graph_core::{
    FrameGraphAction, FrameGraphComponentAction, FrameGraphComponents, FrameGraphKey,
    FrameGraphModifiers, FrameGraphPointerButton, FrameGraphPointerPosition, FrameGraphScrollInput,
    FrameGraphState, FrameGraphStatus,
};
use shrimply_keyframe_graph_core::{GraphDomain, KeyframeGraph};
use shrimply_math_core::Time;
use shrimply_skia_adw_core::canvas::TimelinePainter;
use uuid::Uuid;

/// Toolkit-independent frame-graph controller. Native bridges translate events and draw the
/// returned state; they cannot reach into selection, hit-test, drag, or view state.
#[derive(Clone)]
pub struct FrameGraphCore(Rc<RefCell<FrameGraphComponents>>);

pub type SharedFrameGraphState = FrameGraphCore;

#[derive(Default)]
pub struct FrameGraphInputResult {
    pub actions: Vec<FrameGraphComponentAction>,
    pub focus: bool,
    pub handled: bool,
    pub redraw: bool,
}

#[derive(Clone, Copy)]
pub enum FrameGraphCommand {
    PreviousKey,
    ToggleKey,
    NextKey,
}

impl FrameGraphCore {
    pub fn new(components: FrameGraphComponents) -> Self {
        Self(Rc::new(RefCell::new(components)))
    }

    pub fn single(state: FrameGraphState) -> Self {
        Self::new(FrameGraphComponents::single(state))
    }

    pub fn active_component(&self) -> usize {
        self.0.borrow().active_component()
    }

    pub fn preferred_height(&self) -> i32 {
        self.0.borrow().preferred_height()
    }

    pub fn status(&self) -> FrameGraphStatus {
        self.0.borrow().status()
    }

    pub fn is_animating(&self) -> bool {
        self.0.borrow().is_animating()
    }

    pub fn draw(&self, painter: &TimelinePainter, width: f64, height: f64) {
        self.0.borrow_mut().draw(painter, width, height);
    }

    pub fn edit_value(&self, value: f64) -> Vec<FrameGraphComponentAction> {
        self.0
            .borrow_mut()
            .active_actions(|state| state.set_value(value))
    }

    pub fn edit_component_values(
        &self,
        active_component: usize,
        values: &[(usize, f64)],
    ) -> Vec<FrameGraphComponentAction> {
        self.0
            .borrow_mut()
            .set_component_values(active_component, values)
    }

    pub fn activate_component(&self, component: usize) {
        self.0.borrow_mut().activate(component);
    }

    pub fn replace_components(&self, components: FrameGraphComponents) {
        *self.0.borrow_mut() = components;
    }

    pub fn replace_active_graph(&self, graph: KeyframeGraph) {
        let component = self.active_component();
        self.0
            .borrow_mut()
            .replace_component_graph(component, graph);
    }

    pub fn set_view(
        &self,
        item_range: GraphDomain,
        frame_step: Time,
        playhead: Time,
        snapping: (bool, f64),
        external_clipboard: bool,
        text_interpolation: bool,
    ) {
        let mut state = self.0.borrow_mut();
        state.set_item_range(item_range);
        state.set_frame_step(frame_step);
        state.set_playhead(playhead);
        state.set_snapping(snapping.0, snapping.1);
        state.set_external_clipboard(external_clipboard);
        state.set_text_interpolation(text_interpolation);
    }

    pub fn set_playhead(&self, playhead: Time) {
        self.0.borrow_mut().set_playhead(playhead);
    }

    pub fn set_interpolation(&self, owner_id: Uuid, interpolation: Interpolation) {
        self.0
            .borrow_mut()
            .set_interpolation(owner_id, interpolation);
    }

    pub fn command(&self, command: FrameGraphCommand) -> FrameGraphInputResult {
        let actions = self.0.borrow_mut().active_actions(|state| match command {
            FrameGraphCommand::PreviousKey => state.previous_key(),
            FrameGraphCommand::ToggleKey => state.toggle_key(),
            FrameGraphCommand::NextKey => state.next_key(),
        });
        FrameGraphInputResult {
            actions,
            focus: true,
            handled: true,
            redraw: true,
        }
    }

    pub fn pointer_moved(&self, x: f64, y: f64) -> FrameGraphInputResult {
        self.0.borrow_mut().pointer_moved(x, y);
        FrameGraphInputResult {
            redraw: true,
            ..Default::default()
        }
    }

    pub fn pointer_left(&self) -> FrameGraphInputResult {
        self.0.borrow_mut().pointer_left();
        FrameGraphInputResult {
            redraw: true,
            ..Default::default()
        }
    }

    pub fn begin_pointer(
        &self,
        button: FrameGraphPointerButton,
        position: FrameGraphPointerPosition,
        modifiers: FrameGraphModifiers,
    ) -> FrameGraphInputResult {
        let actions = self.0.borrow_mut().active_actions(|state| {
            state.begin_pointer(
                button,
                position.x,
                position.y,
                position.width,
                position.height,
                modifiers,
            )
        });
        FrameGraphInputResult {
            actions,
            focus: true,
            handled: true,
            redraw: true,
        }
    }

    pub fn update_pointer(&self, position: FrameGraphPointerPosition) -> FrameGraphInputResult {
        FrameGraphInputResult {
            actions: self.0.borrow_mut().active_actions(|state| {
                state.update_pointer(position.x, position.y, position.width, position.height)
            }),
            handled: true,
            redraw: true,
            ..Default::default()
        }
    }

    pub fn end_pointer(&self) -> FrameGraphInputResult {
        FrameGraphInputResult {
            actions: self
                .0
                .borrow_mut()
                .active_actions(FrameGraphState::end_pointer),
            handled: true,
            redraw: true,
            ..Default::default()
        }
    }

    pub fn scroll(
        &self,
        dx: f64,
        dy: f64,
        position: FrameGraphPointerPosition,
        zoom: bool,
        input: FrameGraphScrollInput,
    ) -> FrameGraphInputResult {
        let handled = self.0.borrow_mut().scroll(dx, dy, position, zoom, input);
        FrameGraphInputResult {
            handled,
            redraw: handled,
            ..Default::default()
        }
    }

    pub fn magnify(
        &self,
        magnification: f64,
        position: FrameGraphPointerPosition,
    ) -> FrameGraphInputResult {
        self.0
            .borrow_mut()
            .magnify(magnification, position.x, position.width);
        FrameGraphInputResult {
            handled: true,
            redraw: true,
            ..Default::default()
        }
    }

    pub fn key(&self, key: FrameGraphKey) -> FrameGraphInputResult {
        FrameGraphInputResult {
            actions: self.0.borrow_mut().active_actions(|state| state.key(key)),
            handled: true,
            redraw: true,
            ..Default::default()
        }
    }
}
