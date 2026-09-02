use shrimply_video_modifiers::mirror::MirrorModifier;

use crate::{InspectorRuntime, InspectorSection};

pub(super) fn presentation(
    value: &MirrorModifier,
    index: usize,
    _runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_boolean_control(
        format!("{base}/horizontal"),
        "Horizontal",
        value.horizontal,
        "edit-mirror",
    ));
    section.add(super::modifier_boolean_control(
        format!("{base}/vertical"),
        "Vertical",
        value.vertical,
        "edit-mirror",
    ));
    section
}
