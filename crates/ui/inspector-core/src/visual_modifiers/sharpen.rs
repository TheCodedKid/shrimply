use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::sharpen::SharpenModifier;

pub(super) fn presentation(
    value: &SharpenModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/amount"),
        "Amount",
        &value.amount,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 2.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/radius"),
        "Radius",
        &value.radius,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 20.0,
            drag_step: 0.01,
            digits: 2,
            unit: "px",
        },
        false,
    ));
    section
}
