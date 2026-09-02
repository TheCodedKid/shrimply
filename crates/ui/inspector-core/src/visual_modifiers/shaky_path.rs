use crate::{InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::shaky_path::ShakyPathModifier;

pub(super) fn presentation(
    value: &ShakyPathModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(scalar(
        &base,
        "amplitude",
        "Amplitude",
        &value.amplitude,
        runtime,
        0.0,
        1_000_000.0,
        0.01,
        2,
        "px",
    ));
    section.add(scalar(
        &base,
        "step_size",
        "Step size",
        &value.step_size,
        runtime,
        0.1,
        1_000_000.0,
        0.01,
        2,
        "px",
    ));
    section.add(scalar(
        &base,
        "seed",
        "Seed",
        &value.seed,
        runtime,
        0.0,
        1_000_000.0,
        1.0,
        0,
        "",
    ));
    section.add(scalar(
        &base,
        "evolution",
        "Evolution",
        &value.evolution,
        runtime,
        -1_000_000.0,
        1_000_000.0,
        1.0,
        0,
        "",
    ));
    section
}

fn scalar(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    minimum: f64,
    maximum: f64,
    drag_step: f64,
    digits: i32,
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
            drag_step,
            digits,
            unit,
        },
        false,
    )
}
