use shrimply_video_modifiers::scene_3d::PointLightModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &PointLightModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector3_control(
        format!("{base}/position"),
        "Position",
        &value.position,
        runtime,
        NumberSpec {
            drag_step: 0.1,
            digits: 2,
            ..NumberSpec::default()
        },
        false,
        false,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/color"),
        "Color",
        &value.color,
        runtime,
    ));
    for (field, label, timeline, minimum) in [
        ("intensity", "Intensity", &value.intensity, 0.0),
        ("range", "Range", &value.range, f64::EPSILON),
        ("radius", "Radius", &value.radius, 0.0),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section.set_target(modifier_id);
    section
}

pub(super) fn color<'a>(
    value: &'a PointLightModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>> {
    (field == "effect/effect/config/color" && value.color.id == timeline_id).then_some(&value.color)
}
