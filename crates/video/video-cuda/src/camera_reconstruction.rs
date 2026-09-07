use ffmpeg_next::{format::Pixel, frame::Video};
use shrimply_3dgs::TrackingCameraSource;
use shrimply_project::project::{ItemAddress, Project, Time, TrackAddress};

use crate::compositor::VideoExportRenderer;

pub use shrimply_video_core::camera_reconstruction::{
    AnalysisStatus, ReconstructedCameraSample, apply_custom_camera_offset, cancel,
    has_matching_cache, sample, sample_item, status,
};

const ANALYSIS_AUDIO_SAMPLE_RATE: u32 = 48_000;

pub fn analyze(
    project: Project,
    camera_address: ItemAddress,
    source: TrackingCameraSource,
    server_url: String,
) {
    shrimply_video_core::camera_reconstruction::analyze(
        project,
        camera_address,
        source,
        server_url,
        renderer,
    );
}

fn renderer()
-> Result<Box<dyn shrimply_video_core::camera_reconstruction::AnalysisFrameRenderer>, String> {
    VideoExportRenderer::new(ANALYSIS_AUDIO_SAMPLE_RATE)
        .map(|renderer| Box::new(renderer) as Box<_>)
}

impl shrimply_video_core::camera_reconstruction::AnalysisFrameRenderer for VideoExportRenderer {
    fn render_jpeg(
        &mut self,
        project: &Project,
        position: Time,
        track: &TrackAddress,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<Option<Vec<u8>>, String> {
        if cancelled.load(std::sync::atomic::Ordering::Acquire) {
            return Err("camera analysis cancelled".into());
        }
        let gpu = self
            .render_track(project, position, track)
            .map_err(|error| format!("could not render source frame: {error}"))?;
        let mut rgba = Video::new(
            Pixel::RGBA,
            project.canvas_size.width,
            project.canvas_size.height,
        );
        self.copy_to_rgba_frame(gpu, &mut rgba)
            .map_err(|error| format!("could not copy source frame: {error}"))?;
        shrimply_video_core::camera_reconstruction::encode_tracking_frame(
            rgba.data(0),
            project.canvas_size,
            rgba.stride(0),
        )
    }
}
