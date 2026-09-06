use super::*;
pub use shrimply_timeline_core::draw_state::*;
pub use shrimply_timeline_core::scene::{
    TimelineModifiers, TrackAddMenuRequest, selected_timeline_items, selected_timeline_tracks,
};
pub(super) struct TimelineRuntime {
    pub(super) scene: shrimply_timeline_core::scene::Scene,
    pub(super) renderer: TimelineRenderer,
    pub(super) animation_tick_active: bool,
    pub(super) screen_recording: Option<video_recording::ScreenRecording>,
    pub(super) active_context_menu: Option<gtk::Popover>,
    pub(super) resource_jobs: Vec<shrimply_gtk_components::resource_pipeline::UiSubscription>,
}
impl TimelineRuntime {
    pub(super) fn new(
        project: Rc<RefCell<Project>>,
        player: SharedPlayerState,
        selection: SharedSelectionState,
        preferences: preferences_store::SharedPreferences,
        property_clipboard: shrimply_property_transfer::SharedClipboard,
        playback_performance: playback_performance::SharedCollector,
    ) -> Self {
        Self {
            scene: shrimply_timeline_core::scene::Scene::new(
                project,
                player,
                selection,
                preferences,
                property_clipboard,
                playback_performance,
            ),
            renderer: TimelineRenderer::new(),
            animation_tick_active: false,
            screen_recording: None,
            active_context_menu: None,
            resource_jobs: Vec::new(),
        }
    }
}
