use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::zoom_blur::ZoomBlurModifier;

pub(super) fn presentation(
    value: &ZoomBlurModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector2_control(
        format!("{base}/center"),
        "Center",
        &value.center,
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
    section.add(super::modifier_scalar_control(
        format!("{base}/strength"),
        "Strength",
        &value.strength,
        runtime,
        NumberSpec {
            minimum: -1.0,
            maximum: 1.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/samples"),
        "Samples",
        &value.samples,
        runtime,
        NumberSpec {
            minimum: 2.0,
            maximum: 128.0,
            drag_step: 1.0,
            digits: 0,
            unit: "",
        },
        false,
    ));
    section
}
