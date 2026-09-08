use super::{
    BakeRequest, Progress,
    encoder::{HevcBackend, RgbaAlpha, VideoCacheEncoder},
};
use shrimply_project_document::project::{ItemAddress, Project, Time};
use shrimply_resource_pipeline::JobContext;

/// Backends supply completed RGBA frames; core owns timing, cancellation and encoding.
pub fn render_and_encode(
    request: BakeRequest,
    context: &JobContext<Progress>,
    backend: HevcBackend,
    alpha: RgbaAlpha,
    mut render: impl FnMut(
        &Project,
        Time,
        &ItemAddress,
    ) -> Result<Option<ffmpeg_next::frame::Video>, String>,
) -> Result<(), String> {
    let mut encoder = VideoCacheEncoder::new(
        &request.output,
        request.coded_width,
        request.coded_height,
        request.project.fps,
        request.settings.quality.qp(),
        backend,
    )?;
    for frame_index in 0..request.total_frames {
        let position = shrimply_math_core::time_from_frame(
            request.first_frame.saturating_add(frame_index),
            request.project.fps,
        )
        .ok_or("cache frame rate must be positive")?;
        let frame = loop {
            if context.is_cancelled() {
                return Err("visual cache bake cancelled".into());
            }
            if let Some(frame) = render(&request.project, position, &request.address)? {
                break frame;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        };
        if context.is_cancelled() {
            return Err("visual cache bake cancelled".into());
        }
        encoder.write(&frame, frame_index, alpha)?;
        if !context.report(Progress {
            completed: frame_index + 1,
            total: request.total_frames,
        }) {
            return Err("visual cache bake cancelled".into());
        }
    }
    encoder.finish()
}
