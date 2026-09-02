use shrimply_video_modifiers::emboss::EmbossModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &EmbossModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/direction_degrees"),
        "Direction",
        &value.direction_degrees,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 2,
            unit: "deg",
            ..NumberSpec::default()
        },
        true,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/depth"),
        "Depth",
        &value.depth,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 20.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/amount"),
        "Amount",
        &value.amount,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 1.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
        false,
    ));
    section
}
