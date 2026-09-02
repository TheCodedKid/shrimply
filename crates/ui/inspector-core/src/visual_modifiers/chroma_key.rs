use shrimply_video_modifiers::chroma_key::ChromaKeyModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ChromaKeyModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_color_control(
        format!("{base}/key_color"),
        "Key color",
        &value.key_color,
        runtime,
    ));
    for (field, label, timeline) in [
        ("similarity", "Similarity", &value.similarity),
        ("softness", "Softness", &value.softness),
        ("spill_suppression", "Spill", &value.spill_suppression),
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
                digits: 2,
                unit: "",
            },
            false,
        ));
    }
    section
}
