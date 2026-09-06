use ffmpeg_next::{format::Pixel, frame};
use shrimply_project::project::{ItemAddress, Project, Time};
use uuid::Uuid;

use crate::compositor::{EXPORT_ASSETS_LOADING, VideoExportRenderer};

pub use shrimply_video_core::transparent_fill::analysis::{
    FrameSource, FrameStatus, PreparedStatus, RunId, Status, active_run_prepared, cancel,
    prepare_status, status, status_for_run, status_prepared,
};

struct CudaFrameSource {
    renderer: VideoExportRenderer,
}

impl FrameSource for CudaFrameSource {
    fn frame(
        &mut self,
        project: &Project,
        position: Time,
        address: &ItemAddress,
        width: u32,
        height: u32,
    ) -> Result<FrameStatus, String> {
        let composited = match self
            .renderer
            .render_transparent_fill_input(project, position, address)
        {
            Ok(frame) => frame,
            Err(error) if error == EXPORT_ASSETS_LOADING => return Ok(FrameStatus::Pending),
            Err(error) => return Err(error),
        };
        let mut output = frame::Video::new(Pixel::RGBA, width, height);
        self.renderer.copy_to_rgba_frame(composited, &mut output)?;
        let row_bytes = width as usize * 4;
        let stride = output.stride(0);
        let mut rgba = Vec::with_capacity(row_bytes * height as usize);
        for row in output.data(0).chunks_exact(stride).take(height as usize) {
            rgba.extend_from_slice(&row[..row_bytes]);
        }
        Ok(FrameStatus::Ready(rgba))
    }
}

pub fn analyze(
    project: Project,
    address: &ItemAddress,
    modifier_id: Uuid,
) -> Result<RunId, String> {
    shrimply_video_core::transparent_fill::analysis::analyze_with(
        project,
        address,
        modifier_id,
        || VideoExportRenderer::new(48_000).map(|renderer| CudaFrameSource { renderer }),
    )
}
