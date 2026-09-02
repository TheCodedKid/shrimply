use shrimply_video_modifiers::vectorize::{
    MAX_ANGLE_DEGREES, MAX_BINARY_THRESHOLD, MAX_COLOR_PRECISION, MAX_GRADIENT_STEP,
    MAX_ITERATIONS, MAX_PATH_PRECISION, MAX_SEGMENT_LENGTH, MAX_SPECKLE_SIZE, MIN_COLOR_PRECISION,
    MIN_SEGMENT_LENGTH, VectorizeColorMode, VectorizeModifier, VectorizePathMode,
};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &VectorizeModifier,
    index: usize,
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
    section
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
