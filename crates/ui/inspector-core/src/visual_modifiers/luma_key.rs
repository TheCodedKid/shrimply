use shrimply_video_modifiers::luma_key::LumaKeyModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &LumaKeyModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("threshold", "Threshold", &value.threshold),
        ("softness", "Softness", &value.softness),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section.add(super::modifier_boolean_control(
        format!("{base}/invert"),
        "Invert",
        value.invert,
        "edit-luma-key-invert",
    ));
    section
}
