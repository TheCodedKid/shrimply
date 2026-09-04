use shrimply_core::timeline_value::TimelineValue;
use shrimply_video_modifiers::{
    ModifierEffect, VectorModifierEffect,
    repeat::{RepeatModifier, RepeatOffsetAxis},
};

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub const OFFSET_AXIS_COMMIT: &str = "edit-vector-repeat-offset-axis";

pub(super) fn presentation(
    value: &RepeatModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("copies_x", "Copies X", &value.copies_x),
        ("copies_y", "Copies Y", &value.copies_y),
    ] {
        section.add(
            super::modifier_scalar_control(
                format!("{base}/{field}"),
                label,
                timeline,
                runtime,
                NumberSpec {
                    minimum: 1.0,
                    drag_step: 1.0,
                    digits: 0,
                    ..NumberSpec::default()
                },
                false,
            )
            .integer(),
        );
    }
    section.add(super::modifier_vector2_control(
        format!("{base}/step"),
        "Step",
        &value.step,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/row_offset"),
        "Row offset",
        &value.row_offset,
        runtime,
        NumberSpec {
            drag_step: 0.01,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/row_offset_axis"),
            "Offset axis",
            &value.row_offset_axis,
            runtime,
        )
        .live_commit(OFFSET_AXIS_COMMIT),
    );
    section.set_target(modifier_id);
    section
}

pub fn offset_axis(effect: &ModifierEffect) -> Option<&TimelineValue<RepeatOffsetAxis>> {
    let ModifierEffect::Vector(effect) = effect else {
        return None;
    };
    let VectorModifierEffect::Repeat(value) = &**effect else {
        return None;
    };
    Some(&value.row_offset_axis)
}

pub fn offset_axis_mut(
    effect: &mut ModifierEffect,
) -> Option<&mut TimelineValue<RepeatOffsetAxis>> {
    let ModifierEffect::Vector(effect) = effect else {
        return None;
    };
    let VectorModifierEffect::Repeat(value) = &mut **effect else {
        return None;
    };
    Some(&mut value.row_offset_axis)
}

pub(super) fn offset_axis_at_path<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<RepeatOffsetAxis>> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != "effect/effect/config/row_offset_axis" {
        return None;
    }
    offset_axis(&modifier.effect).filter(|timeline| timeline.id == timeline_id)
}

pub(super) fn number<'a>(
    value: &'a RepeatModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<f32>> {
    let timeline = match field {
        "effect/effect/config/copies_x" => &value.copies_x,
        "effect/effect/config/copies_y" => &value.copies_y,
        "effect/effect/config/row_offset" => &value.row_offset,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}

pub(super) fn vector2<'a>(
    value: &'a RepeatModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<glam::Vec2>> {
    (field == "effect/effect/config/step" && value.step.id == timeline_id).then_some(&value.step)
}
