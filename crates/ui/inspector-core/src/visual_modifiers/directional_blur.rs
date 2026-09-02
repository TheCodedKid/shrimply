use shrimply_video_modifiers::directional_blur::DirectionalBlurModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &DirectionalBlurModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/radius"),
        "Radius",
        &value.radius,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 100.0,
            drag_step: 0.01,
            digits: 2,
            unit: "px",
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/angle_degrees"),
        "Angle",
        &value.angle_degrees,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 2,
            unit: "deg",
            ..NumberSpec::default()
        },
        true,
    ));
    section
}
