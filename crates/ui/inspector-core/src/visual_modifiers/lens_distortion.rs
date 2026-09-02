use shrimply_video_modifiers::lens_distortion::LensDistortionModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &LensDistortionModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/distortion"),
        "Distortion",
        &value.distortion,
        runtime,
        NumberSpec {
            minimum: -1.0,
            maximum: 1.0,
            drag_step: 0.01,
            ..NumberSpec::default()
        },
        false,
    ));
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
    section
}
