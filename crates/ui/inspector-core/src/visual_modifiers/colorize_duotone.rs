use shrimply_video_modifiers::colorize_duotone::ColorizeDuotoneModifier;

use crate::{InspectorRuntime, InspectorSection};

pub(super) fn presentation(
    value: &ColorizeDuotoneModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_color_control(
        format!("{base}/shadow_color"),
        "Shadow color",
        &value.shadow_color,
        runtime,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/highlight_color"),
        "Highlight color",
        &value.highlight_color,
        runtime,
    ));
    section
}
