use std::{rc::Rc, sync::Arc};

use shrimply_math_color::Color;
use shrimply_project_document::project::{
    CanvasSize, ResolvedTransform, VideoItem, VideoItemContent,
};
use uuid::Uuid;

use crate::gpu::{CudaVideoCompositor, VisualFrame};
use crate::layer::{GpuFrame, RasterVisual, Visual};
use crate::visual_source::{VisualElement, VisualRender, VisualRenderRequest, VisualSourceCache};

const LOADING_COLOR: Color<u8> = Color::new(104, 51, 12, 255);
const LOADING_PROGRESS_COLOR: Color<u8> = Color::new(255, 174, 85, 255);

pub struct BlenderElement {
    canvas_size: CanvasSize,
    source: shrimply_visual_core::blender::Source,
    host_frame: Option<Arc<shrimply_visual_core::blender::Frame>>,
    gpu_frame: Option<Rc<VisualFrame>>,
}

impl BlenderElement {
    pub fn new(item: &VideoItem, canvas_size: CanvasSize) -> Result<Self, String> {
        Ok(Self {
            canvas_size,
            source: shrimply_visual_core::blender::Source::new(item, canvas_size)?,
            host_frame: None,
            gpu_frame: None,
        })
    }

    fn loading_placeholder(
        &mut self,
        compositor: &mut CudaVideoCompositor,
        state: crate::layer::VisualState,
    ) -> Result<VisualRender, String> {
        if self.gpu_frame.is_none() {
            let frame = Rc::new(compositor.allocate_cached_rgba_layer(
                self.canvas_size.width,
                self.canvas_size.height,
                "persistent Blender preview",
            )?);
            let pixels = shrimply_loading_screen_skia::render(
                self.canvas_size.width,
                self.canvas_size.height,
                shrimply_i18n::text("Starting Blender…").as_ref(),
                LOADING_COLOR,
                LOADING_PROGRESS_COLOR,
            )?;
            compositor.upload_rgba_layer_into(&frame, &pixels)?;
            self.gpu_frame = Some(frame);
        }
        let frame = self
            .gpu_frame
            .as_ref()
            .expect("Blender loading screen was initialized");
        compositor.prepare_host_backed_frame(frame, "persistent Blender preview")?;
        Ok(VisualRender::LoadingPlaceholder(Visual::Raster(
            RasterVisual::materialized(GpuFrame::Rgba(frame.clone()), state),
        )))
    }
}

impl VisualElement for BlenderElement {
    fn matches(&self, item: &VideoItem, canvas_size: CanvasSize) -> bool {
        matches!(item.content, VideoItemContent::Blender(_)) && self.canvas_size == canvas_size
    }

    fn draw(
        &mut self,
        request: VisualRenderRequest<'_>,
        compositor: &mut CudaVideoCompositor,
        _track_id: Uuid,
        _cache: &mut VisualSourceCache,
    ) -> Result<VisualRender, String> {
        match self.source.poll(
            request.item,
            self.canvas_size,
            request.position,
            request.accuracy.content_accurate(),
        )? {
            shrimply_visual_core::blender::Status::Empty => Ok(VisualRender::Empty),
            shrimply_visual_core::blender::Status::Loading => {
                if self.host_frame.take().is_some() {
                    self.gpu_frame = None;
                }
                self.loading_placeholder(compositor, request.state)
            }
            shrimply_visual_core::blender::Status::Ready(host) => {
                let changed = self
                    .host_frame
                    .as_ref()
                    .is_none_or(|current| !Arc::ptr_eq(current, &host));
                if changed {
                    let frame = Rc::new(compositor.allocate_cached_rgba_layer(
                        host.width,
                        host.height,
                        "persistent Blender preview",
                    )?);
                    compositor.upload_blender_frame(&frame, &host.pixels)?;
                    self.host_frame = Some(host.clone());
                    self.gpu_frame = Some(frame);
                }
                let frame = self.gpu_frame.as_ref().expect("Blender frame was uploaded");
                compositor.prepare_host_backed_frame(frame, "persistent Blender preview")?;
                let mut state = request.state;
                if host.display_scale != glam::Vec2::ONE {
                    state.transform = state.transform.compose(
                        ResolvedTransform {
                            scale: host.display_scale,
                            ..ResolvedTransform::IDENTITY
                        }
                        .composed(),
                    );
                }
                Ok(VisualRender::Ready(Visual::Raster(
                    RasterVisual::materialized(GpuFrame::Rgba(frame.clone()), state),
                )))
            }
        }
    }
}
