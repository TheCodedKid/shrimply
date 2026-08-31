use core::ffi::c_void;
use core::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QPoint, QString};
use shrimply_interpolation::Interpolation;
use shrimply_keyframe_graph_ui::{
    FrameGraphAction, FrameGraphKey, FrameGraphModifiers, FrameGraphPointerButton,
    FrameGraphPointerPosition, FrameGraphScrollInput, FrameGraphState,
};
use shrimply_math_color::Color;
use shrimply_skia_adw_ui::canvas::{TimelineRenderer, UVec2};
use uuid::Uuid;

type SharedGraph = Arc<Mutex<GraphModel>>;

const QT_WHEEL_ANGLE_UNITS_PER_STEP: f64 = 120.0;

struct GraphModel {
    state: FrameGraphState,
    context_owner: Option<Uuid>,
}

struct QtFrameGraphRenderer {
    graph: SharedGraph,
    renderer: TimelineRenderer,
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!("frame_graph.h");
        #[qobject]
        #[namespace = "shrimply"]
        type FrameGraphItemBase;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qpoint.h");
        type QPoint = cxx_qt_lib::QPoint;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = FrameGraphItemBase]
        #[qproperty(bool, can_previous, cxx_name = "canPrevious")]
        #[qproperty(bool, can_next, cxx_name = "canNext")]
        #[qproperty(bool, key_at_playhead, cxx_name = "keyAtPlayhead")]
        #[qproperty(f64, graph_value, cxx_name = "graphValue")]
        #[qproperty(i32, interpolation_count, cxx_name = "interpolationCount")]
        type FrameGraphItem = super::FrameGraphItemRust;

        #[inherit]
        #[cxx_name = "update"]
        fn request_update(self: Pin<&mut FrameGraphItem>);
        #[inherit]
        #[cxx_name = "width"]
        fn item_width(self: &FrameGraphItem) -> f64;
        #[inherit]
        #[cxx_name = "height"]
        fn item_height(self: &FrameGraphItem) -> f64;

        #[cxx_override]
        #[cxx_name = "frameGraphHandle"]
        fn frame_graph_handle(self: &FrameGraphItem) -> usize;

        #[qinvokable]
        #[cxx_name = "pointerMoved"]
        fn pointer_moved(self: Pin<&mut FrameGraphItem>, x: f64, y: f64);
        #[qinvokable]
        #[cxx_name = "pointerLeft"]
        fn pointer_left(self: Pin<&mut FrameGraphItem>);
        #[qinvokable]
        fn begin(
            self: Pin<&mut FrameGraphItem>,
            button: i32,
            x: f64,
            y: f64,
            control: bool,
            shift: bool,
        ) -> i32;
        #[qinvokable]
        #[cxx_name = "updatePointer"]
        fn update_pointer(self: Pin<&mut FrameGraphItem>, x: f64, y: f64);
        #[qinvokable]
        #[cxx_name = "endPointer"]
        fn end_pointer(self: Pin<&mut FrameGraphItem>);
        #[qinvokable]
        fn scroll(
            self: Pin<&mut FrameGraphItem>,
            pixel_delta: &QPoint,
            angle_delta: &QPoint,
            x: f64,
            y: f64,
            control: bool,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "handleKey"]
        fn handle_key(self: Pin<&mut FrameGraphItem>, key: i32);
        #[qinvokable]
        #[cxx_name = "previousKey"]
        fn previous_key(self: Pin<&mut FrameGraphItem>);
        #[qinvokable]
        #[cxx_name = "toggleKey"]
        fn toggle_key(self: Pin<&mut FrameGraphItem>);
        #[qinvokable]
        #[cxx_name = "nextKey"]
        fn next_key(self: Pin<&mut FrameGraphItem>);
        #[qinvokable]
        #[cxx_name = "editGraphValue"]
        fn edit_graph_value(self: Pin<&mut FrameGraphItem>, value: f64);
        #[qinvokable]
        #[cxx_name = "setInterpolation"]
        fn set_interpolation(self: Pin<&mut FrameGraphItem>, index: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "interpolationLabel"]
        fn interpolation_label(self: &FrameGraphItem, index: i32) -> QString;

        #[qsignal]
        #[cxx_name = "togglePlayback"]
        fn toggle_playback(self: Pin<&mut FrameGraphItem>);
    }

    impl cxx_qt::Constructor<()> for FrameGraphItem {}
}

pub struct FrameGraphItemRust {
    can_previous: bool,
    can_next: bool,
    key_at_playhead: bool,
    graph_value: f64,
    interpolation_count: i32,
    graph: SharedGraph,
}

impl Default for FrameGraphItemRust {
    fn default() -> Self {
        let state = FrameGraphState::sample();
        let status = state.status();
        Self {
            can_previous: status.can_previous,
            can_next: status.can_next,
            key_at_playhead: status.key_at_playhead,
            graph_value: status.value,
            interpolation_count: Interpolation::KEYFRAME.len() as i32,
            graph: Arc::new(Mutex::new(GraphModel {
                state,
                context_owner: None,
            })),
        }
    }
}

impl cxx_qt::Initialize for qobject::FrameGraphItem {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::FrameGraphItem {
    pub fn frame_graph_handle(&self) -> usize {
        Arc::as_ptr(&self.rust().graph) as usize
    }

    pub fn pointer_moved(mut self: Pin<&mut Self>, x: f64, y: f64) {
        self.rust().lock().state.pointer_moved(x, y);
        self.as_mut().request_update();
    }

    pub fn pointer_left(mut self: Pin<&mut Self>) {
        self.rust().lock().state.pointer_left();
        self.as_mut().request_update();
    }

    pub fn begin(
        mut self: Pin<&mut Self>,
        button: i32,
        x: f64,
        y: f64,
        control: bool,
        shift: bool,
    ) -> i32 {
        let button = match button {
            0 => FrameGraphPointerButton::Primary,
            1 => FrameGraphPointerButton::Middle,
            2 => FrameGraphPointerButton::Secondary,
            _ => panic!("Qt passed an invalid frame graph pointer button: {button}"),
        };
        let width = self.item_width();
        let height = self.item_height();
        let (actions, selected) = {
            let mut model = self.rust().lock();
            model.context_owner = None;
            let actions = model.state.begin_pointer(
                button,
                x,
                y,
                width,
                height,
                FrameGraphModifiers { control, shift },
            );
            let selected = actions.iter().find_map(|action| match action {
                FrameGraphAction::InterpolationRequested {
                    owner_id,
                    interpolation,
                    ..
                } => Some((*owner_id, *interpolation)),
                _ => None,
            });
            if let Some((owner_id, _)) = selected {
                model.context_owner = Some(owner_id);
            }
            (actions, selected.map(|(_, interpolation)| interpolation))
        };
        self.as_mut().finish(actions);
        selected
            .and_then(|selected| {
                Interpolation::KEYFRAME
                    .iter()
                    .position(|candidate| *candidate == selected)
            })
            .map_or(-1, |index| index as i32)
    }

    pub fn update_pointer(mut self: Pin<&mut Self>, x: f64, y: f64) {
        let width = self.item_width();
        let height = self.item_height();
        let actions = self.rust().lock().state.update_pointer(x, y, width, height);
        self.as_mut().finish(actions);
    }

    pub fn end_pointer(mut self: Pin<&mut Self>) {
        self.rust().lock().state.end_pointer();
        self.as_mut().request_update();
    }

    pub fn scroll(
        mut self: Pin<&mut Self>,
        pixel_delta: &QPoint,
        angle_delta: &QPoint,
        x: f64,
        y: f64,
        control: bool,
    ) -> bool {
        // QWheelEvent vertical deltas point up while GDK scroll controllers
        // use positive Y for scrolling down. Preserve native pixel precision;
        // angle deltas use Qt's documented 120 units per wheel detent.
        let has_pixel_delta = !pixel_delta.is_null();
        let (dx, dy, input) = if has_pixel_delta {
            (
                -f64::from(pixel_delta.x()),
                -f64::from(pixel_delta.y()),
                FrameGraphScrollInput::Surface,
            )
        } else {
            (
                -f64::from(angle_delta.x()) / QT_WHEEL_ANGLE_UNITS_PER_STEP,
                -f64::from(angle_delta.y()) / QT_WHEEL_ANGLE_UNITS_PER_STEP,
                FrameGraphScrollInput::Wheel,
            )
        };
        let width = self.item_width();
        let height = self.item_height();
        let handled = self.rust().lock().state.scroll(
            dx,
            dy,
            FrameGraphPointerPosition {
                x,
                y,
                width,
                height,
            },
            control,
            input,
        );
        if handled {
            self.as_mut().request_update();
        }
        handled
    }

    pub fn handle_key(mut self: Pin<&mut Self>, key: i32) {
        let key = match key {
            0 => FrameGraphKey::PreviousFrame,
            1 => FrameGraphKey::NextFrame,
            2 => FrameGraphKey::Start,
            3 => FrameGraphKey::End,
            4 => FrameGraphKey::ZoomIn,
            5 => FrameGraphKey::ZoomOut,
            6 => FrameGraphKey::Delete,
            7 => FrameGraphKey::Copy,
            8 => FrameGraphKey::Paste,
            9 => FrameGraphKey::TogglePlayback,
            _ => panic!("Qt passed an invalid frame graph key: {key}"),
        };
        let actions = self.rust().lock().state.key(key);
        self.as_mut().finish(actions);
    }

    pub fn previous_key(mut self: Pin<&mut Self>) {
        let actions = self.rust().lock().state.previous_key();
        self.as_mut().finish(actions);
    }

    pub fn toggle_key(mut self: Pin<&mut Self>) {
        let actions = self.rust().lock().state.toggle_key();
        self.as_mut().finish(actions);
    }

    pub fn next_key(mut self: Pin<&mut Self>) {
        let actions = self.rust().lock().state.next_key();
        self.as_mut().finish(actions);
    }

    pub fn edit_graph_value(mut self: Pin<&mut Self>, value: f64) {
        let actions = self.rust().lock().state.set_value(value);
        self.as_mut().finish(actions);
    }

    pub fn set_interpolation(mut self: Pin<&mut Self>, index: i32) -> bool {
        let Some(interpolation) = usize::try_from(index)
            .ok()
            .and_then(|index| Interpolation::KEYFRAME.get(index))
            .copied()
        else {
            return false;
        };
        {
            let mut model = self.rust().lock();
            let Some(owner_id) = model.context_owner.take() else {
                return false;
            };
            model.state.set_interpolation(owner_id, interpolation);
        }
        self.as_mut().request_update();
        true
    }

    pub fn interpolation_label(&self, index: i32) -> QString {
        usize::try_from(index)
            .ok()
            .and_then(|index| Interpolation::KEYFRAME.get(index))
            .map_or_else(QString::default, |interpolation| {
                QString::from(interpolation.label())
            })
    }

    fn finish(mut self: Pin<&mut Self>, actions: Vec<FrameGraphAction>) {
        for action in actions {
            if matches!(action, FrameGraphAction::TogglePlayback) {
                self.as_mut().toggle_playback();
            }
        }
        self.as_mut().sync_status();
        self.as_mut().request_update();
    }

    fn sync_status(mut self: Pin<&mut Self>) {
        let status = self.rust().lock().state.status();
        self.as_mut().set_can_previous(status.can_previous);
        self.as_mut().set_can_next(status.can_next);
        self.as_mut().set_key_at_playhead(status.key_at_playhead);
        self.as_mut().set_graph_value(status.value);
    }
}

impl FrameGraphItemRust {
    fn lock(&self) -> MutexGuard<'_, GraphModel> {
        self.graph
            .lock()
            .unwrap_or_else(|_| panic!("Qt frame graph state lock was poisoned"))
    }
}

fn graph_from_raw(graph: *const c_void) -> SharedGraph {
    assert!(!graph.is_null(), "Qt passed a null frame graph");
    let graph = graph.cast::<Mutex<GraphModel>>();
    unsafe { Arc::increment_strong_count(graph) };
    unsafe { Arc::from_raw(graph) }
}

#[unsafe(no_mangle)]
extern "C" fn shrimply_qt_frame_graph_renderer_new(graph: *const c_void) -> *mut c_void {
    Box::into_raw(Box::new(QtFrameGraphRenderer {
        graph: graph_from_raw(graph),
        renderer: TimelineRenderer::new(),
    }))
    .cast()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn shrimply_qt_frame_graph_renderer_free(renderer: *mut c_void) {
    assert!(!renderer.is_null(), "Qt passed a null frame graph renderer");
    drop(unsafe { Box::from_raw(renderer.cast::<QtFrameGraphRenderer>()) });
}

#[unsafe(no_mangle)]
extern "C" fn shrimply_qt_frame_graph_render(
    renderer: *mut c_void,
    width: u32,
    height: u32,
    scale: f32,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    dark: bool,
) -> i32 {
    assert!(!renderer.is_null(), "Qt passed a null frame graph renderer");
    shrimply_cross_ui_theme::set_dark(dark);
    let renderer = unsafe { &mut *renderer.cast::<QtFrameGraphRenderer>() };
    let result = (|| {
        let painter = renderer.renderer.begin_frame(
            UVec2::new(width, height),
            scale,
            Color::new(red, green, blue, alpha),
        )?;
        let animating = {
            let mut graph = renderer
                .graph
                .lock()
                .unwrap_or_else(|_| panic!("Qt frame graph state lock was poisoned"));
            graph.state.draw(
                &painter,
                f64::from(width) / f64::from(scale),
                f64::from(height) / f64::from(scale),
            );
            graph.state.is_animating()
        };
        renderer.renderer.end_frame()?;
        Ok::<bool, String>(animating)
    })();
    match result {
        Ok(animating) => i32::from(animating),
        Err(error) => {
            eprintln!("Qt frame graph OpenGL render failed: {error}");
            -1
        }
    }
}
