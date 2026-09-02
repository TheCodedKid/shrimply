use shrimply_video_modifiers::drop_shadow::DropShadowModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &DropShadowModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector2_control(
        format!("{base}/offset"),
        "Offset",
        &value.offset,
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
        format!("{base}/blur_radius"),
        "Blur",
        &value.blur_radius,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 100.0,
            drag_step: 0.01,
            digits: 2,
            unit: "px",
        },
        false,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/color"),
        "Color",
        &value.color,
        runtime,
    ));
    section
}
