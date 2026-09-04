use shrimply_core::timeline_value::TimelineValue;
use shrimply_video_modifiers::color_correction::ColorCorrectionModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

struct ColorControl<'a> {
    field: &'static str,
    label: &'static str,
    timeline: &'a TimelineValue<f32>,
    minimum: f64,
    maximum: f64,
    unit: &'static str,
    rotating: bool,
}

pub(super) fn presentation(
    value: &ColorCorrectionModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    let rows = [
        ColorControl {
            field: "exposure",
            label: "Exposure",
            timeline: &value.exposure,
            minimum: -10.0,
            maximum: 10.0,
            unit: "stops",
            rotating: false,
        },
        ColorControl {
            field: "gamma",
            label: "Gamma",
            timeline: &value.gamma,
            minimum: 0.01,
            maximum: 10.0,
            unit: "",
            rotating: false,
        },
        ColorControl {
            field: "temperature",
            label: "Temperature",
            timeline: &value.temperature,
            minimum: -1.0,
            maximum: 1.0,
            unit: "",
            rotating: false,
        },
        ColorControl {
            field: "tint",
            label: "Tint",
            timeline: &value.tint,
            minimum: -1.0,
            maximum: 1.0,
            unit: "",
            rotating: false,
        },
        ColorControl {
            field: "brightness",
            label: "Brightness",
            timeline: &value.brightness,
            minimum: -1.0,
            maximum: 1.0,
            unit: "",
            rotating: false,
        },
        ColorControl {
            field: "contrast",
            label: "Contrast",
            timeline: &value.contrast,
            minimum: -1.0,
            maximum: 1.0,
            unit: "",
            rotating: false,
        },
        ColorControl {
            field: "hue_degrees",
            label: "Hue",
            timeline: &value.hue_degrees,
            minimum: NumberSpec::default().minimum,
            maximum: NumberSpec::default().maximum,
            unit: "deg",
            rotating: true,
        },
        ColorControl {
            field: "saturation",
            label: "Saturation",
            timeline: &value.saturation,
            minimum: 0.0,
            maximum: 2.0,
            unit: "",
            rotating: false,
        },
        ColorControl {
            field: "value",
            label: "Value",
            timeline: &value.value,
            minimum: 0.0,
            maximum: 2.0,
            unit: "",
            rotating: false,
        },
    ];
    for row in rows {
        section.add(super::modifier_scalar_control(
            format!("{base}/{}", row.field),
            row.label,
            row.timeline,
            runtime,
            NumberSpec {
                minimum: row.minimum,
                maximum: row.maximum,
                drag_step: if row.rotating { 1.0 } else { 0.01 },
                digits: 2,
                unit: row.unit,
            },
            row.rotating,
        ));
    }
    section
}
