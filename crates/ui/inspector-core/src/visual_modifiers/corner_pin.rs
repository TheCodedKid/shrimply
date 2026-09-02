use shrimply_core::timeline_value::TimelineValue;
use shrimply_video_modifiers::corner_pin::CornerPinModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &CornerPinModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    let corners: [(&str, &'static str, &TimelineValue<glam::Vec2>); 4] = [
        ("top_left", "Top left", &value.top_left),
        ("top_right", "Top right", &value.top_right),
        ("bottom_right", "Bottom right", &value.bottom_right),
        ("bottom_left", "Bottom left", &value.bottom_left),
    ];
    for (field, label, timeline) in corners {
        section.add(super::modifier_vector2_control(
            format!("{base}/{field}"),
            label,
            timeline,
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
    section.add(super::modifier_scalar_control(
        format!("{base}/perspective"),
        "Perspective",
        &value.perspective,
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
    section
}
