use shrimply_video_modifiers::radial_blur::RadialBlurModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &RadialBlurModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector2_control(
        format!("{base}/center"),
        "Center",
        &value.center,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 1.0,
            drag_step: 0.01,
            digits: 2,
            unit: "x",
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/angle_degrees"),
        "Angle",
        &value.angle_degrees,
        runtime,
        NumberSpec {
            minimum: -360.0,
            maximum: 360.0,
            drag_step: 1.0,
            unit: "deg",
            ..NumberSpec::default()
        },
        true,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/samples"),
        "Samples",
        &value.samples,
        runtime,
        NumberSpec {
            minimum: 2.0,
            maximum: 128.0,
            drag_step: 1.0,
            digits: 0,
            ..NumberSpec::default()
        },
        false,
    ));
    section
}
