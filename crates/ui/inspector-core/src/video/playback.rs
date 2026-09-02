use serde_json::Value;
use shrimply_math_core::{Fraction, fraction_is_finite};
use shrimply_project::project::{
    COMMON_FRAME_RATES, MAX_MOTION_BLUR_SAMPLES, MAX_MOTION_BLUR_SHUTTER_ANGLE_DEGREES,
    MAX_MOTION_BLUR_SHUTTER_PHASE_DEGREES, MIN_MOTION_BLUR_SAMPLES,
    MIN_MOTION_BLUR_SHUTTER_ANGLE_DEGREES, MIN_MOTION_BLUR_SHUTTER_PHASE_DEGREES, Time, VideoItem,
    VisualMotionBlur,
};

use crate::{ControlKind, InspectorControl, InspectorSection, NumberSpec};

use super::VideoCard;

pub fn speed(item: &VideoItem) -> VideoCard {
    let value = shrimply_project::project::playback_speed_or_default(item.playback_speed);
    let mut section = InspectorSection::default();
    section.add(
        InspectorControl::new(ControlKind::Fraction, "/playback_speed", "Value")
            .components(fraction_components(value))
            .number(NumberSpec {
                drag_step: 0.05,
                digits: 2,
                unit: "x",
                ..NumberSpec::default()
            })
            .width_characters(9)
            .live_commit("video-speed"),
    );
    VideoCard::new("speed", "Speed", section).reset_fraction(
        "/playback_speed",
        shrimply_project::project::default_playback_speed(),
        "video-speed",
    )
}

pub(super) fn set_fraction(
    item: &mut VideoItem,
    path: &str,
    value: Fraction,
) -> Result<bool, String> {
    if !fraction_is_finite(value) {
        return Err("video fraction must be finite".to_string());
    }
    match path {
        "/playback_speed" => {
            let value = shrimply_project::project::playback_speed_or_default(value);
            if item.playback_speed == value {
                return Ok(false);
            }
            if shrimply_project::project::playback_speed_is_negative(item.playback_speed)
                != shrimply_project::project::playback_speed_is_negative(value)
            {
                item.time_offset = shrimply_project::project::video_source_time_at(item, item.end)
                    .unwrap_or_else(|| {
                        if shrimply_project::project::playback_speed_is_negative(value) {
                            item.source_duration
                        } else {
                            Time::ZERO
                        }
                    });
            }
            item.playback_speed = value;
        }
        "/playback_fps" => {
            if item.playback_fps == value {
                return Ok(false);
            }
            item.playback_fps = value;
        }
        _ => return Err(format!("unsupported video fraction field: {path}")),
    }
    Ok(true)
}

pub fn frame_rate(item: &VideoItem) -> VideoCard {
    let mut choices = vec![(
        fraction_text(shrimply_project::project::native_playback_fps()),
        "Native".to_string(),
    )];
    choices.extend(
        COMMON_FRAME_RATES
            .iter()
            .map(|rate| (fraction_text(rate.value), rate.label.to_string())),
    );
    if choices
        .iter()
        .all(|(value, _)| value != &fraction_text(item.playback_fps))
    {
        choices.push((
            fraction_text(item.playback_fps),
            shrimply_project::project::fraction_as_label(item.playback_fps),
        ));
    }
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            "/playback_fps",
            "FPS",
            fraction_text(item.playback_fps),
            choices,
        )
        .immediate_commit("video-frame-rate"),
    );
    VideoCard::new("frame-rate", "Frame rate", section).reset_fraction(
        "/playback_fps",
        shrimply_project::project::native_playback_fps(),
        "reset-video-frame-rate",
    )
}

pub fn repeat(item: &VideoItem) -> VideoCard {
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            "/repeat_strategy",
            "Strategy",
            enum_text(item.repeat_strategy),
            [
                ("repeat".to_string(), "Repeat".to_string()),
                ("ping_pong".to_string(), "Ping Pong".to_string()),
                ("hold".to_string(), "Hold".to_string()),
                ("empty".to_string(), "Empty".to_string()),
            ],
        )
        .immediate_commit("video-repeat"),
    );
    VideoCard::new("repeat", "Repeat", section).reset(
        "/repeat_strategy",
        serde_json::to_value(shrimply_project::project::RepeatStrategy::default())
            .expect("default repeat strategy must serialize"),
        "reset-video-repeat",
    )
}

pub fn motion_blur(item: &VideoItem) -> VideoCard {
    let enabled = item.motion_blur.enabled;
    let mut section = InspectorSection::default();
    section.add(
        InspectorControl::new(ControlKind::Boolean, "/motion_blur/enabled", "Enabled")
            .value(enabled.to_string())
            .immediate_commit("motion-blur-enabled"),
    );
    section.add(
        number(
            "/motion_blur/shutter_angle_degrees",
            "Angle",
            item.motion_blur.shutter_angle_degrees,
        )
        .number(NumberSpec {
            minimum: f64::from(MIN_MOTION_BLUR_SHUTTER_ANGLE_DEGREES),
            maximum: f64::from(MAX_MOTION_BLUR_SHUTTER_ANGLE_DEGREES),
            digits: 0,
            unit: "°",
            ..NumberSpec::default()
        })
        .sensitive(enabled)
        .immediate_commit("motion-blur-angle"),
    );
    section.add(
        number(
            "/motion_blur/shutter_phase_degrees",
            "Phase",
            item.motion_blur.shutter_phase_degrees,
        )
        .number(NumberSpec {
            minimum: f64::from(MIN_MOTION_BLUR_SHUTTER_PHASE_DEGREES),
            maximum: f64::from(MAX_MOTION_BLUR_SHUTTER_PHASE_DEGREES),
            digits: 0,
            unit: "°",
            ..NumberSpec::default()
        })
        .sensitive(enabled)
        .immediate_commit("motion-blur-phase"),
    );
    section.add(
        number("/motion_blur/samples", "Samples", item.motion_blur.samples)
            .number(NumberSpec {
                minimum: f64::from(MIN_MOTION_BLUR_SAMPLES),
                maximum: f64::from(MAX_MOTION_BLUR_SAMPLES),
                digits: 0,
                ..NumberSpec::default()
            })
            .sensitive(enabled)
            .immediate_commit("motion-blur-samples"),
    );
    VideoCard::new("motion-blur", "Motion blur", section).reset(
        "/motion_blur",
        serde_json::to_value(VisualMotionBlur::default())
            .expect("default motion blur must serialize"),
        "reset-motion-blur",
    )
}

pub(super) fn set_field(
    item: &mut VideoItem,
    path: &str,
    text: &str,
) -> Option<Result<bool, String>> {
    let result = match path {
        "/repeat_strategy" => serde_json::from_value(Value::String(text.to_string()))
            .map_err(|error| format!("invalid video repeat strategy: {error}"))
            .map(|value| replace(&mut item.repeat_strategy, value)),
        "/motion_blur/enabled" => text
            .parse()
            .map_err(|_| "motion blur enabled state must be true or false".to_string())
            .map(|value| replace(&mut item.motion_blur.enabled, value)),
        "/motion_blur/shutter_angle_degrees" => parse_bounded(
            text,
            MIN_MOTION_BLUR_SHUTTER_ANGLE_DEGREES,
            MAX_MOTION_BLUR_SHUTTER_ANGLE_DEGREES,
        )
        .map(|value| replace(&mut item.motion_blur.shutter_angle_degrees, value)),
        "/motion_blur/shutter_phase_degrees" => parse_bounded(
            text,
            MIN_MOTION_BLUR_SHUTTER_PHASE_DEGREES,
            MAX_MOTION_BLUR_SHUTTER_PHASE_DEGREES,
        )
        .map(|value| replace(&mut item.motion_blur.shutter_phase_degrees, value)),
        "/motion_blur/samples" => {
            parse_bounded(text, MIN_MOTION_BLUR_SAMPLES, MAX_MOTION_BLUR_SAMPLES)
                .map(|value| replace(&mut item.motion_blur.samples, value))
        }
        _ => return None,
    };
    Some(result)
}

fn parse_bounded<T>(text: &str, minimum: T, maximum: T) -> Result<T, String>
where
    T: Copy + Ord + std::str::FromStr,
{
    let value = text
        .parse::<T>()
        .map_err(|_| format!("invalid integer video playback value: {text}"))?;
    if value < minimum || value > maximum {
        Err(format!("video playback value is out of range: {text}"))
    } else {
        Ok(value)
    }
}

fn replace<T: PartialEq>(current: &mut T, value: T) -> bool {
    if *current == value {
        false
    } else {
        *current = value;
        true
    }
}

fn number(path: &str, label: &str, value: impl ToString) -> InspectorControl {
    InspectorControl::new(ControlKind::Number, path, label).value(value.to_string())
}

fn enum_text(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .expect("video playback enum must serialize")
        .as_str()
        .expect("video playback enum must serialize as text")
        .to_string()
}

fn fraction_text(value: shrimply_math_core::Fraction) -> String {
    format!(
        "{}/{}",
        shrimply_math_core::fraction_numerator(value),
        shrimply_math_core::fraction_denominator(value),
    )
}

fn fraction_components(value: shrimply_math_core::Fraction) -> Vec<String> {
    vec![
        shrimply_math_core::fraction_numerator(value).to_string(),
        shrimply_math_core::fraction_denominator(value).to_string(),
    ]
}
