use std::rc::Rc;

use shrimply_asset::AssetSnapshot;
use shrimply_project::project::{CanvasSize, VideoItem, VideoItemContent};
use uuid::Uuid;

use crate::gpu::CudaVideoCompositor;
use crate::layer::{GpuFrame, RasterVisual, Visual};
use crate::visual_source::{VisualElement, VisualRender, VisualRenderRequest, VisualSourceCache};

pub struct ObjElement {
    state: shrimply_video_core::obj::State,
    cached: Option<CachedObjFrame>,
}

struct CachedObjFrame {
    renderer_generation: u64,
    width: u32,
    height: u32,
    environment: Option<AssetSnapshot>,
    scene: shrimply_render_3d::SceneIdentity,
    uniforms: shrimply_render_3d::obj::SceneUniforms,
    layer: Rc<crate::gpu::VisualFrame>,
}

impl ObjElement {
    pub fn new(_item: &VideoItem) -> Result<Self, String> {
        Ok(Self {
            state: Default::default(),
            cached: None,
        })
    }
}

impl VisualElement for ObjElement {
    fn matches(&self, item: &VideoItem, _canvas_size: CanvasSize) -> bool {
        self.state.matches(item)
    }

    fn draw(
        &mut self,
        request: VisualRenderRequest<'_>,
        compositor: &mut CudaVideoCompositor,
        track_id: Uuid,
        _cache: &mut VisualSourceCache,
    ) -> Result<VisualRender, String> {
        let VideoItemContent::Obj(scene) = &request.item.content else {
            return Err("OBJ renderer received a non-OBJ visual".to_string());
        };
        let tracked_camera = if let shrimply_scene_3d::CameraSource::Tracking(source) =
            &scene.camera.source
            && source.track_id != track_id
            && request
                .project
                .video_tracks
                .iter()
                .any(|track| track.id == source.track_id)
        {
            crate::camera_reconstruction::sample(
                request.item.id,
                source,
                request
                    .position
                    .signed_sub(request.item.start)
                    .saturating_add(request.item.animation_time_offset),
            )
            .map(|camera| shrimply_video_core::obj::Camera {
                position: camera.position,
                rotation: camera.rotation,
                projection: camera.projection,
                vertical_fov_degrees: camera.vertical_fov_degrees,
            })
        } else {
            None
        };
        let prepared = self.state.prepare(shrimply_video_core::obj::Request {
            project: request.project,
            item: request.item,
            position: request.position,
            audio_analysis: request.audio_analysis,
            render_canvas: request.render_canvas,
            content_accurate: request.accuracy.content_accurate(),
            tracked_camera,
        })?;
        let cached = self.cached.as_ref().filter(|cached| {
            let mut cached_uniforms = cached.uniforms;
            let cached_quality = cached_uniforms.pbr.render_quality;
            let requested_quality = prepared.uniforms.pbr.render_quality;
            let quality_satisfies = match requested_quality {
                shrimply_render_3d::obj::RenderQuality::Interactive => true,
                shrimply_render_3d::obj::RenderQuality::Final => matches!(
                    cached_quality,
                    shrimply_render_3d::obj::RenderQuality::Final
                ),
            };
            cached_uniforms.pbr.render_quality = requested_quality;
            request.transmission_background.is_none()
                && cached.renderer_generation == compositor.generated_renderer_generation()
                && cached.width == prepared.width
                && cached.height == prepared.height
                && cached.environment == prepared.environment
                && cached.scene == *prepared.session.identity()
                && quality_satisfies
                && cached_uniforms == prepared.uniforms
        });
        let layer = if let Some(cached) = cached {
            cached.layer.clone()
        } else {
            let layer = Rc::new(compositor.render_scene_3d(
                prepared.session.as_ref(),
                prepared.width,
                prepared.height,
                &prepared.params,
                request.transmission_background,
            )?);
            if request.transmission_background.is_none() {
                self.cached = Some(CachedObjFrame {
                    renderer_generation: compositor.generated_renderer_generation(),
                    width: prepared.width,
                    height: prepared.height,
                    environment: prepared.environment,
                    scene: prepared.session.identity().clone(),
                    uniforms: prepared.uniforms,
                    layer: layer.clone(),
                });
            }
            layer
        };
        Ok(VisualRender::Ready(Visual::Raster(
            RasterVisual::materialized(GpuFrame::Rgba(layer), request.state.baked()),
        )))
    }
}
