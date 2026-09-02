use shrimply_scene_3d::{MAX_IOR, MIN_IOR, MIN_ROUGHNESS};
use shrimply_video_modifiers::scene_3d::Object3dModifier;

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &Object3dModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    let filename = value
        .file
        .as_deref()
        .and_then(std::path::Path::file_name)
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    section.add(
        InspectorControl::new(ControlKind::ReadOnly, format!("{base}/file"), "Model")
            .value(filename),
    );
    section.add(
        InspectorControl::new(
            ControlKind::Action,
            format!("{base}/file/select"),
            "",
        )
        .value(if value.file.is_some() {
            "Replace model"
        } else {
            "Select model"
        }),
    );
    section.add(
        InspectorControl::new(ControlKind::Action, format!("{base}/file/clear"), "")
            .value("Clear model")
            .sensitive(value.file.is_some()),
    );
    for (field, label, timeline, degrees) in [
        (
            "position",
            "Position",
            &value.transform.position,
            false,
        ),
        ("anchor", "Anchor", &value.transform.anchor, false),
        (
            "rotation_degrees",
            "Rotation",
            &value.transform.rotation_degrees,
            true,
        ),
    ] {
        section.add(super::modifier_vector3_control(
            format!("{base}/transform/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                drag_step: if degrees { 1.0 } else { 0.1 },
                digits: 2,
                unit: if degrees { "°" } else { "" },
                ..NumberSpec::default()
            },
            false,
            false,
        ));
    }
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
        (
            "metallic",
            "Metallic",
            &value.material.metallic,
            0.0,
            1.0,
        ),
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
        section.add(super::modifier_scalar_control(
            format!("{base}/material/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                maximum,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section.add(
        crate::selector::selector(
            format!("{base}/material/normal_mode"),
            "Normals",
            super::enum_text(value.material.normal_mode),
            [
                ("smooth".to_string(), "Phong".to_string()),
                ("spherical".to_string(), "SLERP".to_string()),
                ("pn_triangle".to_string(), "PN triangles".to_string()),
                ("flat".to_string(), "Flat".to_string()),
            ],
        )
        .immediate_commit("edit-3d-object-normals"),
    );
    section
}
