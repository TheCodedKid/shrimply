use shrimply_video_modifiers::displacement_map::DisplacementMapModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &DisplacementMapModifier,
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
            drag_step: 0.01,
            digits: 2,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/scale"),
        "Scale",
        &value.scale,
        runtime,
        NumberSpec {
            minimum: 0.0,
            drag_step: 0.01,
            digits: 2,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/phase"),
        "Phase",
        &value.phase,
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
