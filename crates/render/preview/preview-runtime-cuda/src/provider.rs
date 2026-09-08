use shrimply_preview_interaction_skia::provider::{self, GeometryPreparation};
pub use shrimply_preview_interaction_skia::provider::{
    BuildContext, PreparedGeometry, SnapPreparation, prepare_snap_scene, update_text_source_size,
};

pub fn prepare_geometry(
    project: &shrimply_project_document::project::Project,
    address: &shrimply_project_document::project::ItemAddress,
    position: shrimply_project_document::project::Time,
    audio_analysis: &shrimply_project_evaluation::FrameAudioAnalysis,
    expression_cache: &std::cell::RefCell<shrimply_project_evaluation::TransformExpressionCache>,
    viewport: shrimply_preview_provider_skia::PreviewViewport,
    extensions: Option<
        &std::collections::HashMap<
            shrimply_preview_provider_skia::PreviewExtensionKey,
            Box<dyn std::any::Any>,
        >,
    >,
) -> Option<PreparedGeometry> {
    provider::prepare_geometry(
        project,
        address,
        position,
        GeometryPreparation {
            audio_analysis,
            expression_cache,
            viewport,
            extensions,
            camera_sampler: sample_camera,
        },
    )
}

pub fn sample_camera(
    id: uuid::Uuid,
    source: &shrimply_3dgs_core::TrackingCameraSource,
    time: shrimply_project_document::project::Time,
) -> Option<shrimply_project_document::project::TrackedCameraPreview> {
    shrimply_visual_core::camera_reconstruction::sample(id, source, time).map(|camera| {
        shrimply_project_document::project::TrackedCameraPreview {
            position: camera.position,
            rotation: camera.rotation,
            projection: camera.projection,
            vertical_fov_degrees: camera.vertical_fov_degrees,
        }
    })
}
