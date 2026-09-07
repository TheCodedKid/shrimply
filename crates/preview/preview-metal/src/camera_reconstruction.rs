use crate::compositor::Compositor;
use shrimply_3dgs::TrackingCameraSource;
use shrimply_project::project::{ItemAddress, Project, Time, TrackAddress};
use shrimply_video_core::camera_reconstruction::{self, AnalysisFrameRenderer};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn analyze(
    project: Project,
    address: ItemAddress,
    source: TrackingCameraSource,
    server_url: String,
) {
    camera_reconstruction::analyze(project, address, source, server_url, renderer);
}

fn renderer() -> Result<Box<dyn AnalysisFrameRenderer>, String> {
    Ok(Box::new(Compositor::default()))
}

impl AnalysisFrameRenderer for Compositor {
    fn render_jpeg(
        &mut self,
        project: &Project,
        position: Time,
        track: &TrackAddress,
        cancelled: &AtomicBool,
    ) -> Result<Option<Vec<u8>>, String> {
        self.set_capture_target(shrimply_preview_render_core::CaptureTarget::Track(
            track.clone(),
        ));
        loop {
            if cancelled.load(Ordering::Acquire) {
                return Err("camera analysis cancelled".into());
            }
            let image =
                objc2::rc::autoreleasepool(|_| self.poll_accurate_image(project, position))?;
            if let Some(image) = image {
                let pixels = crate::capture::rgba(&image, project.canvas_size)?;
                let stride = project.canvas_size.width as usize * size_of::<u32>();
                return camera_reconstruction::encode_tracking_frame(
                    &pixels,
                    project.canvas_size,
                    stride,
                );
            }
            std::thread::sleep(crate::FRAME_POLL_INTERVAL);
        }
    }
}
