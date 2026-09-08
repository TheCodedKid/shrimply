use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::Pixel;
use shrimply_project_document::project::{ItemAddress, Project};
use shrimply_resource_pipeline::JobContext;
use shrimply_visual_core::modifier_cache::encoder::{HevcBackend, RgbaAlpha};
use uuid::Uuid;

use crate::compositor::{EXPORT_ASSETS_LOADING, VideoExportRenderer};

pub use shrimply_visual_core::modifier_cache::{Status, effective_item, invalidate, status};

pub fn bake(project: Project, address: ItemAddress, modifier_id: Uuid) -> Result<(), String> {
    shrimply_visual_core::modifier_cache::bake(project, address, modifier_id, bake_inner)
}

fn bake_inner(
    request: shrimply_visual_core::modifier_cache::BakeRequest,
    context: &JobContext<shrimply_visual_core::modifier_cache::Progress>,
) -> Result<(), String> {
    let mut renderer = VideoExportRenderer::new(48_000)?;
    let (width, height) = (request.width, request.height);
    shrimply_visual_core::modifier_cache::render_and_encode(
        request,
        context,
        HevcBackend::Nvenc,
        RgbaAlpha::Premultiplied,
        move |project, position, address| match renderer
            .render_cache_item(project, position, address)
        {
            Ok(composited) => {
                let mut rgba = ffmpeg::frame::Video::new(Pixel::RGBA, width, height);
                renderer.copy_to_rgba_frame(composited, &mut rgba)?;
                Ok(Some(rgba))
            }
            Err(error) if error == EXPORT_ASSETS_LOADING => Ok(None),
            Err(error) => Err(error),
        },
    )
}
