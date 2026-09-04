use shrimply_core::timeline_value::{TimelineExpressionValue, TimelineValue};
use shrimply_evaluation::{
    ExpressionOutcome, FrameAudioAnalysis, TransformExpressionCache, VisualEvaluation,
};
use shrimply_project::project::{ItemAddress, Project, Time};

pub mod scalar;
pub mod vector;

pub const SCALAR_EXPRESSION_DEFAULT: &str = "value";
pub const VECTOR2_EXPRESSION_DEFAULT: &str = "[x, y]";
pub const VECTOR3_EXPRESSION_DEFAULT: &str = "[x, y, z]";

pub fn evaluate_visual_expression<T: TimelineExpressionValue>(
    project: &Project,
    item: &ItemAddress,
    position: Time,
    audio: &FrameAudioAnalysis,
    cache: &mut TransformExpressionCache,
    value: &TimelineValue<T>,
) -> Option<ExpressionOutcome<T>> {
    let position = project.timeline_time_to_sequence(&item.track(), position)?;
    let item = project.video_item(item)?;
    let evaluation = VisualEvaluation::for_item_with_audio(project, item, position, audio);
    Some(shrimply_evaluation::resolve_with_error(
        value,
        &evaluation,
        cache,
    ))
}
