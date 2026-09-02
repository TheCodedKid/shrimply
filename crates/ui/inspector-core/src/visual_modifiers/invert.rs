use shrimply_video_modifiers::invert::InvertModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &InvertModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("/modifiers/{index}/effect/effect/config/amount"),
        "Amount",
        &value.amount,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 1.0,
            drag_step: 0.01,
            ..NumberSpec::default()
        },
        false,
    ));
    section
}
