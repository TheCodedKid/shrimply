use shrimply_scene_3d::{MAX_IOR, MIN_IOR, MIN_ROUGHNESS};
use shrimply_shape_3d::{MAX_SMOOTHNESS, MIN_SMOOTHNESS, Shape3dKind, Shape3dModifier};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &Shape3dModifier,
    index: usize,
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
    section.add(vector3(
        &base,
        "size",
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
            section.add(number(
                &base,
                "star_points",
                "Points",
                &value.star_points,
                runtime,
                spec(3.0, 64.0, 1.0, 0, ""),
            ));
            section.add(number(
                &base,
                "star_inner_radius_percent",
                "Inner radius",
                &value.star_inner_radius_percent,
                runtime,
                unit_spec("%"),
            ));
        }
        Shape3dKind::Arrow => {
            section.add(number(
                &base,
                "arrow_shaft_width_percent",
                "Shaft width",
                &value.arrow_shaft_width_percent,
                runtime,
                unit_spec("%"),
            ));
            section.add(number(
                &base,
                "arrow_head_length_percent",
                "Head length",
                &value.arrow_head_length_percent,
                runtime,
                unit_spec("%"),
            ));
        }
        Shape3dKind::Cross => section.add(number(
            &base,
            "cross_arm_thickness_percent",
            "Arm thickness",
            &value.cross_arm_thickness_percent,
            runtime,
            unit_spec("%"),
        )),
        Shape3dKind::Disk => {
            section.add(number(
                &base,
                "disk_completion_degrees",
                "Completion",
                &value.disk_completion_degrees,
                runtime,
                spec(0.0, 360.0, 0.01, 2, "deg"),
            ));
            section.add(number(
                &base,
                "disk_inner_radius_percent",
                "Inner radius",
                &value.disk_inner_radius_percent,
                runtime,
                unit_spec("%"),
            ));
        }
        Shape3dKind::Torus => section.add(number(
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
        section.add(number(
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
        section.add(number(
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
    section.add(number(
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
    section.add(vector3(
        &base,
        "transform/position",
        "Position",
        &value.transform.position,
        runtime,
        vector_spec(false),
        false,
        false,
    ));
    section.add(vector3(
        &base,
        "transform/anchor",
        "Anchor",
        &value.transform.anchor,
        runtime,
        vector_spec(false),
        false,
        false,
    ));
    section.add(vector3(
        &base,
        "transform/rotation_degrees",
        "Rotation",
        &value.transform.rotation_degrees,
        runtime,
        vector_spec(true),
        false,
        true,
    ));
    section.add(vector3(
        &base,
        "transform/scale",
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
        section.add(number(
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
    section
}

fn number(
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
fn vector3(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<glam::Vec3>,
    runtime: InspectorRuntime,
    spec: NumberSpec,
    lock: bool,
    rotating: bool,
) -> InspectorControl {
    super::modifier_vector3_control(
        format!("{base}/{field}"),
        label,
        value,
        runtime,
        spec,
        lock,
        rotating,
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
