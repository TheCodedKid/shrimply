use super::*;

pub(crate) struct ToolkitPreviewRenderer {
    renderer: VideoRenderer,
}

impl ToolkitPreviewRenderer {
    pub(crate) fn new() -> Result<Self, String> {
        Ok(Self {
            renderer: VideoRenderer::new()?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        project: &Project,
        position: Time,
        frame: Option<&CompositedVideoFrame>,
        surface: IVec2,
        pixels_per_point: f32,
        background_color: Color,
        preferences: &preferences_store::PreferencesSnapshot,
    ) -> Result<(), String> {
        let content_rect = video_content_rect(
            surface.x,
            surface.y,
            project.canvas_size.width,
            project.canvas_size.height,
            preferences.preview_padding_px,
        );
        self.renderer.render(
            surface,
            pixels_per_point,
            frame,
            Appearance {
                content_rect,
                shadow_size_px: preferences.preview_shadow_size_px,
                background_color,
                upsample_method: preferences.preview_upsample_method,
                downsample_method: preferences.preview_downsample_method,
            },
            |painter| {
                let surface_rect =
                    Rect::from_min_size(vec2(0.0, 0.0), vec2(surface.x as f32, surface.y as f32));
                draw_captions(
                    painter,
                    project,
                    position,
                    CaptionAppearance {
                        preview_rect: surface_rect,
                        font_size: preferences.caption_font_size,
                        background_color: preferences.caption_background_color,
                        bottom_inset: 0.0,
                    },
                    None,
                );
                if preferences.preview_guides_visible {
                    guides::draw(
                        painter,
                        project.preview_guides.as_ref(),
                        GlamVec2::new(
                            project.canvas_size.width.max(1) as f32,
                            project.canvas_size.height.max(1) as f32,
                        ),
                        content_rect,
                        surface_rect,
                        Color::BLUE5,
                    );
                }
            },
        )
    }

    pub(crate) fn destroy(&mut self) {
        self.renderer.destroy();
    }
}
