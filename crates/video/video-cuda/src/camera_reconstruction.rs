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
    ) -> Result<Option<Vec<u8>>, String> {
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
        let width = rgba.width() as usize;
        let height = rgba.height() as usize;
        let visible = rgba
            .data(0)
            .chunks(rgba.stride(0))
            .take(height)
            .any(|row| row[..width * 4].chunks_exact(4).any(|pixel| pixel[3] != 0));
        visible.then(|| encode_jpeg(&rgba)).transpose()
    }
}

fn encode_jpeg(frame: &Video) -> Result<Vec<u8>, String> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let mut pixels = Vec::with_capacity(width * height * 4);
    for row in frame.data(0).chunks(frame.stride(0)).take(height) {
        for pixel in row[..width * 4].chunks_exact(4) {
            let alpha = u16::from(pixel[3]);
            pixels.extend_from_slice(&[
                (u16::from(pixel[0]) * alpha / 255) as u8,
                (u16::from(pixel[1]) * alpha / 255) as u8,
                (u16::from(pixel[2]) * alpha / 255) as u8,
                255,
            ]);
        }
    }
    let image = skia_safe::images::raster_from_data(
        &skia_safe::ImageInfo::new(
            (frame.width() as i32, frame.height() as i32),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Opaque,
            None,
        ),
        skia_safe::Data::new_copy(&pixels),
        width * 4,
    )
    .ok_or_else(|| "could not create 3D tracking proxy image".to_string())?;
    image
        .encode(None, skia_safe::EncodedImageFormat::JPEG, Some(95))
        .map(|data| data.as_bytes().to_vec())
        .ok_or_else(|| "could not encode 3D tracking proxy JPEG".to_string())
}
