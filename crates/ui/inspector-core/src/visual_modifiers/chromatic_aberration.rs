use shrimply_video_modifiers::chromatic_aberration::ChromaticAberrationModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ChromaticAberrationModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("red_offset_x", "Red X", &value.red_offset_x),
        ("red_offset_y", "Red Y", &value.red_offset_y),
        ("blue_offset_x", "Blue X", &value.blue_offset_x),
        ("blue_offset_y", "Blue Y", &value.blue_offset_y),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                drag_step: 0.01,
                digits: 2,
                unit: "px",
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section
}
