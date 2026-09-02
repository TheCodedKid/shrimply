use shrimply_video_modifiers::mask::MaskModifier;

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection};

pub(super) fn presentation(
    value: &MaskModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        InspectorControl::new(ControlKind::Action, format!("{base}/item_id"), "Source")
            .value(
                value
                    .item_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "Drag onto a visual clip…".to_string()),
            )
            .tooltip("Drag onto a visual clip in the timeline"),
    );
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/mode"),
            "Mode",
            &value.mode,
            runtime,
        )
        .live_commit("edit-mask-mode"),
    );
    section.add(super::modifier_boolean_control(
        format!("{base}/invert"),
        "Invert",
        value.invert,
        "edit-mask",
    ));
    section
}
