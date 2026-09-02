use shrimply_video_modifiers::transform::TransformModifier;

use crate::{InspectorControl, InspectorRuntime, NumberSpec};

#[derive(Clone, Debug, PartialEq)]
pub struct TransformModifierPresentation {
    pub position: InspectorControl,
    pub anchor: InspectorControl,
    pub scale: InspectorControl,
    pub shear: InspectorControl,
    pub rotation: InspectorControl,
}

pub(super) fn presentation(
    value: &TransformModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> TransformModifierPresentation {
    let base = format!("/modifiers/{index}/effect/effect/config/transform");
    TransformModifierPresentation {
        position: vector(
            &base,
            "position",
            "Position",
            value.position(),
            runtime,
            false,
        ),
        anchor: vector(&base, "anchor", "Anchor", value.anchor(), runtime, false),
        scale: vector(&base, "scale", "Scale", value.scale(), runtime, true),
        shear: vector(&base, "shear", "Shear", value.shear(), runtime, false),
        rotation: super::modifier_scalar_control(
            format!("{base}/rotation_degrees"),
            "Rotation",
            value.rotation_degrees(),
            runtime,
            NumberSpec {
                drag_step: 1.0,
                digits: 2,
                unit: "°",
                ..NumberSpec::default()
            },
            true,
        ),
    }
}

fn vector(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<glam::Vec2>,
    runtime: InspectorRuntime,
    scale: bool,
) -> InspectorControl {
    super::modifier_vector2_control(
        format!("{base}/{field}"),
        label,
        value,
        runtime,
        NumberSpec {
            drag_step: if scale { 0.01 } else { 1.0 },
            digits: if scale { 2 } else { 0 },
            unit: if scale { "x" } else { "px" },
            ..NumberSpec::default()
        },
        scale,
    )
}
