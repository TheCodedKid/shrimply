use shrimply_video_modifiers::scanlines_crt::ScanlinesCrtModifier;

use crate::{InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ScanlinesCrtModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(scalar(
        &base,
        "spacing",
        "Spacing",
        &value.spacing,
        runtime,
        1.0,
        100.0,
        "px",
    ));
    section.add(unit_scalar(
        &base,
        "intensity",
        "Intensity",
        &value.intensity,
        runtime,
    ));
    section.add(unit_scalar(
        &base,
        "curvature",
        "Curvature",
        &value.curvature,
        runtime,
    ));
    section.add(unit_scalar(
        &base,
        "mask_strength",
        "Mask strength",
        &value.mask_strength,
        runtime,
    ));
    section.set_target(modifier_id);
    section
}

pub(super) fn number<'a>(
    value: &'a ScanlinesCrtModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<f32>> {
    let timeline = match field {
        "effect/effect/config/spacing" => &value.spacing,
        "effect/effect/config/intensity" => &value.intensity,
        "effect/effect/config/curvature" => &value.curvature,
        "effect/effect/config/mask_strength" => &value.mask_strength,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}

fn unit_scalar(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    scalar(base, field, label, value, runtime, 0.0, 1.0, "")
}

fn scalar(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    minimum: f64,
    maximum: f64,
    unit: &'static str,
) -> InspectorControl {
    super::modifier_scalar_control(
        format!("{base}/{field}"),
        label,
        value,
        runtime,
        NumberSpec {
            minimum,
            maximum,
            drag_step: 0.01,
            digits: 2,
            unit,
        },
        false,
    )
}
