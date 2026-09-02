use shrimply_video_modifiers::alpha_outline::AlphaOutlineModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &AlphaOutlineModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/width"),
        "Width",
        &value.width,
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
    section.add(super::modifier_color_control(
        format!("{base}/color"),
        "Color",
        &value.color,
        runtime,
    ));
    section
}
