use shrimply_project_document::project::{CanvasSize, Project, VideoItem, VideoItemContent};
use shrimply_project_evaluation::{FrameAudioAnalysis, TransformExpressionCache, VisualEvaluation};

#[derive(Clone, Copy)]
pub struct TrackingSample {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub projection: shrimply_3dgs_core::Projection,
    pub vertical_fov_degrees: f32,
}

pub struct Source {
    session: shrimply_3dgs_core::RenderSession,
}

pub struct Request<'a> {
    pub project: &'a Project,
    pub item: &'a VideoItem,
    pub position: shrimply_math_core::Time,
    pub audio: &'a FrameAudioAnalysis,
    pub canvas: CanvasSize,
    pub sequence_path: &'a [uuid::Uuid],
    pub track_id: uuid::Uuid,
}

#[derive(Clone)]
pub struct Prepared {
    pub session: shrimply_3dgs_core::RenderSession,
    pub params: shrimply_3dgs_core::RenderParams,
    pub width: u32,
    pub height: u32,
}

impl Source {
    pub fn new(item: &VideoItem) -> Result<Self, String> {
        if !matches!(item.content, VideoItemContent::Gaussian(_)) {
            return Err("Gaussian source received a different visual type".into());
        }
        Ok(Self {
            session: shrimply_3dgs_core::RenderSession::load(&item.file)
                .map_err(|error| error.to_string())?,
        })
    }

    pub fn matches(&self, item: &VideoItem) -> bool {
        matches!(item.content, VideoItemContent::Gaussian(_))
            && self.session.matches_asset(&item.file).unwrap_or(false)
    }

    pub fn prepare(
        &self,
        request: Request<'_>,
        expressions: &mut TransformExpressionCache,
    ) -> Result<Prepared, String> {
        let Request {
            project,
            item,
            position,
            audio,
            canvas,
            sequence_path,
            track_id,
        } = request;
        let VideoItemContent::Gaussian(_) = &item.content else {
            return Err("Gaussian source received a different visual type".into());
        };
        let mut params = evaluate(project, item, position, audio, expressions)?;
        if let Some(camera) = crate::camera_reconstruction::sample_for_visual(
            project,
            item,
            sequence_path,
            track_id,
            position,
        )? {
            apply_tracking(
                &mut params,
                TrackingSample {
                    position: camera.position,
                    rotation: camera.rotation,
                    projection: camera.projection,
                    vertical_fov_degrees: camera.vertical_fov_degrees,
                },
            );
        }
        Ok(Prepared {
            session: self.session.clone(),
            params,
            width: canvas.width.max(1),
            height: canvas.height.max(1),
        })
    }
}

pub fn evaluate(
    project: &Project,
    item: &VideoItem,
    position: shrimply_math_core::Time,
    audio: &FrameAudioAnalysis,
    expressions: &mut TransformExpressionCache,
) -> Result<shrimply_3dgs_core::RenderParams, String> {
    let VideoItemContent::Gaussian(scene) = &item.content else {
        return Err("Gaussian source received a different visual type".into());
    };
    let evaluation = VisualEvaluation::for_item_with_audio(project, item, position, audio);
    Ok(shrimply_project_evaluation::resolve_gaussian_scene(
        scene,
        &evaluation,
        expressions,
    ))
}

pub fn apply_tracking(params: &mut shrimply_3dgs_core::RenderParams, mut sample: TrackingSample) {
    (sample.position, sample.rotation) = shrimply_math_geometry::apply_reconstructed_camera_motion(
        sample.position,
        sample.rotation,
        params.camera.position,
        params.camera.rotation_degrees,
    );
    params.camera.position = sample.position;
    params.camera.rotation_degrees = shrimply_3dgs_core::rotation_degrees(
        sample.rotation,
        shrimply_3dgs_core::RotationOrder::Xyz,
    );
    params.camera.projection = sample.projection;
    params.camera.vertical_fov_degrees = sample.vertical_fov_degrees;
}
