use std::rc::Rc;

use shrimply_project::project::{CanvasSize, VideoItem, VideoItemContent};
use uuid::Uuid;

use crate::gpu::CudaVideoCompositor;
use crate::layer::{GpuFrame, RasterVisual, Visual};
use crate::visual_source::{VisualElement, VisualRender, VisualRenderRequest, VisualSourceCache};

pub struct GaussianElement {
    session: shrimply_3dgs::RenderSession,
    expressions: shrimply_evaluation::TransformExpressionCache,
    cached: Option<CachedFrame>,
}

struct CachedFrame {
    renderer_generation: u64,
    width: u32,
    height: u32,
    params: shrimply_3dgs::RenderParams,
    layer: Rc<crate::gpu::VisualFrame>,
}

impl GaussianElement {
    pub fn new(item: &VideoItem) -> Result<Self, String> {
        Ok(Self {
            session: shrimply_3dgs::RenderSession::load(&item.file)
                .map_err(|error| error.to_string())?,
            expressions: Default::default(),
            cached: None,
        })
    }
}

impl VisualElement for GaussianElement {
    fn matches(&self, item: &VideoItem, _canvas_size: CanvasSize) -> bool {
        matches!(&item.content, VideoItemContent::Gaussian(_))
            && self.session.matches_asset(&item.file).unwrap_or(false)
    }

    fn draw(
        &mut self,
        request: VisualRenderRequest<'_>,
        compositor: &mut CudaVideoCompositor,
        track_id: Uuid,
        _cache: &mut VisualSourceCache,
    ) -> Result<VisualRender, String> {
        let VideoItemContent::Gaussian(_) = &request.item.content else {
            return Err("3DGS renderer received a non-3DGS visual".to_string());
        };
        let mut params = shrimply_video_core::gaussian::evaluate(
            request.project,
            request.item,
            request.position,
            request.audio_analysis,
            &mut self.expressions,
        )?;
        if let Some(camera) = shrimply_video_core::camera_reconstruction::sample_for_visual(
            request.project,
            request.item,
            request.sequence_path,
            track_id,
            request.position,
        )? {
            shrimply_video_core::gaussian::apply_tracking(
                &mut params,
                shrimply_video_core::gaussian::TrackingSample {
                    position: camera.position,
                    rotation: camera.rotation,
                    projection: camera.projection,
                    vertical_fov_degrees: camera.vertical_fov_degrees,
                },
            );
        }
        let canvas_size = request.render_canvas;
        let width = canvas_size.width.max(1);
        let height = canvas_size.height.max(1);
        let layer = if let Some(cached) = self.cached.as_ref().filter(|cached| {
            cached.renderer_generation == compositor.generated_renderer_generation()
                && cached.width == width
                && cached.height == height
                && cached.params == params
        }) {
            cached.layer.clone()
        } else {
            self.cached = None;
            let layer =
                Rc::new(compositor.render_gaussian_3d(&self.session, width, height, &params)?);
            self.cached = Some(CachedFrame {
                renderer_generation: compositor.generated_renderer_generation(),
                width,
                height,
                params,
                layer: layer.clone(),
            });
            layer
        };
        Ok(VisualRender::Ready(Visual::Raster(
            RasterVisual::materialized(GpuFrame::Rgba(layer), request.state.baked()),
        )))
    }
}
