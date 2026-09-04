use crate::item::InspectorAction;
use crate::section::{ControlKind, InspectorControl};

pub(crate) fn fraction_value(control: &InspectorControl) -> f64 {
    let numerator = control
        .components
        .first()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or_default();
    let denominator = control
        .components
        .get(1)
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value != 0.0)
        .unwrap_or(1.0);
    numerator / denominator
}

pub(crate) fn timeline_value(control: &InspectorControl) -> Result<serde_json::Value, String> {
    match control.kind {
        ControlKind::LayeredNumber => control_number_value(control, &control.value),
        ControlKind::LayeredVector2 | ControlKind::LayeredVector3 => {
            let expected = if control.kind == ControlKind::LayeredVector2 {
                2
            } else {
                3
            };
            if control.components.len() != expected {
                return Err(format!(
                    "timeline vector must contain {expected} components"
                ));
            }
            control
                .components
                .iter()
                .map(|value| {
                    value
                        .parse::<f64>()
                        .map(|value| value * control.store_multiplier)
                        .map_err(|_| format!("invalid timeline component: {value}"))
                        .and_then(number_value)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(serde_json::Value::Array)
        }
        ControlKind::LayeredColor if control.components.len() == 4 => {
            let mut channels = [0_u8; 4];
            for (channel, value) in channels.iter_mut().zip(&control.components) {
                *channel = value
                    .parse::<u8>()
                    .map_err(|_| format!("invalid timeline color channel: {value}"))?;
            }
            Ok(serde_json::to_value(shrimply_core::Color::new(
                channels[0],
                channels[1],
                channels[2],
                channels[3],
            ))
            .expect("timeline color must serialize"))
        }
        ControlKind::LayeredColor => Err("timeline color must contain four channels".to_string()),
        ControlKind::LayeredBoolean => control
            .value
            .parse::<bool>()
            .map(shrimply_core::timeline_value::TimelineBool::from)
            .map(|value| serde_json::to_value(value).expect("timeline boolean must serialize"))
            .map_err(|_| format!("invalid timeline boolean: {}", control.value)),
        ControlKind::LayeredSelector => Ok(serde_json::Value::String(control.value.clone())),
        _ => Err("inspector control is not a layered value".to_string()),
    }
}

pub(crate) fn control_value(
    control: &InspectorControl,
    value: &str,
) -> Result<serde_json::Value, String> {
    match control.kind {
        ControlKind::LayeredNumber => control_number_value(control, value),
        ControlKind::LayeredBoolean => value
            .parse::<bool>()
            .map(shrimply_core::timeline_value::TimelineBool::from)
            .map(|value| serde_json::to_value(value).expect("timeline boolean must serialize"))
            .map_err(|_| format!("invalid timeline boolean: {value}")),
        ControlKind::LayeredSelector if control.values.iter().any(|choice| choice == value) => {
            Ok(serde_json::Value::String(value.to_string()))
        }
        ControlKind::LayeredSelector => Err(format!("invalid timeline selector value: {value}")),
        _ => Err("inspector control is not a layered scalar".to_string()),
    }
}

fn number_value(value: f64) -> Result<serde_json::Value, String> {
    serde_json::Number::from_f64(value)
        .map(serde_json::Value::Number)
        .ok_or_else(|| "timeline value must be finite".to_string())
}

fn control_number_value(
    control: &InspectorControl,
    value: &str,
) -> Result<serde_json::Value, String> {
    let value = value
        .parse::<f64>()
        .map(|value| control.store_number(value))
        .map_err(|_| format!("invalid timeline value: {value}"))?;
    if control.integer {
        if !value.is_finite()
            || value.fract() != 0.0
            || !(0.0..=f64::from(u32::MAX)).contains(&value)
        {
            return Err(format!("invalid unsigned integer timeline value: {value}"));
        }
        return Ok(serde_json::Value::from(value as u32));
    }
    number_value(value)
}

pub(crate) fn boolean_action(action: InspectorAction, active: bool) -> InspectorAction {
    match action {
        InspectorAction::SetBoolean { path, .. } => InspectorAction::SetBoolean {
            path,
            value: active,
        },
        InspectorAction::SetAlphaMask { target, .. } => InspectorAction::SetAlphaMask {
            target,
            enabled: active,
        },
        action => action,
    }
}
