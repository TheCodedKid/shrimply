use serde_json::Value;
use shrimply_audio_modifiers::AudioModifier;
use shrimply_core::modifier_model::ModifierModel;
use shrimply_core::timeline_value::{Interpolation, TimelineBase, TimelineValue};
use shrimply_inspector_core::{AudioModifierControl, InspectorRuntime};
use shrimply_project::project::Time;

use crate::item::{HeaderAction, HeaderToggle, InspectorAction, InspectorItem, InspectorListItem};
use crate::section::{
    ControlKind, GraphPoint, GraphSegment, InspectorControl, InspectorSection, NumberSpec,
    ScalarGraph,
};

pub(crate) fn items(
    modifiers: &[Value],
    runtime: InspectorRuntime,
    can_paste: bool,
) -> Vec<InspectorListItem> {
    let mut items = modifiers
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let modifier: AudioModifier = serde_json::from_value(value.clone())
                .expect("audio modifier inspector value must be valid");
            modifier_item(&modifier, index, modifiers.len(), runtime).boxed()
        })
        .collect::<Vec<_>>();
    let catalog = shrimply_inspector_core::audio_modifier_catalog();
    let mut section = InspectorSection::default();
    section.add(
        InspectorControl::new(ControlKind::AudioModifierMenu, "", "")
            .value(can_paste.to_string())
            .choices(
                catalog.iter().map(|choice| choice.key.clone()).collect(),
                catalog
                    .iter()
                    .map(|choice| choice.label.to_string())
                    .collect(),
            )
            .choice_search_terms(
                catalog
                    .iter()
                    .map(|choice| choice.search_text.clone())
                    .collect(),
            ),
    );
    items.push(InspectorListItem::Flat(section));
    items
}

fn modifier_item(
    modifier: &AudioModifier,
    index: usize,
    len: usize,
    runtime: InspectorRuntime,
) -> InspectorItem {
    let mut section = InspectorSection::default();
    for control in shrimply_inspector_core::audio_modifier_controls(&modifier.effect) {
        match control {
            AudioModifierControl::Cache(cache) => {
                section.add(
                    InspectorControl::new(ControlKind::AudioCachePreset, "", "Format")
                        .target(modifier.id)
                        .value(shrimply_inspector_core::audio_cache_preset(&cache).key())
                        .choices(
                            shrimply_inspector_core::AudioCachePreset::OPTIONS
                                .iter()
                                .map(|(preset, _)| preset.key().to_string())
                                .collect(),
                            shrimply_inspector_core::AudioCachePreset::OPTIONS
                                .iter()
                                .map(|(_, label)| (*label).to_string())
                                .collect(),
                        ),
                );
                let cache = crate::audio_cache_control(modifier.id);
                section
                    .controls
                    .last_mut()
                    .expect("cache preset was added")
                    .sensitive = !cache.baking;
                section.add(
                    InspectorControl::new(ControlKind::AudioCache, "", "")
                        .target(modifier.id)
                        .value(cache.label)
                        .components(vec![cache.progress.to_string()])
                        .tooltip(cache.tooltip),
                );
            }
            AudioModifierControl::Scalar {
                path,
                label,
                value: timeline,
                presentation,
            } => scalar(&mut section, &path, label, &timeline, runtime, presentation),
            AudioModifierControl::Boolean { path, label, value } => section.add(
                InspectorControl::new(ControlKind::Boolean, path, label).value(value.to_string()),
            ),
            AudioModifierControl::Selector {
                path,
                label,
                value,
                options,
            } => section.add(crate::selector::selector(
                path,
                label,
                &value,
                options
                    .into_iter()
                    .map(|option| (option.value.to_string(), option.label.to_string())),
            )),
            AudioModifierControl::Number {
                path,
                label,
                value,
                minimum,
                maximum,
                step,
                digits,
            } => section.add(
                InspectorControl::new(ControlKind::Number, path, label)
                    .value(value.to_string())
                    .number(NumberSpec {
                        minimum,
                        maximum,
                        drag_step: step,
                        digits,
                        ..NumberSpec::default()
                    }),
            ),
            AudioModifierControl::VoiceModel { path, label, value } => {
                let models = crate::voice_models(&value);
                section.add(
                    crate::selector::selector(
                        path,
                        label,
                        &value,
                        models
                            .values
                            .iter()
                            .map(|model| (model.clone(), model.clone())),
                    )
                    .sensitive(!models.loading && models.error.is_none())
                    .tooltip(models.error.as_deref().unwrap_or(if models.loading {
                        "Loading Pneuma models…"
                    } else {
                        ""
                    })),
                );
            }
        }
    }
    for control in &mut section.controls {
        control.target_id = Some(modifier.id);
        control.audio_modifier = true;
    }

    InspectorItem::new(
        format!("audio-modifier:{}", modifier.id),
        modifier.effect.display_name(),
        section,
    )
    .reset(InspectorAction::ResetAudioModifier {
        id: modifier.id,
        effect: serde_json::to_value(shrimply_inspector_core::default_audio_modifier_effect(
            &modifier.effect,
        ))
        .expect("default audio modifier must serialize"),
    })
    .toggle(HeaderToggle {
        active: modifier.enabled,
        tooltip: if modifier.enabled {
            "Disable modifier"
        } else {
            "Enable modifier"
        },
        activate: InspectorAction::SetAudioModifierEnabled {
            id: modifier.id,
            enabled: !modifier.enabled,
        },
    })
    .actions(actions(modifier.id, index, len))
}

fn actions(id: uuid::Uuid, index: usize, len: usize) -> Vec<HeaderAction> {
    [
        ("edit-copy-symbolic", "Copy", 0_isize, true),
        ("go-up-symbolic", "Move up", -1, index > 0),
        ("go-down-symbolic", "Move down", 1, index + 1 < len),
        ("user-trash-symbolic", "Remove", 0, true),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(action, (icon, tooltip, offset, sensitive))| HeaderAction {
            icon,
            tooltip,
            sensitive,
            activate: match action {
                0 => InspectorAction::CopyAudioModifier { id },
                1 | 2 => InspectorAction::MoveAudioModifier { id, offset },
                3 => InspectorAction::RemoveAudioModifier { id },
                _ => unreachable!(),
            },
        },
    )
    .collect()
}

#[allow(clippy::too_many_arguments)]
fn scalar(
    section: &mut InspectorSection,
    path: &str,
    label: &str,
    timeline: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    presentation: shrimply_inspector_core::AudioModifierScalarPresentation,
) {
    section.add(
        InspectorControl::new(ControlKind::LayeredNumber, path, label)
            .value(
                (presentation.display)(timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO)))
                    .to_string(),
            )
            .number(NumberSpec {
                minimum: presentation.minimum,
                maximum: presentation.maximum,
                drag_step: presentation.drag_step,
                digits: i32::try_from(presentation.digits)
                    .expect("audio modifier scalar digits must fit i32"),
                unit: presentation.unit.unwrap_or_default(),
            })
            .store_multiplier(presentation.store_multiplier)
            .layered(path, shrimply_inspector_core::LayeredState::from(timeline))
            .timeline(
                timeline.id,
                scalar_graph(timeline, runtime, presentation.display),
            ),
    );
}

pub(crate) fn scalar_graph(
    timeline: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    display: fn(f32) -> f64,
) -> Option<ScalarGraph> {
    if !matches!(timeline.base, TimelineBase::Keyframes(_)) {
        return None;
    }
    let static_value = display(timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO)));
    let shrimply_keyframe_graph_ui::KeyframeGraph::RawValue {
        points, segments, ..
    } = shrimply_inspector_core::keyframe_model::scalar_graph(timeline, static_value, display)
    else {
        unreachable!("scalar timeline must produce a raw keyframe graph")
    };
    Some(ScalarGraph {
        points: points
            .into_iter()
            .map(|point| GraphPoint {
                time: point.time,
                value: point.value,
            })
            .collect(),
        segments: segments
            .into_iter()
            .map(|segment| GraphSegment {
                owner_id: segment.owner_id,
                start: segment.start,
                end: segment.end,
                start_value: segment.start_value,
                end_value: segment.end_value,
                interpolation: Interpolation::KEYFRAME
                    .iter()
                    .position(|candidate| *candidate == segment.interpolation)
                    .expect("scalar keyframe interpolation must be available"),
            })
            .collect(),
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}
