use shrimply_video_modifiers::scene_3d::{GroundKind, GroundModifier};

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &GroundModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            format!("{base}/kind"),
            "Kind",
            super::enum_text(value.kind),
            [
                ("infinite".to_string(), "Infinite".to_string()),
                ("square".to_string(), "Square".to_string()),
            ],
        )
        .immediate_commit("edit-ground-kind"),
    );
    if value.kind == GroundKind::Square {
        section.add(scalar(
            &base,
            "size",
            "Size",
            &value.size,
            runtime,
            NumberSpec {
                minimum: f64::EPSILON,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
        ));
    }
    section.add(
        crate::selector::selector(
            format!("{base}/composite_enabled"),
            "Composite background",
            value.composite_enabled.to_string(),
            [
                ("false".to_string(), "Off".to_string()),
                ("true".to_string(), "On".to_string()),
            ],
        )
        .immediate_commit("edit-ground-composite"),
    );
    if value.composite_enabled {
        section.add(scalar(
            &base,
            "intensity",
            "Ground intensity",
            &value.intensity,
            runtime,
            NumberSpec {
                minimum: 0.0,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
        ));
    }
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
        false,
    ));
    for (field, label, timeline) in [
        ("opacity", "Opacity", &value.opacity),
        ("shadow_strength", "Shadow strength", &value.shadow_strength),
        ("reflection", "Reflection opacity", &value.reflection),
        ("roughness", "Roughness", &value.roughness),
    ] {
        section.add(scalar(
            &base,
            field,
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
        ));
    }
    for control in &mut section.controls {
        control.target_id = Some(modifier_id);
    }
    section
}

fn scalar(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    number: NumberSpec,
) -> crate::InspectorControl {
    super::modifier_scalar_control(
        format!("{base}/{field}"),
        label,
        value,
        runtime,
        number,
        false,
    )
}
