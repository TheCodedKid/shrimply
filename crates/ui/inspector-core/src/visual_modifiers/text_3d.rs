use serde::de::DeserializeOwned;
use shrimply_scene_3d::{MAX_IOR, MIN_IOR, MIN_ROUGHNESS};
use shrimply_text_3d::{MAX_SMOOTHNESS, MIN_SMOOTHNESS};
use shrimply_video_modifiers::{
    ModifierEffect,
    scene_3d::{Scene3dModifierEffect, Text3dModifier},
};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &Text3dModifier,
    index: usize,
    modifier_id: uuid::Uuid,
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
    section.add(font_families_control(&base, &value.font_families));
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
    for axis in font_axes(value) {
        let current = value
            .font_variations
            .iter()
            .find(|variation| variation.axis == axis.tag)
            .map_or(axis.default, |variation| variation.value);
        section.add(
            InspectorControl::new(
                ControlKind::Number,
                format!("{base}/font_variations/{}", axis.tag),
                axis.tag,
            )
            .value(current.to_string())
            .number(NumberSpec {
                minimum: f64::from(axis.minimum),
                maximum: f64::from(axis.maximum),
                drag_step: f64::from(((axis.maximum - axis.minimum).abs() / 100.0).max(0.01)),
                digits: 2,
                unit: "",
            })
            .immediate_commit("edit-3d-text-variation"),
        );
    }
    section.add(number_control(
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
    section.add(number_control(
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
    section.add(number_control(
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
    section.add(number_control(
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
    section.add(
        number_control(
            &base,
            "smoothness",
            "Smoothness",
            &value.smoothness,
            runtime,
            NumberSpec {
                minimum: f64::from(MIN_SMOOTHNESS),
                maximum: f64::from(MAX_SMOOTHNESS),
                drag_step: 1.0,
                digits: 0,
                unit: "",
            },
        )
        .integer(),
    );
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
        section.add(number_control(
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
    section.set_target(modifier_id);
    section
}

fn font_families_control(base: &str, selected: &[shrimply_core::FontFamily]) -> InspectorControl {
    InspectorControl::new(
        ControlKind::FontFamilies,
        format!("{base}/font_families"),
        "Fonts",
    )
    .value(serde_json::to_string(selected).expect("font families must serialize"))
    .immediate_commit("edit-3d-text-font")
}

fn font_axes(value: &Text3dModifier) -> Vec<crate::font_cache::FontAxis> {
    let Some(family) = value.font_families.first() else {
        return Vec::new();
    };
    let capabilities = match family {
        shrimply_core::FontFamily::GoogleFonts { name } => {
            crate::font_cache::cached_capabilities(name).unwrap_or_default()
        }
        shrimply_core::FontFamily::Local { name } => crate::font_cache::local_capabilities(name),
    };
    capabilities
        .axes
        .into_iter()
        .filter(|axis| !matches!(axis.tag.as_str(), "wght" | "ital"))
        .collect()
}

fn number_control(
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

pub(super) fn number<'a>(
    value: &'a Text3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<f32>> {
    let timeline = match field {
        "effect/effect/config/font_weight" => &value.font_weight,
        "effect/effect/config/font_size" => &value.font_size,
        "effect/effect/config/depth" => &value.depth,
        "effect/effect/config/roundness" => &value.roundness,
        "effect/effect/config/smoothness" => &value.smoothness,
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
    value: &'a Text3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<glam::Vec3>> {
    let timeline = match field {
        "effect/effect/config/transform/position" => &value.transform.position,
        "effect/effect/config/transform/anchor" => &value.transform.anchor,
        "effect/effect/config/transform/rotation_degrees" => &value.transform.rotation_degrees,
        "effect/effect/config/transform/scale" => &value.transform.scale,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}

pub(super) fn color<'a>(
    value: &'a Text3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>> {
    (field == "effect/effect/config/material/base_color"
        && value.material.base_color.id == timeline_id)
        .then_some(&value.material.base_color)
}

pub(super) fn text<'a>(
    value: &'a Text3dModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<String>> {
    (field == "effect/effect/config/text" && value.text.id == timeline_id).then_some(&value.text)
}

pub(super) fn set_field(
    item: &mut shrimply_project::project::VideoItem,
    path: &str,
    text: &str,
) -> Option<Result<bool, String>> {
    let (index, field) = path.strip_prefix("/modifiers/")?.split_once('/')?;
    if !matches!(
        field,
        "effect/effect/config/font_families"
            | "effect/effect/config/font_style"
            | "effect/effect/config/h_align"
            | "effect/effect/config/v_align"
            | "effect/effect/config/direction"
            | "effect/effect/config/material/normal_mode"
    ) && !field.starts_with("effect/effect/config/font_variations/")
    {
        return None;
    }
    let Some(modifier) = index
        .parse::<usize>()
        .ok()
        .and_then(|index| item.modifiers.get_mut(index))
    else {
        return Some(Err("3D text modifier is no longer available".to_string()));
    };
    let ModifierEffect::Scene3d(effect) = &mut modifier.effect else {
        return None;
    };
    let Scene3dModifierEffect::Text(value) = &mut **effect else {
        return None;
    };
    Some(match field {
        "effect/effect/config/font_families" => set_font_families(value, text),
        "effect/effect/config/font_style" => set(&mut value.font_style, enum_value(text)),
        "effect/effect/config/h_align" => set(&mut value.h_align, enum_value(text)),
        "effect/effect/config/v_align" => set(&mut value.v_align, enum_value(text)),
        "effect/effect/config/direction" => set(&mut value.direction, enum_value(text)),
        "effect/effect/config/material/normal_mode" => {
            set(&mut value.material.normal_mode, enum_value(text))
        }
        field => set_font_variation(
            value,
            field
                .strip_prefix("effect/effect/config/font_variations/")
                .expect("validated font variation path must have an axis"),
            text,
        ),
    })
}

fn set_font_families(value: &mut Text3dModifier, text: &str) -> Result<bool, String> {
    let mut next: Vec<shrimply_core::FontFamily> =
        serde_json::from_str(text).map_err(|error| format!("invalid font families: {error}"))?;
    next.retain(|family| !family.name().trim().is_empty());
    for family in &mut next {
        match family {
            shrimply_core::FontFamily::Local { name }
            | shrimply_core::FontFamily::GoogleFonts { name } => {
                *name = name.trim().to_string();
            }
        }
    }
    let mut names = hashbrown::HashSet::new();
    next.retain(|family| names.insert(family.name().to_lowercase()));
    set(&mut value.font_families, Ok(next))
}

fn set_font_variation(value: &mut Text3dModifier, axis: &str, text: &str) -> Result<bool, String> {
    let next = text
        .parse::<f32>()
        .map_err(|_| format!("invalid font variation value: {text}"))?;
    if !next.is_finite() {
        return Err("font variation value must be finite".to_string());
    }
    let specification = font_axes(value)
        .into_iter()
        .find(|specification| specification.tag == axis)
        .ok_or_else(|| format!("font variation axis is no longer available: {axis}"))?;
    if !(specification.minimum..=specification.maximum).contains(&next) {
        return Err(format!(
            "font variation {axis} must be between {} and {}",
            specification.minimum, specification.maximum
        ));
    }
    if let Some(variation) = value
        .font_variations
        .iter_mut()
        .find(|variation| variation.axis == axis)
    {
        return set(&mut variation.value, Ok(next));
    }
    value.font_variations.push(shrimply_core::FontVariation {
        axis: axis.to_string(),
        value: next,
    });
    Ok(true)
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
