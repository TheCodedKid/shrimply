use shrimply_video_modifiers::hsv::HsvModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &HsvModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/hue_degrees"),
        "Hue",
        &value.hue_degrees,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            unit: "deg",
            ..NumberSpec::default()
        },
        true,
    ));
    for (field, label, timeline) in [
        ("saturation", "Saturation", &value.saturation),
        ("value", "Value", &value.value),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 2.0,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section
}
