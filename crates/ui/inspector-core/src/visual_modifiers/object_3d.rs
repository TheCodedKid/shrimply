use std::path::Path;

use shrimply_scene_3d::{MAX_IOR, MIN_IOR, MIN_ROUGHNESS};
use shrimply_video_modifiers::{
    ModifierEffect,
    scene_3d::{Object3dModifier, Scene3dModifierEffect},
};

use crate::{
    ControlKind, InspectorControl, InspectorControlAction, InspectorController, InspectorRuntime,
    InspectorSection, InspectorTarget, NumberSpec,
};

pub(super) fn presentation(
    value: &Object3dModifier,
    index: usize,
    modifier_id: uuid::Uuid,
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
            .value(&filename)
            .tooltip(filename),
    );
    let mut select = InspectorControl::new(ControlKind::Action, format!("{base}/file/select"), "")
        .value(if value.file.is_some() {
            "Replace model"
        } else {
            "Select model"
        })
        .action(InspectorControlAction::SelectObject3dModel { modifier_id });
    select.prefix_icon = "folder-open-symbolic".to_string();
    section.add(select);
    let mut clear = InspectorControl::new(ControlKind::Action, format!("{base}/file/clear"), "")
        .value("Clear model")
        .tooltip("Clear model")
        .sensitive(value.file.is_some())
        .action(InspectorControlAction::ClearObject3dModel { modifier_id });
    clear.prefix_icon = "window-close-symbolic".to_string();
    section.add(clear);
    for (field, label, timeline, degrees) in [
        ("position", "Position", &value.transform.position, false),
        ("anchor", "Anchor", &value.transform.anchor, false),
        (
            "rotation_degrees",
            "Rotation",
            &value.transform.rotation_degrees,
            true,
        ),
    ] {
        section.add(
            super::modifier_vector3_control(
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
            )
            .live_commit("edit-scene-3d-vec3"),
        );
    }
    section.add(
        super::modifier_vector3_control(
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
        )
        .live_commit("edit-scene-3d-vec3"),
    );
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
    section.set_target(modifier_id);
    section
}

pub(super) fn color<'a>(
    value: &'a Object3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>> {
    (field == "effect/effect/config/material/base_color"
        && value.material.base_color.id == timeline_id)
        .then_some(&value.material.base_color)
}

impl InspectorController {
    pub fn set_object_3d_model(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        path: &Path,
    ) -> Result<(), String> {
        validate_model(path)?;
        let mut project = self.project.borrow_mut();
        object_mut(&mut project, target, modifier_id)?.file = Some(path.into());
        shrimply_project::project::commit_edit(&project, "edit-3d-object-file");
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }

    pub fn clear_object_3d_model(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        object_mut(&mut project, target, modifier_id)?.file = None;
        shrimply_project::project::commit_edit(&project, "clear-3d-object-file");
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }
}

fn validate_model(path: &Path) -> Result<(), String> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
    {
        shrimply_scene_3d::load_glb(path)
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else {
        shrimply_scene_3d::load_obj(path)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn object_mut<'a>(
    project: &'a mut shrimply_project::project::Project,
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
) -> Result<&'a mut Object3dModifier, String> {
    project
        .video_item_mut(super::video_address(target)?)
        .and_then(|item| {
            item.modifiers
                .iter_mut()
                .find(|modifier| modifier.id == modifier_id)
        })
        .and_then(|modifier| match &mut modifier.effect {
            ModifierEffect::Scene3d(effect) => match &mut **effect {
                Scene3dModifierEffect::Object(value) => Some(&mut **value),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| "3D object modifier is no longer available".to_string())
}
