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
        ControlKind::LayeredNumber => number_value(
            control
                .value
                .parse::<f64>()
                .map_err(|_| format!("invalid timeline value: {}", control.value))?
                * control.store_multiplier,
        ),
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
        ControlKind::LayeredNumber => value
            .parse::<f64>()
            .map(|value| value * control.store_multiplier)
            .map_err(|_| format!("invalid timeline value: {value}"))
            .and_then(number_value),
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

pub(crate) fn default_expression(control: &InspectorControl) -> &'static str {
    match control.kind {
        ControlKind::LayeredVector2 => "[x, y]",
        ControlKind::LayeredVector3 => "[x, y, z]",
        _ => "value",
    }
}

pub(crate) fn boolean_action(action: InspectorAction, active: bool) -> InspectorAction {
    match action {
        InspectorAction::SetBoolean { path, .. } => InspectorAction::SetBoolean {
            path,
            value: active,
        },
        InspectorAction::SetVisualModifierAlphaMask { id, .. } => {
            InspectorAction::SetVisualModifierAlphaMask {
                id,
                enabled: active,
            }
        }
        action => action,
    }
}

pub(crate) fn optional_action(action: InspectorAction, active: bool) -> InspectorAction {
    match action {
        InspectorAction::SetOptional { path, value } => InspectorAction::SetOptional {
            path,
            value: active.then_some(value).flatten(),
        },
        action => boolean_action(action, active),
    }
}
