use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::vignette::VignetteModifier;

pub(super) fn presentation(
    value: &VignetteModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let control = |field, label, timeline| {
        super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                digits: 2,
                unit: "",
            },
            false,
        )
    };
    let mut section = InspectorSection::default();
    section.add(control("amount", "Amount", &value.amount));
    section.add(control("midpoint", "Midpoint", &value.midpoint));
    section.add(control("softness", "Softness", &value.softness));
    section
}
