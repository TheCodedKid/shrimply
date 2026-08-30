use super::*;

pub struct ToolkitPreview {
    project: Rc<RefCell<Project>>,
    player_state: SharedPlayerState,
    preferences: preferences_store::SharedPreferences,
    media: PreviewMedia,
    audio_player: Rc<AudioPlayer>,
    renderer: Option<preview_surface::ToolkitPreviewRenderer>,
    frame: Option<crate::video::gpu::CompositedVideoFrame>,
    frame_rate_label: String,
}

impl ToolkitPreview {
    pub fn new(
        project: Rc<RefCell<Project>>,
        player_state: SharedPlayerState,
        playback_performance: playback_performance::SharedCollector,
        preferences: preferences_store::SharedPreferences,
        audio_player: Rc<AudioPlayer>,
    ) -> Result<Self, String> {
        let media = PreviewMedia::new(
            project.clone(),
            player_state.clone(),
            playback_performance,
            preferences.clone(),
        );
        Ok(Self {
            project,
            player_state,
            preferences,
            media,
            audio_player,
            renderer: None,
            frame: None,
            frame_rate_label: String::from("--"),
        })
    }

    pub fn render(
        &mut self,
        width: u32,
        height: u32,
        pixels_per_point: f32,
        background_color: Color,
    ) -> Result<(), String> {
        let snapshot = player_state::snapshot(&self.player_state);
        let update = self.media.poll();
        assert!(update.running, "video compositor stopped unexpectedly");
        if let Some(label) = update.render_elapsed.and_then(rendered_frame_rate_label) {
            self.frame_rate_label = label;
        }
        match update.visual {
            Some(VideoEvent::Frame { frame, .. }) => self.frame = Some(frame),
            Some(VideoEvent::Clear { .. }) => self.frame = None,
            Some(_) => unreachable!(),
            None => {}
        }
        if self.renderer.is_none() {
            self.renderer = Some(preview_surface::ToolkitPreviewRenderer::new()?);
        }
        let project = self.project.borrow();
        self.renderer
            .as_mut()
            .expect("toolkit preview renderer was initialized")
            .render(
                &project,
                snapshot.position,
                self.frame.as_ref(),
                glam::IVec2::new(width.max(1) as i32, height.max(1) as i32),
                pixels_per_point,
                background_color,
                &preferences_store::snapshot(&self.preferences),
            )
    }

    pub fn destroy(&mut self) {
        self.media.stop();
        self.audio_player.stop();
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.destroy();
        }
        self.renderer = None;
    }

    pub fn mark_step(&self, delta: i32) {
        self.media.mark_step(if delta < 0 {
            StepDirection::Backward
        } else {
            StepDirection::Forward
        });
    }

    pub fn frame_rate_label(&self) -> &str {
        &self.frame_rate_label
    }
}

impl Drop for ToolkitPreview {
    fn drop(&mut self) {
        self.media.stop();
        self.audio_player.stop();
    }
}
