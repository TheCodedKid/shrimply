use shrimply_video_modifiers::gaussian_blur::GaussianBlurModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &GaussianBlurModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector2_control(
        format!("/modifiers/{index}/effect/effect/config/radius"),
        "Radius",
        &value.radius,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 100.0,
            drag_step: 1.0,
            digits: 0,
            unit: "px",
        },
        true,
    ));
    section
}
