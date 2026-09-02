use shrimply_video_modifiers::transparent_fill::{MAXIMUM_GAP, TransparentFillModifier};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &TransparentFillModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/tolerance"),
        "Tolerance",
        &value.tolerance,
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
    section.add(
        InspectorControl::new(
            ControlKind::Number,
            format!("{base}/maximum_gap"),
            "Maximum gap",
        )
        .value(value.maximum_gap.to_string())
        .number(NumberSpec {
            minimum: 0.0,
            maximum: f64::from(MAXIMUM_GAP),
            drag_step: 1.0,
            digits: 0,
            unit: "",
        })
        .tooltip("0 disables gap closing; positive values set the maximum gap in pixels")
        .live_commit("edit-transparent-fill"),
    );
    for (point_index, point) in value.points.iter().enumerate() {
        section.add(super::modifier_vector2_control(
            format!("{base}/points/{point_index}/position"),
            format!("Point {}", point_index + 1),
            &point.position,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                digits: 2,
                unit: "x",
            },
            false,
        ));
    }
    section
}
