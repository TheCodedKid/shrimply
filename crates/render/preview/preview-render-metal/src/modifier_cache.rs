use crate::compositor::Compositor;
use shrimply_project_document::project::{CanvasSize, ItemAddress, Project};
use shrimply_resource_pipeline::JobContext;
use shrimply_visual_core::modifier_cache::{
    self, BakeRequest, Progress,
    encoder::{HevcBackend, RgbaAlpha},
};
use uuid::Uuid;

pub fn bake(project: Project, address: ItemAddress, modifier_id: Uuid) -> Result<(), String> {
    modifier_cache::bake(project, address, modifier_id, bake_inner)
}

fn bake_inner(request: BakeRequest, context: &JobContext<Progress>) -> Result<(), String> {
    let mut renderer = Compositor::default();
    renderer.set_capture_target(shrimply_preview_render_core::CaptureTarget::ModifierInput {
        address: request.address.clone(),
        snap_content: false,
    });
    let size = CanvasSize {
        width: request.width,
        height: request.height,
    };
    modifier_cache::render_and_encode(
        request,
        context,
        HevcBackend::X265,
        RgbaAlpha::Straight,
        move |project, position, _| {
            objc2::rc::autoreleasepool(|_| {
                let Some(image) = renderer.poll_accurate_image(project, position)? else {
                    return Ok(None);
                };
                let pixels = crate::capture::rgba(&image, size)?;
                let mut frame = ffmpeg_next::frame::Video::new(
                    ffmpeg_next::format::Pixel::RGBA,
                    size.width,
                    size.height,
                );
                let stride = frame.stride(0);
                let row_bytes = size.width as usize * 4;
                for (row, source) in pixels.chunks_exact(row_bytes).enumerate() {
                    frame.data_mut(0)[row * stride..][..row_bytes].copy_from_slice(source);
                }
                Ok(Some(frame))
            })
        },
    )
}
