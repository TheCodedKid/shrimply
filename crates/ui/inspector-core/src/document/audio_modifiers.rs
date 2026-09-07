use super::{BasicInspectorAction as Action, InspectorItem};
use crate::item::HeaderToggle;
use crate::{
    AudioModifierControl, ControlKind, GraphPoint, GraphSegment, InspectorControl,
    InspectorRuntime, InspectorSection, NumberSpec, ScalarGraph,
};
use shrimply_audio_modifiers::AudioModifier;
use shrimply_core::{
    modifier_model::ModifierModel,
    timeline_value::{Interpolation, TimelineBase, TimelineValue},
};
use shrimply_project::project::Time;

pub(super) fn item(
    modifier: &AudioModifier,
    index: usize,
    len: usize,
    runtime: InspectorRuntime,
) -> InspectorItem {
    let mut section = InspectorSection::default();
    for control in crate::audio_modifier_controls(&modifier.effect) {
        match control {
            AudioModifierControl::Cache(cache) => {
                section.controls.extend(
                    crate::audio_cache_presentation(&cache, modifier.id)
                        .section
                        .controls,
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
                section
                    .add(InspectorControl::new(ControlKind::VoiceModel, path, label).value(value));
            }
        }
    }
    for control in &mut section.controls {
        control.target_id = Some(modifier.id);
        control.audio_modifier = true;
    }

    let mut item = InspectorItem::new(
        format!("audio-modifier:{}", modifier.id),
        modifier.effect.display_name(),
        section,
    )
    .reset(Action::ResetModifier {
        audio: true,
        id: modifier.id,
    })
    .toggle(HeaderToggle {
        active: modifier.enabled,
        tooltip: if modifier.enabled {
            "Disable modifier"
        } else {
            "Enable modifier"
        },
        activate: Action::SetModifierEnabled {
            audio: true,
            id: modifier.id,
            enabled: !modifier.enabled,
        },
    });
    item.actions =
        super::modifiers::chain_actions(modifier.id, true, index > 0, index + 1 < len, true);
    item
}

#[allow(clippy::too_many_arguments)]
fn scalar(
    section: &mut InspectorSection,
    path: &str,
    label: &str,
    timeline: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    presentation: crate::AudioModifierScalarPresentation,
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
            .layered(path, crate::LayeredState::from(timeline))
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
    let crate::keyframe_graph::KeyframeGraph::RawValue {
        points, segments, ..
    } = crate::keyframe_model::scalar_graph(timeline, static_value, display)
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
