use super::*;

pub struct ToolkitPreview {
    project: Rc<RefCell<Project>>,
    player_state: SharedPlayerState,
    preferences: preferences_store::SharedPreferences,
    media: PreviewMedia,
    audio_player: Rc<AudioPlayer>,
    renderer: Option<renderer::ToolkitPreviewRenderer>,
    guide_input: guides::GuideInput,
    frame: Option<CompositedVideoFrame>,
    frame_rate_label: String,
    fullscreen: bool,
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
            guide_input: guides::GuideInput::default(),
            frame: None,
            frame_rate_label: String::from("--"),
            fullscreen: false,
        })
    }

    pub fn render(
        &mut self,
        width: u32,
        height: u32,
        pixels_per_point: f32,
        background_color: Color,
        fullscreen: bool,
    ) -> Result<(), String> {
        self.fullscreen = fullscreen;
        let background_color =
            shrimply_preview_runtime::background_color(background_color, fullscreen);
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
            self.renderer = Some(renderer::ToolkitPreviewRenderer::new()?);
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
                fullscreen,
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

    pub fn guides_visible(&self) -> bool {
        preferences_store::snapshot(&self.preferences).preview_guides_visible
    }

    pub fn set_guides_visible(&self, visible: bool) {
        preferences_store::set_preview_guides_visible(&self.preferences, visible);
    }

    pub fn pointer_move(&mut self, width: f32, height: f32, x: f32, y: f32) {
        let preferences = preferences_store::snapshot(&self.preferences);
        let mut project = self.project.borrow_mut();
        let viewport = renderer::toolkit_guide_viewport(
            &project,
            &preferences,
            width,
            height,
            self.fullscreen,
        );
        self.guide_input.pointer_move(
            &mut project.preview_guides,
            viewport,
            preferences.preview_guides_visible,
            glam::vec2(x, y),
        );
    }

    pub fn pointer_press(&mut self, width: f32, height: f32, x: f32, y: f32) -> bool {
        let preferences = preferences_store::snapshot(&self.preferences);
        let mut project = self.project.borrow_mut();
        let viewport = renderer::toolkit_guide_viewport(
            &project,
            &preferences,
            width,
            height,
            self.fullscreen,
        );
        self.guide_input.pointer_press(
            &mut project.preview_guides,
            viewport,
            preferences.preview_guides_visible,
            glam::vec2(x, y),
        )
    }

    pub fn pointer_release(&mut self, width: f32, height: f32, x: f32, y: f32) {
        let preferences = preferences_store::snapshot(&self.preferences);
        let mut project = self.project.borrow_mut();
        let viewport = renderer::toolkit_guide_viewport(
            &project,
            &preferences,
            width,
            height,
            self.fullscreen,
        );
        let changed = self.guide_input.pointer_release(
            &mut project.preview_guides,
            viewport,
            glam::vec2(x, y),
        );
        drop(project);
        if changed == Some(true) {
            guides::commit_edit(&self.project.borrow());
        }
    }

    pub fn pointer_cancel(&mut self) {
        self.guide_input
            .pointer_cancel(&mut self.project.borrow_mut().preview_guides);
    }

    pub fn pointer_leave(&mut self) {
        self.guide_input.pointer_leave();
    }

    pub fn pointer_cursor(&self) -> u8 {
        self.guide_input.cursor() as u8
    }
}

impl Drop for ToolkitPreview {
    fn drop(&mut self) {
        self.media.stop();
        self.audio_player.stop();
    }
}
