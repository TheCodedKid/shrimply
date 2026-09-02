use serde_json::{Map, Value};
use shrimply_inspector_core::audio_generator::AudioGeneratorControl;
use shrimply_inspector_core::{InspectorRuntime, LayeredState};
use shrimply_project::project::{AudioGenerator, Time};

use crate::item::{InspectorAction, InspectorItem};
use crate::section::{ControlKind, InspectorControl, InspectorSection, NumberSpec};

pub(crate) fn item(
    source: &Map<String, Value>,
    path: &str,
    runtime: InspectorRuntime,
) -> InspectorItem {
    let generator: AudioGenerator = serde_json::from_value(Value::Object(source.clone()))
        .expect("audio generator inspector value must be valid");
    let mut section = InspectorSection::default();
    for control in shrimply_inspector_core::audio_generator::controls(&generator) {
        match control {
            AudioGeneratorControl::Waveform { value, choices } => {
                let current = choices
                    .iter()
                    .find(|choice| choice.value == value)
                    .expect("audio generator waveform must be available");
                section.add(crate::selector::selector(
                    format!("{path}/waveform"),
                    "Waveform",
                    current.key,
                    choices
                        .iter()
                        .map(|choice| (choice.key.to_string(), choice.label.to_string())),
                ));
            }
            AudioGeneratorControl::Number {
                value,
                presentation,
            } => {
                let control_path = format!("{path}/{}", presentation.field.field());
                section.add(
                    InspectorControl::new(
                        ControlKind::LayeredNumber,
                        &control_path,
                        presentation.label,
                    )
                    .value(
                        (presentation.display)(
                            value.value_at(runtime.local_time.unwrap_or(Time::ZERO)),
                        )
                        .to_string(),
                    )
                    .number(NumberSpec {
                        minimum: presentation.minimum,
                        maximum: presentation.maximum,
                        drag_step: presentation.drag_step,
                        digits: i32::try_from(presentation.digits)
                            .expect("audio generator digits must fit i32"),
                        unit: presentation.unit,
                    })
                    .store_multiplier(presentation.store_multiplier)
                    .layered(&control_path, LayeredState::from(value))
                    .timeline(
                        value.id,
                        crate::audio_modifiers::scalar_graph(value, runtime, presentation.display),
                    ),
                );
            }
        }
    }

    let default = AudioGenerator::default();
    InspectorItem::new("audio-generator", "Generator", section).reset(
        InspectorAction::ResetFields {
            values: [
                ("waveform", serde_json::to_value(default.waveform)),
                ("frequency_hz", serde_json::to_value(default.frequency_hz)),
                ("pulse_width", serde_json::to_value(default.pulse_width)),
                ("seed", serde_json::to_value(default.seed)),
            ]
            .into_iter()
            .map(|(field, value)| {
                (
                    format!("{path}/{field}"),
                    value.expect("default audio generator field must serialize"),
                )
            })
            .collect(),
        },
    )
}
