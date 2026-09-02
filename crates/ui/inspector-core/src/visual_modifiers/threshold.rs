use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::threshold::ThresholdModifier;

pub(super) fn presentation(
    value: &ThresholdModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/threshold"),
        "Threshold",
        &value.threshold,
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
    section.add(super::modifier_color_control(
        format!("{base}/low_color"),
        "Low color",
        &value.low_color,
        runtime,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/high_color"),
        "High color",
        &value.high_color,
        runtime,
    ));
    section
}
