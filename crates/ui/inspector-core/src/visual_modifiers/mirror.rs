use shrimply_video_modifiers::mirror::MirrorModifier;

use crate::{InspectorRuntime, InspectorSection};

pub(super) fn presentation(
    value: &MirrorModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    _runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        super::modifier_boolean_control(
            format!("{base}/horizontal"),
            "Horizontal",
            value.horizontal,
            "edit-mirror",
        )
        .target(modifier_id),
    );
    section.add(
        super::modifier_boolean_control(
            format!("{base}/vertical"),
            "Vertical",
            value.vertical,
            "edit-mirror",
        )
        .target(modifier_id),
    );
    section
}
