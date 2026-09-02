use shrimply_scene_3d::{MAX_IOR, MIN_IOR, MIN_ROUGHNESS};
use shrimply_video_modifiers::scene_3d::Text3dModifier;

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &Text3dModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_text_control(
        format!("{base}/text"),
        "Text",
        &value.text,
        runtime,
    ));
    // The GTK-ordered Fonts and dynamic font variation rows are inserted centrally here.
    section.add(selector(
        format!("{base}/font_style"),
        "Style",
        enum_text(value.font_style),
        &[
            ("normal", "Normal"),
            ("italic", "Italic"),
            ("oblique", "Oblique"),
        ],
        "edit-3d-text-style",
    ));
    section.add(number(
        &base,
        "font_weight",
        "Font weight",
        &value.font_weight,
        runtime,
        NumberSpec {
            minimum: 1.0,
            maximum: 1000.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
    ));
    section.add(number(
        &base,
        "font_size",
        "Font size",
        &value.font_size,
        runtime,
        NumberSpec {
            minimum: 0.001,
            drag_step: 0.01,
            digits: 2,
            ..NumberSpec::default()
        },
    ));
    section.add(selector(
        format!("{base}/h_align"),
        "Horizontal alignment",
        enum_text(value.h_align),
        &[
            ("left", "Left"),
            ("center", "Center"),
            ("right", "Right"),
            ("fill", "Fill"),
        ],
        "edit-3d-text-align",
    ));
    section.add(selector(
        format!("{base}/v_align"),
        "Vertical alignment",
        enum_text(value.v_align),
        &[("top", "Top"), ("middle", "Middle"), ("bottom", "Bottom")],
        "edit-3d-text-align",
    ));
    section.add(selector(
        format!("{base}/direction"),
        "Direction",
        enum_text(value.direction),
        &[("horizontal", "Horizontal"), ("vertical", "Vertical")],
        "edit-3d-text-direction",
    ));
    section.add(number(
        &base,
        "depth",
        "Depth",
        &value.depth,
        runtime,
        NumberSpec {
            minimum: 0.001,
            drag_step: 0.01,
            digits: 2,
            ..NumberSpec::default()
        },
    ));
    section.add(number(
        &base,
        "roundness",
        "Roundness",
        &value.roundness,
        runtime,
        NumberSpec {
            minimum: 0.0,
            drag_step: 0.01,
            digits: 2,
            ..NumberSpec::default()
        },
    ));
    section.add(number(
        &base,
        "smoothness",
        "Smoothness",
        &value.smoothness,
        runtime,
        NumberSpec {
            minimum: f64::from(shrimply_text_3d::MIN_SMOOTHNESS),
            maximum: f64::from(shrimply_text_3d::MAX_SMOOTHNESS),
            drag_step: 1.0,
            digits: 0,
            unit: "",
        },
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
            NumberSpec {
                minimum,
                maximum,
                drag_step: 0.01,
                digits: 2,
                unit: "",
            },
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
        "edit-3d-text-normals",
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
        .expect("3D text enum must serialize")
        .as_str()
        .expect("3D text enum must be text")
        .to_string()
}
