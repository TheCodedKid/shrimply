use shrimply_video_modifiers::erode_dilate::ErodeDilateModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ErodeDilateModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/operation"),
            "Operation",
            &value.operation,
            runtime,
        )
        .live_commit("edit-erode-dilate-operation"),
    );
    section.add(
        super::modifier_scalar_control(
            format!("{base}/radius"),
            "Radius",
            &value.radius,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 128.0,
                drag_step: 1.0,
                digits: 0,
                unit: "px",
            },
            false,
        )
        .integer(),
    );
    section
}
