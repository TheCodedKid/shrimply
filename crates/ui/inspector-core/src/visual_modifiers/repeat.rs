use shrimply_video_modifiers::repeat::RepeatModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &RepeatModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("copies_x", "Copies X", &value.copies_x),
        ("copies_y", "Copies Y", &value.copies_y),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 1.0,
                drag_step: 1.0,
                digits: 0,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section.add(super::modifier_vector2_control(
        format!("{base}/step"),
        "Step",
        &value.step,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/row_offset"),
        "Row offset",
        &value.row_offset,
        runtime,
        NumberSpec {
            drag_step: 0.01,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/row_offset_axis"),
            "Offset axis",
            &value.row_offset_axis,
            runtime,
        )
        .live_commit("edit-vector-repeat-offset-axis"),
    );
    section
}
