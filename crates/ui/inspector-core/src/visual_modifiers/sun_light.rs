use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::scene_3d::SunLightModifier;

pub(super) fn presentation(
    value: &SunLightModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector3_control(
        format!("{base}/rotation_degrees"),
        "Rotation",
        &value.rotation_degrees,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 2,
            unit: "°",
            ..NumberSpec::default()
        },
        false,
        true,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/color"),
        "Color",
        &value.color,
        runtime,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/intensity"),
        "Intensity",
        &value.intensity,
        runtime,
        NumberSpec {
            minimum: 0.0,
            drag_step: 0.01,
            digits: 2,
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(super::modifier_scalar_control(
        format!("{base}/angular_radius_degrees"),
        "Angular radius",
        &value.angular_radius_degrees,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 45.0,
            drag_step: 0.01,
            digits: 2,
            unit: "°",
        },
        false,
    ));
    section
}
