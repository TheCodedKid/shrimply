use shrimply_video_modifiers::glow_bloom::GlowBloomModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &GlowBloomModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline, maximum, unit) in [
        ("threshold", "Threshold", &value.threshold, 1.0, ""),
        ("radius", "Radius", &value.radius, 100.0, "px"),
        ("intensity", "Intensity", &value.intensity, 10.0, ""),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum,
                drag_step: 0.01,
                digits: 2,
                unit,
            },
            false,
        ));
    }
    section
}
