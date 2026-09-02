use shrimply_video_modifiers::scanlines_crt::ScanlinesCrtModifier;

use crate::{InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ScanlinesCrtModifier,
    index: usize,
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
    section
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
