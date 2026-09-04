use serde::de::DeserializeOwned;
use shrimply_project::project::VideoItem;
use shrimply_scene_3d::{MAX_IOR, MIN_IOR, MIN_ROUGHNESS};
use shrimply_shape_3d::{MAX_SMOOTHNESS, MIN_SMOOTHNESS, Shape3dKind, Shape3dModifier};
use shrimply_video_modifiers::{ModifierEffect, scene_3d::Scene3dModifierEffect};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &Shape3dModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(selector(
        format!("{base}/shape"),
        "Shape",
        enum_text(value.shape),
        &[
            ("box", "Box"),
            ("disk", "Disk / cylinder"),
            ("triangle", "Triangle"),
            ("star", "Star"),
            ("arrow", "Arrow"),
            ("diamond", "Diamond"),
            ("pentagon", "Pentagon"),
            ("hexagon", "Hexagon"),
            ("heart", "Heart"),
            ("octagon", "Octagon"),
            ("cross", "Cross"),
            ("sphere", "Sphere"),
            ("cone", "Cone"),
            ("torus", "Torus"),
            ("capsule", "Capsule"),
        ],
        "edit-3d-shape-kind",
    ));
    section.add(super::modifier_vector3_control(
        format!("{base}/size"),
        "Size",
        &value.size,
        runtime,
        NumberSpec {
            minimum: 0.0,
            drag_step: 0.1,
            digits: 2,
            ..NumberSpec::default()
        },
        true,
        false,
    ));

    match value.shape {
        Shape3dKind::Star => {
            section.add(integer(
                &base,
                "star_points",
                "Points",
                &value.star_points,
                runtime,
                spec(3.0, 64.0, 1.0, 0, ""),
            ));
            section.add(scalar_control(
                &base,
                "star_inner_radius_percent",
                "Inner radius",
                &value.star_inner_radius_percent,
                runtime,
                unit_spec("%"),
            ));
        }
        Shape3dKind::Arrow => {
            section.add(scalar_control(
                &base,
                "arrow_shaft_width_percent",
                "Shaft width",
                &value.arrow_shaft_width_percent,
                runtime,
                unit_spec("%"),
            ));
            section.add(scalar_control(
                &base,
                "arrow_head_length_percent",
                "Head length",
                &value.arrow_head_length_percent,
                runtime,
                unit_spec("%"),
            ));
        }
        Shape3dKind::Cross => section.add(scalar_control(
            &base,
            "cross_arm_thickness_percent",
            "Arm thickness",
            &value.cross_arm_thickness_percent,
            runtime,
            unit_spec("%"),
        )),
        Shape3dKind::Disk => {
            section.add(scalar_control(
                &base,
                "disk_completion_degrees",
                "Completion",
                &value.disk_completion_degrees,
                runtime,
                spec(0.0, 360.0, 0.01, 2, "deg"),
            ));
            section.add(scalar_control(
                &base,
                "disk_inner_radius_percent",
                "Inner radius",
                &value.disk_inner_radius_percent,
                runtime,
                unit_spec("%"),
            ));
        }
        Shape3dKind::Torus => section.add(scalar_control(
            &base,
            "torus_inner_radius_percent",
            "Inner radius",
            &value.torus_inner_radius_percent,
            runtime,
            unit_spec("%"),
        )),
        _ => {}
    }
    if value.shape.has_profile_corners() {
        section.add(scalar_control(
            &base,
            "corner_radius",
            "Corner radius",
            &value.corner_radius,
            runtime,
            NumberSpec {
                minimum: 0.0,
                drag_step: 0.01,
                digits: 2,
                ..NumberSpec::default()
            },
        ));
        section.add(selector(
            format!("{base}/rounding_strategy"),
            "Corner rounding",
            enum_text(value.rounding_strategy),
            &[
                ("continuous", "Continuous"),
                ("circular", "Circular"),
                ("chamfer", "Chamfer"),
            ],
            "edit-3d-shape-rounding",
        ));
    }
    if value.shape.is_extruded() || value.shape == Shape3dKind::Cone {
        section.add(scalar_control(
            &base,
            "edge_roundness",
            "Depth edge roundness",
            &value.edge_roundness,
            runtime,
            NumberSpec {
                minimum: 0.0,
                drag_step: 0.01,
                digits: 2,
                ..NumberSpec::default()
            },
        ));
    }
    section.add(integer(
        &base,
        "smoothness",
        "Smoothness",
        &value.smoothness,
        runtime,
        spec(
            f64::from(MIN_SMOOTHNESS),
            f64::from(MAX_SMOOTHNESS),
            1.0,
            0,
            "",
        ),
    ));
    section.add(super::modifier_vector3_control(
        format!("{base}/transform/position"),
        "Position",
        &value.transform.position,
        runtime,
        vector_spec(false),
        false,
        false,
    ));
    section.add(super::modifier_vector3_control(
        format!("{base}/transform/anchor"),
        "Anchor",
        &value.transform.anchor,
        runtime,
        vector_spec(false),
        false,
        false,
    ));
    section.add(super::modifier_vector3_control(
        format!("{base}/transform/rotation_degrees"),
        "Rotation",
        &value.transform.rotation_degrees,
        runtime,
        vector_spec(true),
        false,
        true,
    ));
    section.add(super::modifier_vector3_control(
        format!("{base}/transform/scale"),
        "Scale",
        &value.transform.scale,
        runtime,
        NumberSpec {
            minimum: 0.0,
            drag_step: 0.1,
            digits: 2,
            ..NumberSpec::default()
        },
        true,
        false,
    ));
    section.add(super::modifier_color_control(
        format!("{base}/material/base_color"),
        "Base color",
        &value.material.base_color,
        runtime,
    ));
    for (field, label, timeline, minimum, maximum) in [
        ("metallic", "Metallic", &value.material.metallic, 0.0, 1.0),
        (
            "roughness",
            "Roughness",
            &value.material.roughness,
            f64::from(MIN_ROUGHNESS),
            1.0,
        ),
        (
            "subsurface",
            "Subsurface",
            &value.material.subsurface,
            0.0,
            1.0,
        ),
        (
            "clearcoat",
            "Clearcoat",
            &value.material.clearcoat,
            0.0,
            1.0,
        ),
        ("sheen", "Sheen", &value.material.sheen, 0.0, 1.0),
        (
            "transmission",
            "Transmission",
            &value.material.transmission,
            0.0,
            1.0,
        ),
        (
            "ior",
            "Index of refraction",
            &value.material.ior,
            f64::from(MIN_IOR),
            f64::from(MAX_IOR),
        ),
    ] {
        section.add(scalar_control(
            &base,
            &format!("material/{field}"),
            label,
            timeline,
            runtime,
            spec(minimum, maximum, 0.01, 2, ""),
        ));
    }
    section.add(selector(
        format!("{base}/material/normal_mode"),
        "Normals",
        enum_text(value.material.normal_mode),
        &[
            ("smooth", "Phong"),
            ("spherical", "SLERP"),
            ("pn_triangle", "PN triangles"),
            ("flat", "Flat"),
        ],
        "edit-3d-shape-normals",
    ));
    section.set_target(modifier_id);
    section
}

fn integer(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    spec: NumberSpec,
) -> InspectorControl {
    scalar_control(base, field, label, value, runtime, spec).integer()
}

fn scalar_control(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    spec: NumberSpec,
) -> InspectorControl {
    super::modifier_scalar_control(
        format!("{base}/{field}"),
        label,
        value,
        runtime,
        spec,
        false,
    )
}
fn vector_spec(degrees: bool) -> NumberSpec {
    NumberSpec {
        drag_step: if degrees { 1.0 } else { 0.1 },
        digits: 2,
        unit: if degrees { "°" } else { "" },
        ..NumberSpec::default()
    }
}
fn unit_spec(unit: &'static str) -> NumberSpec {
    spec(0.0, 1.0, 0.01, 2, unit)
}
fn spec(minimum: f64, maximum: f64, drag_step: f64, digits: i32, unit: &'static str) -> NumberSpec {
    NumberSpec {
        minimum,
        maximum,
        drag_step,
        digits,
        unit,
    }
}
fn selector(
    path: String,
    label: &'static str,
    value: String,
    choices: &[(&str, &str)],
    commit: &'static str,
) -> InspectorControl {
    InspectorControl::new(ControlKind::Selector, path, label)
        .value(value)
        .choices(
            choices.iter().map(|v| v.0.to_string()).collect(),
            choices.iter().map(|v| v.1.to_string()).collect(),
        )
        .immediate_commit(commit)
}
fn enum_text(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .expect("3D shape enum must serialize")
        .as_str()
        .expect("3D shape enum must be text")
        .to_string()
}

pub(super) fn number<'a>(
    value: &'a Shape3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<f32>> {
    let timeline = match field {
        "effect/effect/config/corner_radius" => &value.corner_radius,
        "effect/effect/config/edge_roundness" => &value.edge_roundness,
        "effect/effect/config/smoothness" => &value.smoothness,
        "effect/effect/config/star_points" => &value.star_points,
        "effect/effect/config/star_inner_radius_percent" => &value.star_inner_radius_percent,
        "effect/effect/config/arrow_shaft_width_percent" => &value.arrow_shaft_width_percent,
        "effect/effect/config/arrow_head_length_percent" => &value.arrow_head_length_percent,
        "effect/effect/config/cross_arm_thickness_percent" => &value.cross_arm_thickness_percent,
        "effect/effect/config/disk_inner_radius_percent" => &value.disk_inner_radius_percent,
        "effect/effect/config/disk_completion_degrees" => &value.disk_completion_degrees,
        "effect/effect/config/torus_inner_radius_percent" => &value.torus_inner_radius_percent,
        "effect/effect/config/material/metallic" => &value.material.metallic,
        "effect/effect/config/material/roughness" => &value.material.roughness,
        "effect/effect/config/material/subsurface" => &value.material.subsurface,
        "effect/effect/config/material/clearcoat" => &value.material.clearcoat,
        "effect/effect/config/material/sheen" => &value.material.sheen,
        "effect/effect/config/material/transmission" => &value.material.transmission,
        "effect/effect/config/material/ior" => &value.material.ior,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}

pub(super) fn vector3<'a>(
    value: &'a Shape3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<glam::Vec3>> {
    let timeline = match field {
        "effect/effect/config/size" => &value.size,
        "effect/effect/config/transform/position" => &value.transform.position,
        "effect/effect/config/transform/anchor" => &value.transform.anchor,
        "effect/effect/config/transform/rotation_degrees" => &value.transform.rotation_degrees,
        "effect/effect/config/transform/scale" => &value.transform.scale,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}

pub(super) fn color<'a>(
    value: &'a Shape3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>> {
    (field == "effect/effect/config/material/base_color"
        && value.material.base_color.id == timeline_id)
        .then_some(&value.material.base_color)
}

pub(super) fn set_field(
    item: &mut VideoItem,
    path: &str,
    text: &str,
) -> Option<Result<bool, String>> {
    let (index, field) = path.strip_prefix("/modifiers/")?.split_once('/')?;
    if !matches!(
        field,
        "effect/effect/config/shape"
            | "effect/effect/config/rounding_strategy"
            | "effect/effect/config/material/normal_mode"
    ) {
        return None;
    }
    let Some(modifier) = index
        .parse::<usize>()
        .ok()
        .and_then(|index| item.modifiers.get_mut(index))
    else {
        return Some(Err("3D shape modifier is no longer available".to_string()));
    };
    let ModifierEffect::Scene3d(effect) = &mut modifier.effect else {
        return Some(Err("modifier is no longer a 3D shape".to_string()));
    };
    let Scene3dModifierEffect::Shape(value) = &mut **effect else {
        return Some(Err("modifier is no longer a 3D shape".to_string()));
    };
    Some(match field {
        "effect/effect/config/shape" => set(&mut value.shape, enum_value(text)),
        "effect/effect/config/rounding_strategy" => {
            set(&mut value.rounding_strategy, enum_value(text))
        }
        "effect/effect/config/material/normal_mode" => {
            set(&mut value.material.normal_mode, enum_value(text))
        }
        _ => unreachable!("validated 3D shape field must be handled"),
    })
}

fn set<T: PartialEq>(field: &mut T, next: Result<T, String>) -> Result<bool, String> {
    let next = next?;
    if *field == next {
        return Ok(false);
    }
    *field = next;
    Ok(true)
}

fn enum_value<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(text.to_string()))
        .map_err(|error| error.to_string())
}
