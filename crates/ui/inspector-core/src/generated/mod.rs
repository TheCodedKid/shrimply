pub mod shape;
pub mod text;
use shrimply_core::{Color, FontFamily, timeline_value::TimelineValue};
use shrimply_project::project::{CanvasSize, ShapeItem, TextItem, VideoItem, VideoItemContent};

use crate::{
    ControlKind, InspectorControl, InspectorController, InspectorRuntime, InspectorTarget,
    ScalarGraph, VideoCard,
};

pub enum GeneratedItem<'a> {
    Shape(&'a ShapeItem),
    Text(&'a TextItem),
}

pub fn item(item: &VideoItem) -> Option<GeneratedItem<'_>> {
    match &item.content {
        VideoItemContent::Shape(shape) => Some(GeneratedItem::Shape(shape)),
        VideoItemContent::Text(text) => Some(GeneratedItem::Text(text)),
        _ => None,
    }
}

pub fn cards(
    item: &VideoItem,
    canvas_size: CanvasSize,
    runtime: InspectorRuntime,
    default_text_font: Option<&FontFamily>,
) -> Option<[VideoCard; 2]> {
    match self::item(item)? {
        GeneratedItem::Shape(shape) => Some(shape::cards(shape, canvas_size, runtime)),
        GeneratedItem::Text(text) => {
            Some(text::cards(text, canvas_size, runtime, default_text_font))
        }
    }
}

pub fn set_field(item: &mut VideoItem, path: &str, value: &str) -> Option<Result<bool, String>> {
    if matches!(&item.content, VideoItemContent::Text(_)) {
        text::set_field(item, path, value)
    } else {
        None
    }
}

pub fn is_timeline_path(path: &str) -> bool {
    shape::is_timeline_path(path) || text::is_timeline_path(path)
}

pub fn is_timeline_control(control: &InspectorControl) -> bool {
    matches!(
        control.kind,
        ControlKind::LayeredNumber
            | ControlKind::LayeredSelector
            | ControlKind::LayeredVector2
            | ControlKind::LayeredVector3
            | ControlKind::LayeredColor
            | ControlKind::LayeredText
    ) && is_timeline_path(
        control
            .timeline_path
            .as_deref()
            .unwrap_or(control.path.as_str()),
    )
}

pub fn has_dynamic_graph(control: &InspectorControl) -> bool {
    control.path == text::TEXT_PATH
}

pub(crate) fn control_graph(
    controller: &InspectorController,
    target: &InspectorTarget,
    control: &InspectorControl,
) -> Option<Result<Option<ScalarGraph>, String>> {
    if !has_dynamic_graph(control) {
        return None;
    }
    if control.kind != ControlKind::LayeredText
        || control.timeline_path.as_deref() != Some(text::TEXT_PATH)
    {
        return Some(Err(
            "generated text control has invalid graph metadata".to_string()
        ));
    }
    let Some(timeline_id) = control.timeline_id else {
        return Some(Err("generated text control has no timeline ID".to_string()));
    };
    Some(controller.generated_text_graph(target, &control.path, timeline_id))
}

pub(crate) fn integer_commits(path: &str) -> Option<(&'static str, &'static str, &'static str)> {
    (path == shape::STAR_POINTS_PATH).then_some((
        shape::INTEGER_EDIT_COMMIT,
        shape::INTEGER_KEYFRAME_COMMIT,
        shape::INTEGER_EXPRESSION_COMMIT,
    ))
}

pub(crate) fn integer_value<'a>(
    item: &'a VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<u32>> {
    shape::integer_value(item, path, timeline_id)
}

pub(crate) fn color_value<'a>(
    item: &'a VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<Color<u8>>> {
    shape::color_value(item, path, timeline_id)
        .or_else(|| text::color_value(item, path, timeline_id))
}

pub(crate) fn text_value<'a>(
    item: &'a VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<String>> {
    text::text_value(item, path, timeline_id)
}

#[derive(Clone, Copy)]
pub(crate) enum StepTimeline {
    Shape,
    Rounding,
    HorizontalAlign,
    VerticalAlign,
    Direction,
    FontStyle,
}

pub(crate) fn step_timeline(path: &str) -> Option<StepTimeline> {
    match path {
        shape::SHAPE_PATH => Some(StepTimeline::Shape),
        shape::ROUNDING_PATH => Some(StepTimeline::Rounding),
        text::HORIZONTAL_ALIGN_PATH => Some(StepTimeline::HorizontalAlign),
        text::VERTICAL_ALIGN_PATH => Some(StepTimeline::VerticalAlign),
        text::DIRECTION_PATH => Some(StepTimeline::Direction),
        text::FONT_STYLE_PATH => Some(StepTimeline::FontStyle),
        _ => None,
    }
}

pub(crate) fn step_timeline_type(path: &str) -> Option<&'static str> {
    Some(match step_timeline(path)? {
        StepTimeline::Shape => std::any::type_name::<shrimply_project::project::ShapeKind>(),
        StepTimeline::Rounding => {
            std::any::type_name::<shrimply_project::project::ShapeRoundingStrategy>()
        }
        StepTimeline::HorizontalAlign => {
            std::any::type_name::<shrimply_project::project::TextHorizontalAlign>()
        }
        StepTimeline::VerticalAlign => {
            std::any::type_name::<shrimply_project::project::VerticalAlign>()
        }
        StepTimeline::Direction => {
            std::any::type_name::<shrimply_project::project::TextDirection>()
        }
        StepTimeline::FontStyle => {
            std::any::type_name::<shrimply_project::project::TextFontStyle>()
        }
    })
}
