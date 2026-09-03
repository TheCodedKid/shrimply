use shrimply_project::project::VideoItem;
use shrimply_video_modifiers::{
    ModifierEffect,
    vectorize::{
        MAX_ANGLE_DEGREES, MAX_BINARY_THRESHOLD, MAX_COLOR_PRECISION, MAX_GRADIENT_STEP,
        MAX_ITERATIONS, MAX_PATH_PRECISION, MAX_SEGMENT_LENGTH, MAX_SPECKLE_SIZE,
        MIN_COLOR_PRECISION, MIN_SEGMENT_LENGTH, VectorizeColorMode, VectorizeModifier,
        VectorizePathMode, VectorizePreset,
    },
};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &VectorizeModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    _runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect");
    let color = value.color_mode == VectorizeColorMode::Color;
    let spline = value.path_mode == VectorizePathMode::Spline;
    let mut section = InspectorSection::default();
    section.add(selector(
        format!("{base}/preset"),
        "Preset",
        enum_text(value.preset),
        &[
            ("custom", "Custom"),
            ("poster", "Poster"),
            ("photo", "Photo"),
            ("black_and_white", "Black & White"),
        ],
        true,
        "edit-vectorize-preset",
    ));
    section.add(selector(
        format!("{base}/color_mode"),
        "Color mode",
        enum_text(value.color_mode),
        &[("color", "Color"), ("black_and_white", "Black & White")],
        true,
        "edit-vectorize-option",
    ));
    section.add(selector(
        format!("{base}/hierarchy"),
        "Hierarchy",
        enum_text(value.hierarchy),
        &[("stacked", "Stacked"), ("cutout", "Cutout")],
        color,
        "edit-vectorize-option",
    ));
    section.add(selector(
        format!("{base}/path_mode"),
        "Path mode",
        enum_text(value.path_mode),
        &[
            ("pixel", "Pixel"),
            ("polygon", "Polygon"),
            ("spline", "Spline"),
        ],
        true,
        "edit-vectorize-option",
    ));
    section.add(integer(
        format!("{base}/speckle_size"),
        "Speckle size",
        value.speckle_size,
        0,
        MAX_SPECKLE_SIZE,
        true,
    ));
    section.add(integer(
        format!("{base}/color_precision"),
        "Color precision",
        value.color_precision,
        MIN_COLOR_PRECISION,
        MAX_COLOR_PRECISION,
        color,
    ));
    section.add(integer(
        format!("{base}/gradient_step"),
        "Gradient step",
        value.gradient_step,
        0,
        MAX_GRADIENT_STEP,
        color,
    ));
    section.add(integer(
        format!("{base}/binary_threshold"),
        "B&W threshold",
        value.binary_threshold,
        0,
        MAX_BINARY_THRESHOLD,
        !color,
    ));
    section.add(integer(
        format!("{base}/corner_threshold_degrees"),
        "Corner threshold",
        value.corner_threshold_degrees,
        0,
        MAX_ANGLE_DEGREES,
        spline,
    ));
    section.add(decimal(
        format!("{base}/segment_length"),
        "Segment length",
        value.segment_length,
        MIN_SEGMENT_LENGTH,
        MAX_SEGMENT_LENGTH,
        spline,
    ));
    section.add(integer(
        format!("{base}/max_iterations"),
        "Max iterations",
        value.max_iterations,
        0,
        MAX_ITERATIONS,
        spline,
    ));
    section.add(integer(
        format!("{base}/splice_threshold_degrees"),
        "Splice threshold",
        value.splice_threshold_degrees,
        0,
        MAX_ANGLE_DEGREES,
        spline,
    ));
    section.add(integer(
        format!("{base}/path_precision"),
        "Path precision",
        value.path_precision,
        0,
        MAX_PATH_PRECISION,
        true,
    ));
    for control in &mut section.controls {
        control.target_id = Some(modifier_id);
    }
    section
}

pub(super) fn set_field(
    item: &mut VideoItem,
    path: &str,
    text: &str,
) -> Option<Result<bool, String>> {
    let (index, field) = path.strip_prefix("/modifiers/")?.split_once('/')?;
    if !matches!(
        field,
        "effect/effect/preset"
            | "effect/effect/color_mode"
            | "effect/effect/hierarchy"
            | "effect/effect/path_mode"
            | "effect/effect/speckle_size"
            | "effect/effect/color_precision"
            | "effect/effect/gradient_step"
            | "effect/effect/binary_threshold"
            | "effect/effect/corner_threshold_degrees"
            | "effect/effect/segment_length"
            | "effect/effect/max_iterations"
            | "effect/effect/splice_threshold_degrees"
            | "effect/effect/path_precision"
    ) {
        return None;
    }
    let Some(modifier) = index
        .parse::<usize>()
        .ok()
        .and_then(|index| item.modifiers.get_mut(index))
    else {
        return Some(Err("vectorize modifier is no longer available".to_string()));
    };
    let ModifierEffect::Vectorize(value) = &mut modifier.effect else {
        return Some(Err("modifier is no longer Vectorize".to_string()));
    };
    Some(edit_field(value, field, text))
}

fn edit_field(value: &mut VectorizeModifier, field: &str, text: &str) -> Result<bool, String> {
    macro_rules! set_custom {
        ($field:ident, $value:expr) => {{
            let next = $value?;
            let changed = value.$field != next || value.preset != VectorizePreset::Custom;
            value.$field = next;
            value.preset = VectorizePreset::Custom;
            changed
        }};
    }
    let changed = match field {
        "effect/effect/preset" => {
            let preset = enum_value(text)?;
            if value.preset == preset {
                false
            } else {
                if preset == VectorizePreset::Custom {
                    value.preset = preset;
                } else {
                    *value = VectorizeModifier::from_preset(preset);
                }
                true
            }
        }
        "effect/effect/color_mode" => set_custom!(color_mode, enum_value(text)),
        "effect/effect/hierarchy" => set_custom!(hierarchy, enum_value(text)),
        "effect/effect/path_mode" => set_custom!(path_mode, enum_value(text)),
        "effect/effect/speckle_size" => {
            set_custom!(speckle_size, integer_value(text, 0, MAX_SPECKLE_SIZE))
        }
        "effect/effect/color_precision" => set_custom!(
            color_precision,
            integer_value(text, MIN_COLOR_PRECISION, MAX_COLOR_PRECISION)
        ),
        "effect/effect/gradient_step" => {
            set_custom!(gradient_step, integer_value(text, 0, MAX_GRADIENT_STEP))
        }
        "effect/effect/binary_threshold" => set_custom!(
            binary_threshold,
            integer_value(text, 0, MAX_BINARY_THRESHOLD)
        ),
        "effect/effect/corner_threshold_degrees" => {
            set_custom!(
                corner_threshold_degrees,
                integer_value(text, 0, MAX_ANGLE_DEGREES)
            )
        }
        "effect/effect/segment_length" => set_custom!(
            segment_length,
            decimal_value(text, MIN_SEGMENT_LENGTH, MAX_SEGMENT_LENGTH)
        ),
        "effect/effect/max_iterations" => {
            set_custom!(max_iterations, integer_value(text, 0, MAX_ITERATIONS))
        }
        "effect/effect/splice_threshold_degrees" => {
            set_custom!(
                splice_threshold_degrees,
                integer_value(text, 0, MAX_ANGLE_DEGREES)
            )
        }
        "effect/effect/path_precision" => {
            set_custom!(path_precision, integer_value(text, 0, MAX_PATH_PRECISION))
        }
        _ => unreachable!("only Vectorize fields are routed here"),
    };
    Ok(changed)
}

fn enum_value<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(text.to_string()))
        .map_err(|_| format!("invalid Vectorize option: {text}"))
}

fn integer_value(text: &str, minimum: u32, maximum: u32) -> Result<u32, String> {
    text.parse::<u32>()
        .ok()
        .filter(|value| (minimum..=maximum).contains(value))
        .ok_or_else(|| format!("invalid Vectorize value: {text}"))
}

fn decimal_value(text: &str, minimum: f32, maximum: f32) -> Result<f32, String> {
    text.parse::<f32>()
        .ok()
        .filter(|value| value.is_finite() && (minimum..=maximum).contains(value))
        .ok_or_else(|| format!("invalid Vectorize value: {text}"))
}

fn selector(
    path: String,
    label: &'static str,
    value: String,
    choices: &[(&str, &str)],
    sensitive: bool,
    commit: &'static str,
) -> InspectorControl {
    InspectorControl::new(ControlKind::Selector, path, label)
        .value(value)
        .choices(
            choices.iter().map(|v| v.0.to_string()).collect(),
            choices.iter().map(|v| v.1.to_string()).collect(),
        )
        .sensitive(sensitive)
        .immediate_commit(commit)
}
fn integer(
    path: String,
    label: &'static str,
    value: u32,
    minimum: u32,
    maximum: u32,
    sensitive: bool,
) -> InspectorControl {
    InspectorControl::new(ControlKind::Number, path, label)
        .value(value.to_string())
        .number(NumberSpec {
            minimum: f64::from(minimum),
            maximum: f64::from(maximum),
            drag_step: 1.0,
            digits: 0,
            unit: "",
        })
        .sensitive(sensitive)
        .live_commit("edit-vectorize-value")
}
fn decimal(
    path: String,
    label: &'static str,
    value: f32,
    minimum: f32,
    maximum: f32,
    sensitive: bool,
) -> InspectorControl {
    InspectorControl::new(ControlKind::Number, path, label)
        .value(value.to_string())
        .number(NumberSpec {
            minimum: f64::from(minimum),
            maximum: f64::from(maximum),
            drag_step: 0.1,
            digits: 1,
            unit: "",
        })
        .sensitive(sensitive)
        .live_commit("edit-vectorize-value")
}
fn enum_text(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .expect("vectorize enum must serialize")
        .as_str()
        .expect("vectorize enum must be text")
        .to_string()
}
