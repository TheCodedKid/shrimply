use shrimply_core::timeline_value::TimelineValue;
use shrimply_video_modifiers::color_correction::ColorCorrectionModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ColorCorrectionModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    let rows: [(
        &str,
        &'static str,
        &TimelineValue<f32>,
        f64,
        f64,
        &'static str,
        bool,
    ); 9] = [
        (
            "exposure",
            "Exposure",
            &value.exposure,
            -10.0,
            10.0,
            "stops",
            false,
        ),
        ("gamma", "Gamma", &value.gamma, 0.01, 10.0, "", false),
        (
            "temperature",
            "Temperature",
            &value.temperature,
            -1.0,
            1.0,
            "",
            false,
        ),
        ("tint", "Tint", &value.tint, -1.0, 1.0, "", false),
        (
            "brightness",
            "Brightness",
            &value.brightness,
            -1.0,
            1.0,
            "",
            false,
        ),
        (
            "contrast",
            "Contrast",
            &value.contrast,
            -1.0,
            1.0,
            "",
            false,
        ),
        (
            "hue_degrees",
            "Hue",
            &value.hue_degrees,
            NumberSpec::default().minimum,
            NumberSpec::default().maximum,
            "deg",
            true,
        ),
        (
            "saturation",
            "Saturation",
            &value.saturation,
            0.0,
            2.0,
            "",
            false,
        ),
        ("value", "Value", &value.value, 0.0, 2.0, "", false),
    ];
    for (field, label, timeline, minimum, maximum, unit, rotating) in rows {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                maximum,
                drag_step: if rotating { 1.0 } else { 0.01 },
                digits: 2,
                unit,
            },
            rotating,
        ));
    }
    section
}
