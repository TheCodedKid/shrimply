use shrimply_video_modifiers::edge_detection::EdgeDetectionModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &EdgeDetectionModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/amount"),
        "Amount",
        &value.amount,
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
    section.add(super::modifier_color_control(
        format!("{base}/edge_color"),
        "Edge color",
        &value.edge_color,
        runtime,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/background_color"),
        "Background color",
        &value.background_color,
        runtime,
    ));
    section
}
