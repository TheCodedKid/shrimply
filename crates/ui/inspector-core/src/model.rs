use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Number, Value};
use shrimply_audio_modifiers::{AudioModifier, AudioModifierEffect};
use shrimply_core::modifier_model::ModifierModel;
use shrimply_core::timeline_value::TimelineValue;
use shrimply_preview_core::PreviewTarget;
use shrimply_project::project::{
    AudioSource, Interpolation, ItemAddress, ItemMut, ItemRef, Project, Time, TrackMut, TrackRef,
    TransitionSide, VideoItemContent,
};
use shrimply_state::player_state::{self, SharedPlayerState};
use shrimply_timeline::selection_state::SharedSelectionState;
use std::{cell::RefCell, rc::Rc};

use crate::audio_modifiers::{
    audio_cache_status, audio_item_address, audio_modifier_evaluation_time, audio_modifier_key,
    audio_modifier_keyframe_time, audio_modifier_number, audio_modifier_number_mut,
    audio_modifier_time, audio_title,
};
use crate::target::InspectorTarget;

pub const INSPECTOR_MIN_WIDTH: i32 = 320;
const EXPRESSION_AUDIO_SAMPLE_RATE_HZ: u32 = 48_000;

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorSnapshot {
    pub target: InspectorTarget,
    pub title: String,
    pub value: Value,
    pub details: Vec<InspectorDetail>,
    pub media: Option<crate::info::InspectorMedia>,
    pub capabilities: InspectorCapabilities,
    pub runtime: InspectorRuntime,
    pub project: Option<crate::project::ProjectPresentation>,
    pub track: Option<crate::track::TrackPresentation>,
    pub transition: Option<crate::transition::TransitionPresentation>,
    pub video: Option<crate::video::VideoPresentation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectorRuntime {
    pub position: Time,
    pub local_time: Option<Time>,
    pub duration: Option<Time>,
    pub keyframe_range: Option<(Time, Time)>,
    pub frame_step: Time,
    pub keyframe_playhead: Option<Time>,
    pub frame_rate: shrimply_core::timeline_value::Fraction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorDetail {
    pub label: &'static str,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InspectorCapabilities {
    pub vector_transitions: bool,
    pub text: bool,
    pub drawing: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioCacheStatus {
    Missing,
    Baking { completed: u64, total: u64 },
    Ready,
    Failed(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioModifierChoice {
    pub key: String,
    pub label: &'static str,
    pub search_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorExpressionOutput<T = f32> {
    pub value: T,
    pub error: Option<String>,
}

pub struct TimelineModeChange<'a> {
    pub keyframes: bool,
    pub enabled: bool,
    pub current: Value,
    pub default_expression: &'a str,
}

pub struct AudioModifierKeyframeMove {
    pub old_time: Time,
    pub time: Time,
    pub displayed_value: f64,
    pub store_multiplier: f64,
}

#[derive(Clone)]
pub struct InspectorController {
    pub(crate) project: Rc<RefCell<Project>>,
    pub(crate) player_state: SharedPlayerState,
    selection_state: SharedSelectionState,
    pub(crate) keyframe_clipboard: Rc<RefCell<Option<crate::keyframe_model::KeyframeClipboard>>>,
    pub(crate) expression_cache: Rc<RefCell<shrimply_evaluation::TransformExpressionCache>>,
    pub(crate) audio_sampler: Rc<RefCell<shrimply_audio::streaming::FrameAudioSampler>>,
}

impl InspectorController {
    pub fn new(
        project: Rc<RefCell<Project>>,
        player_state: SharedPlayerState,
        selection_state: SharedSelectionState,
    ) -> Self {
        Self {
            project,
            player_state,
            selection_state,
            keyframe_clipboard: Rc::new(RefCell::new(None)),
            expression_cache: Rc::new(RefCell::new(Default::default())),
            audio_sampler: Rc::new(RefCell::new(
                shrimply_audio::streaming::FrameAudioSampler::preview(
                    EXPRESSION_AUDIO_SAMPLE_RATE_HZ,
                ),
            )),
        }
    }

    pub fn audio_sampler(&self) -> Rc<RefCell<shrimply_audio::streaming::FrameAudioSampler>> {
        self.audio_sampler.clone()
    }

    pub fn audio_analysis_at(&self, position: Time) -> shrimply_evaluation::FrameAudioAnalysis {
        let revision = player_state::snapshot(&self.player_state).revision;
        self.audio_sampler
            .borrow_mut()
            .sample(&self.project.borrow(), position, revision)
    }

    pub fn target(&self) -> InspectorTarget {
        crate::target::resolve(&self.project.borrow(), &self.selection_state)
    }

    pub fn refresh_video(&self) {
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                video: true,
                ..player_state::ProjectChange::default()
            },
        );
    }

    pub fn valid_preview_focus(
        &self,
        focused_item: &ItemAddress,
        focused_target: PreviewTarget,
        current_item: &ItemAddress,
    ) -> bool {
        self.project
            .borrow()
            .video_item(current_item)
            .is_some_and(|video| {
                crate::item::valid_preview_focus(focused_item, focused_target, current_item, video)
            })
    }

    pub fn snapshot(&self) -> InspectorSnapshot {
        let project = self.project.borrow();
        let target = crate::target::resolve(&project, &self.selection_state);
        let track = match &target {
            InspectorTarget::Track(address) => Some(
                crate::track::presentation(&project, address.clone())
                    .expect("resolved track inspector target must have a presentation"),
            ),
            _ => None,
        };
        let transition = match &target {
            InspectorTarget::Transition { item, side } => Some(
                crate::transition::presentation(&project, item, *side)
                    .expect("resolved transition inspector target must have a presentation"),
            ),
            _ => None,
        };
        let runtime = target_runtime(&project, &self.player_state, &target);
        let video = match &target {
            InspectorTarget::Item(address @ ItemAddress::Video { .. }) => {
                Some(crate::video::presentation(
                    &project,
                    address,
                    project
                        .video_item(address)
                        .expect("resolved video inspector target must remain available"),
                    runtime,
                ))
            }
            _ => None,
        };
        let (title, value) = match (&track, &transition, &video) {
            (Some(track), _, _) => (track.title().to_string(), Value::Null),
            (_, Some(transition), _) => (transition.title.to_string(), transition.value.clone()),
            (_, _, Some(video)) => (video.title.to_string(), video.value.clone()),
            (None, None, None) if matches!(&target, InspectorTarget::Project) => {
                ("Project".to_string(), Value::Null)
            }
            (None, None, None) => target_value(&project, &target).expect(
                "resolved inspector target must remain available while the project is borrowed",
            ),
        };
        InspectorSnapshot {
            capabilities: transition
                .as_ref()
                .map(|transition| transition.capabilities)
                .unwrap_or_default(),
            details: target_details(&project, &target, track.as_ref()),
            media: crate::info::target_media(&project, &target),
            runtime,
            project: matches!(&target, InspectorTarget::Project)
                .then(|| crate::project::presentation(&project)),
            track,
            transition,
            video,
            target,
            title,
            value,
        }
    }

    pub(crate) fn set_regular_field(
        &self,
        target: &InspectorTarget,
        path: &str,
        text: &str,
    ) -> Result<(), String> {
        if path == "/name" && matches!(target, InspectorTarget::Project) {
            self.set_project_name(text);
            return Ok(());
        }
        if path == "/track_id" && matches!(target, InspectorTarget::Item(ItemAddress::Video { .. }))
        {
            let track_id = text
                .parse::<u32>()
                .map_err(|_| format!("invalid video stream: {text}"))?;
            return self.set_video_stream(target, track_id);
        }
        if path == "/kind" && matches!(target, InspectorTarget::Transition { .. }) {
            return self.set_transition_kind(target, text);
        }
        let kind = if asset_path(path) {
            EditKind::Asset
        } else if structural_path(path) {
            EditKind::Structural
        } else {
            EditKind::Live
        };
        self.set_field_with_kind(target, path, text, kind)
    }

    fn set_field_with_kind(
        &self,
        target: &InspectorTarget,
        path: &str,
        text: &str,
        kind: EditKind,
    ) -> Result<(), String> {
        self.edit_value_if_changed_with_refresh(
            target,
            kind,
            duration_path(path),
            crate::refresh::audio_path_change(target, path, kind),
            |value| {
                let current = value
                    .pointer_mut(path)
                    .ok_or_else(|| format!("inspector field is no longer available: {path}"))?;
                if !editable_path(path) {
                    return Err(format!("inspector field is read-only: {path}"));
                }
                let replacement = parsed_value(current, text)?;
                if *current == replacement {
                    return Ok(false);
                }
                *current = replacement;
                Ok(true)
            },
        )
    }

    pub fn set_components(
        &self,
        target: &InspectorTarget,
        path: &str,
        components: &[(usize, String)],
    ) -> Result<(), String> {
        self.edit_value(target, EditKind::Live, false, |value| {
            let current = value
                .pointer_mut(path)
                .ok_or_else(|| format!("inspector field is no longer available: {path}"))?;
            for (component, text) in components {
                let next = text
                    .parse::<Number>()
                    .map(Value::Number)
                    .map_err(|_| format!("invalid numeric inspector value: {text}"))?;
                match current {
                    Value::Array(values) => {
                        *values.get_mut(*component).ok_or_else(|| {
                            "inspector component is no longer available".to_string()
                        })? = next;
                    }
                    Value::Object(values) => {
                        let keys = vector_keys(values).ok_or_else(|| {
                            "inspector value is not a vector or color".to_string()
                        })?;
                        *values
                            .get_mut(*keys.get(*component).ok_or_else(|| {
                                "inspector component is no longer available".to_string()
                            })?)
                            .expect("inspector component key must exist") = next;
                    }
                    _ => return Err("inspector value is not a vector or color".to_string()),
                }
            }
            Ok(())
        })
    }

    pub fn set_fraction(
        &self,
        target: &InspectorTarget,
        path: &str,
        value: shrimply_math_core::Fraction,
    ) -> Result<(), String> {
        if !shrimply_math_core::fraction_is_finite(value) {
            return Err("inspector fraction must be finite".to_string());
        }
        self.edit_value_if_changed_with_refresh(
            target,
            EditKind::Live,
            duration_path(path),
            crate::refresh::audio_path_change(target, path, EditKind::Live),
            |root| {
                let current = root
                    .pointer_mut(path)
                    .ok_or_else(|| format!("inspector time is no longer available: {path}"))?;
                *current = serde_json::json!({
                    "numerator": shrimply_math_core::fraction_numerator(value),
                    "denominator": shrimply_math_core::fraction_denominator(value),
                });
                Ok(true)
            },
        )
    }

    pub fn set_timeline_mode(
        &self,
        target: &InspectorTarget,
        path: &str,
        keyframes: bool,
        enabled: bool,
        current: Value,
        default_expression: &str,
    ) -> Result<(), String> {
        self.set_timeline_mode_with_kind(
            target,
            path,
            TimelineModeChange {
                keyframes,
                enabled,
                current,
                default_expression,
            },
            EditKind::Structural,
        )
    }

    fn set_timeline_mode_with_kind(
        &self,
        target: &InspectorTarget,
        path: &str,
        change: TimelineModeChange<'_>,
        kind: EditKind,
    ) -> Result<(), String> {
        let local_time = if change.keyframes && change.enabled {
            let InspectorTarget::Item(address) = target else {
                return Err("keyframes require an item inspector target".to_string());
            };
            let project = self.project.borrow();
            let position = player_state::snapshot(&self.player_state).position;
            project
                .keyframe_time(address, position)
                .ok_or_else(|| "the current item time is not available".to_string())?
        } else {
            Time::ZERO
        };
        self.edit_value_if_changed_with_refresh(
            target,
            kind,
            false,
            crate::refresh::audio_path_change(target, path, kind),
            |root| {
                let timeline = root
                    .pointer_mut(path)
                    .and_then(Value::as_object_mut)
                    .ok_or_else(|| {
                        format!("inspector timeline value is no longer available: {path}")
                    })?;
                if change.keyframes {
                    let base = timeline
                        .get_mut("base")
                        .and_then(Value::as_object_mut)
                        .ok_or_else(|| "inspector timeline base is invalid".to_string())?;
                    let currently_enabled = base.contains_key("keyframes");
                    if currently_enabled == change.enabled {
                        return Ok(false);
                    }
                    *base = if change.enabled {
                        serde_json::json!({
                            "keyframes": [{
                                "id": uuid::Uuid::new_v4(),
                                "time": local_time,
                                "value": change.current,
                                "interpolation_to_next": Interpolation::default(),
                            }]
                        })
                        .as_object()
                        .expect("timeline keyframe base must be an object")
                        .clone()
                    } else {
                        serde_json::json!({ "const": change.current })
                            .as_object()
                            .expect("timeline constant base must be an object")
                            .clone()
                    };
                    return Ok(true);
                }
                match timeline.get_mut("expression") {
                    Some(Value::Object(expression)) => {
                        let current = expression
                            .get("enabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        if current == change.enabled {
                            return Ok(false);
                        }
                        expression.insert("enabled".to_string(), Value::Bool(change.enabled));
                    }
                    Some(expression @ Value::Null) if change.enabled => {
                        *expression = serde_json::json!({
                            "id": uuid::Uuid::new_v4(),
                            "enabled": true,
                            "source": change.default_expression,
                        });
                    }
                    Some(Value::Null) => return Ok(false),
                    Some(_) => return Err("inspector timeline expression is invalid".to_string()),
                    None if change.enabled => {
                        timeline.insert(
                            "expression".to_string(),
                            serde_json::json!({
                                "id": uuid::Uuid::new_v4(),
                                "enabled": true,
                                "source": change.default_expression,
                            }),
                        );
                    }
                    None => return Ok(false),
                }
                Ok(true)
            },
        )
    }

    pub fn set_expression_source(
        &self,
        target: &InspectorTarget,
        path: &str,
        source: &str,
    ) -> Result<(), String> {
        self.set_expression_source_with_kind(target, path, source, EditKind::Live)
    }

    fn set_expression_source_with_kind(
        &self,
        target: &InspectorTarget,
        path: &str,
        source: &str,
        kind: EditKind,
    ) -> Result<(), String> {
        self.edit_value_if_changed_with_refresh(
            target,
            kind,
            false,
            (kind == EditKind::Live)
                .then(|| crate::refresh::audio_scalar_expression_change(target))
                .flatten(),
            |root| {
                let expression = root
                    .pointer_mut(path)
                    .and_then(|timeline| timeline.get_mut("expression"))
                    .and_then(Value::as_object_mut)
                    .ok_or_else(|| "inspector timeline expression is not enabled".to_string())?;
                if expression.get("source").and_then(Value::as_str) == Some(source) {
                    return Ok(false);
                }
                expression.insert("source".to_string(), Value::String(source.to_string()));
                Ok(true)
            },
        )
    }

    pub fn set_timeline_base(
        &self,
        target: &InspectorTarget,
        path: &str,
        replacement: Value,
    ) -> Result<(), String> {
        self.set_timeline_base_with_kind(target, path, replacement, EditKind::Live)
    }

    fn set_timeline_base_with_kind(
        &self,
        target: &InspectorTarget,
        path: &str,
        replacement: Value,
        kind: EditKind,
    ) -> Result<(), String> {
        let local_time = {
            let project = self.project.borrow();
            let value = target_value(&project, target)
                .ok_or_else(|| "inspector target is no longer available".to_string())?
                .1;
            let keyframed = value
                .pointer(path)
                .and_then(|timeline| timeline.get("base"))
                .and_then(|base| base.get("keyframes"))
                .is_some();
            if keyframed {
                let InspectorTarget::Item(address) = target else {
                    return Err("keyframes require an item inspector target".to_string());
                };
                let position = player_state::snapshot(&self.player_state).position;
                Some((
                    project
                        .keyframe_time(address, position)
                        .ok_or_else(|| "the current item time is not available".to_string())?,
                    target_runtime(&project, &self.player_state, target).frame_step,
                ))
            } else {
                None
            }
        };
        self.edit_value_if_changed_with_refresh(
            target,
            kind,
            false,
            crate::refresh::audio_path_change(target, path, kind),
            |root| {
                let base = root
                    .pointer_mut(path)
                    .and_then(|timeline| timeline.get_mut("base"))
                    .and_then(Value::as_object_mut)
                    .ok_or_else(|| {
                        format!("inspector timeline value is no longer available: {path}")
                    })?;
                let Some((local_time, frame_step)) = local_time else {
                    let current = base
                        .get_mut("const")
                        .ok_or_else(|| "inspector timeline constant is invalid".to_string())?;
                    if *current == replacement {
                        return Ok(false);
                    }
                    *current = replacement;
                    return Ok(true);
                };
                let keyframes = base
                    .get_mut("keyframes")
                    .and_then(Value::as_array_mut)
                    .ok_or_else(|| "inspector timeline keyframes are invalid".to_string())?;
                let keyframe_times = keyframes
                    .iter()
                    .map(|keyframe| {
                        keyframe
                            .get("time")
                            .cloned()
                            .ok_or_else(|| "inspector keyframe time is missing".to_string())
                            .and_then(|time| {
                                serde_json::from_value::<Time>(time).map_err(|error| {
                                    format!("inspector keyframe time is invalid: {error}")
                                })
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(index) = keyframe_times.iter().position(|keyframe_time| {
                    crate::keyframe_model::same_frame(*keyframe_time, local_time, frame_step)
                }) {
                    let keyframe = &mut keyframes[index];
                    let changed = keyframe.get("value") != Some(&replacement)
                        || keyframe_times[index] != local_time;
                    let keyframe = keyframe
                        .as_object_mut()
                        .ok_or_else(|| "inspector keyframe is invalid".to_string())?;
                    keyframe.insert("value".to_string(), replacement);
                    keyframe.insert("time".to_string(), serialize(&local_time));
                    crate::keyframe_model::sort_json_keyframes(keyframes)?;
                    return Ok(changed);
                }
                keyframes.push(serde_json::json!({
                    "id": uuid::Uuid::new_v4(),
                    "time": local_time,
                    "value": replacement,
                    "interpolation_to_next": Interpolation::default(),
                }));
                crate::keyframe_model::sort_json_keyframes(keyframes)?;
                Ok(true)
            },
        )
    }

    pub fn apply_project_settings(
        &self,
        canvas_size: shrimply_project::project::CanvasSize,
        frame_rate: shrimply_math_core::Fraction,
    ) -> Result<(), String> {
        if !(crate::project::MIN_CANVAS_DIMENSION..=crate::project::MAX_CANVAS_DIMENSION)
            .contains(&canvas_size.width)
            || !(crate::project::MIN_CANVAS_DIMENSION..=crate::project::MAX_CANVAS_DIMENSION)
                .contains(&canvas_size.height)
        {
            return Err(format!(
                "project dimensions must be between {} and {}",
                crate::project::MIN_CANVAS_DIMENSION,
                crate::project::MAX_CANVAS_DIMENSION,
            ));
        }
        if !shrimply_math_core::fraction_is_finite(frame_rate)
            || shrimply_math_core::fraction_numerator(frame_rate) <= 0
        {
            return Err("project frame rate must be positive".to_string());
        }
        let mut project = self.project.borrow_mut();
        let frame_rate_changed = project.fps != frame_rate;
        let resolution_changed = project.canvas_size != canvas_size;
        if !frame_rate_changed && !resolution_changed {
            return Ok(());
        }
        project.fps = frame_rate;
        project.canvas_size = canvas_size;
        shrimply_project::project::commit_edit(&project, "project-settings");
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                frame_rate: frame_rate_changed.then_some(frame_rate),
                video: true,
                captions: true,
                inspector: true,
                ..Default::default()
            },
        );
        Ok(())
    }

    pub fn set_audio_modifier_enabled(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let address = audio_item_address(target)?;
        let modifier = project
            .audio_item_mut(address)
            .and_then(|item| item.modifiers.iter_mut().find(|modifier| modifier.id == id))
            .ok_or_else(|| "audio modifier is no longer available".to_string())?;
        if modifier.enabled == enabled {
            return Ok(());
        }
        modifier.enabled = enabled;
        shrimply_project::project::commit_edit(&project, "toggle-audio-modifier");
        drop(project);
        self.refresh_audio_inspector();
        Ok(())
    }

    pub fn set_audio_modifier_field(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        path: &str,
        text: &str,
    ) -> Result<(), String> {
        let path = self.audio_modifier_path(target, id, path)?;
        self.set_field_with_kind(target, &path, text, EditKind::AudioModifierStructural)
    }

    pub fn set_audio_modifier_live_field(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        path: &str,
        text: &str,
    ) -> Result<(), String> {
        let path = self.audio_modifier_path(target, id, path)?;
        self.set_field_with_kind(target, &path, text, EditKind::AudioModifierLive)
    }

    pub fn set_audio_modifier_timeline_base(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        next: f32,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let time = audio_modifier_time(&project, &self.player_state, target)?;
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        if !crate::keyframe_model::set_scalar_value(value, time, next) {
            return Ok(());
        }
        shrimply_project::project::commit_coalesced_edit(&project, "audio-modifier-value");
        drop(project);
        self.refresh_audio_modifier(false, false, true);
        Ok(())
    }

    pub fn timeline_number_value(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<f64, String> {
        let project = self.project.borrow();
        let value = target_value(&project, target)
            .ok_or_else(|| "inspector target is no longer available".to_string())?
            .1;
        let timeline: shrimply_core::timeline_value::TimelineValue<f32> =
            deserialize(value.pointer(path).cloned().ok_or_else(|| {
                format!("inspector timeline value is no longer available: {path}")
            })?)?;
        let local_time = match target {
            InspectorTarget::Item(ItemAddress::Audio { .. }) => {
                audio_modifier_evaluation_time(&project, &self.player_state, target)?
            }
            InspectorTarget::Item(address) => {
                let position = player_state::snapshot(&self.player_state).position;
                project
                    .keyframe_time(address, position)
                    .ok_or_else(|| "the current item time is not available".to_string())?
            }
            _ => Time::ZERO,
        };
        Ok(f64::from(timeline.value_at(local_time)))
    }

    pub fn audio_modifier_number_value(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        value_id: uuid::Uuid,
    ) -> Result<f64, String> {
        let project = self.project.borrow();
        let local_time = audio_modifier_evaluation_time(&project, &self.player_state, target)?;
        let value = audio_modifier_number(&project, target, id, value_id)?;
        Ok(f64::from(value.value_at(local_time)))
    }

    pub fn audio_modifier_timeline(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        value_id: uuid::Uuid,
    ) -> Result<TimelineValue<f32>, String> {
        let project = self.project.borrow();
        audio_modifier_number(&project, target, id, value_id).cloned()
    }

    pub fn audio_modifier_expression_output(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
    ) -> Result<InspectorExpressionOutput, String> {
        let project = self.project.borrow();
        let local_time = audio_modifier_evaluation_time(&project, &self.player_state, target)?;
        let item = project
            .audio_item(audio_item_address(target)?)
            .ok_or_else(|| "audio modifier item is no longer available".to_string())?;
        let value = item
            .modifiers
            .iter()
            .find(|modifier| modifier.id == modifier_id)
            .and_then(|modifier| modifier.effect.number(value_id))
            .ok_or_else(|| "audio modifier number is no longer available".to_string())?;
        let evaluation = shrimply_evaluation::VisualEvaluation::for_audio_item_local_time(
            &project, item, local_time,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub fn set_audio_modifier_timeline_mode(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        path: &str,
        change: TimelineModeChange<'_>,
    ) -> Result<(), String> {
        if change.keyframes {
            return self.set_audio_modifier_keyframes_enabled(
                target,
                modifier_id,
                value_id,
                change.enabled,
            );
        }
        self.set_audio_modifier_expression_enabled(
            target,
            modifier_id,
            path,
            change.enabled,
            change.default_expression,
        )
    }

    pub fn set_audio_modifier_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let evaluation_time = audio_modifier_evaluation_time(&project, &self.player_state, target)?;
        let keyframe_time = audio_modifier_time(&project, &self.player_state, target)?;
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        let current = value.value_at(evaluation_time);
        if !shrimply_core::timeline_value::set_keyframes_enabled(
            value,
            keyframe_time,
            current,
            enabled,
        ) {
            return Ok(());
        }
        shrimply_project::project::commit_edit(&project, "audio-modifier-value");
        drop(project);
        self.refresh_audio_modifier(true, true, false);
        Ok(())
    }

    pub fn set_audio_modifier_expression_enabled(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        path: &str,
        enabled: bool,
        default_expression: &str,
    ) -> Result<(), String> {
        self.set_timeline_mode_with_kind(
            target,
            &self.audio_modifier_path(target, modifier_id, path)?,
            TimelineModeChange {
                keyframes: false,
                enabled,
                current: Value::Null,
                default_expression,
            },
            EditKind::AudioModifierStructural,
        )
    }

    pub fn set_audio_modifier_expression_source(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        path: &str,
        source: &str,
    ) -> Result<(), String> {
        self.set_expression_source_with_kind(
            target,
            &self.audio_modifier_path(target, id, path)?,
            source,
            EditKind::AudioModifierLive,
        )
    }

    pub fn move_audio_modifier_keyframe(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        change: AudioModifierKeyframeMove,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let time = audio_modifier_keyframe_time(&project, target, change.time)?;
        let next = (change.displayed_value * change.store_multiplier) as f32;
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        if !crate::keyframe_model::update_scalar_keyframe(value, change.old_time, time, next) {
            return Ok(());
        }
        shrimply_project::project::commit_coalesced_edit(&project, "audio-modifier-keyframe");
        drop(project);
        self.refresh_audio_modifier_graph();
        Ok(())
    }

    pub fn delete_audio_modifier_keyframe(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        time: Time,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        if !crate::keyframe_model::delete_scalar_keyframe(value, time) {
            return Ok(());
        }
        let keyframes_disabled = !matches!(
            value.base,
            shrimply_core::timeline_value::TimelineBase::Keyframes(_)
        );
        shrimply_project::project::commit_edit(&project, "audio-modifier-keyframe");
        drop(project);
        self.refresh_audio_modifier(keyframes_disabled, true, true);
        Ok(())
    }

    pub fn add_audio_modifier_keyframe(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        time: Time,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let time = audio_modifier_keyframe_time(&project, target, time)?;
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        if !crate::keyframe_model::add_scalar_keyframe(value, time) {
            return Ok(());
        }
        shrimply_project::project::commit_edit(&project, "audio-modifier-keyframe");
        drop(project);
        self.refresh_audio_modifier_graph();
        Ok(())
    }

    pub fn set_audio_modifier_keyframe_interpolation(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
    ) -> Result<(), String> {
        let interpolation = Interpolation::KEYFRAME
            .get(interpolation_index)
            .copied()
            .ok_or_else(|| "audio modifier keyframe interpolation is invalid".to_string())?;
        let mut project = self.project.borrow_mut();
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        if !crate::keyframe_model::set_scalar_interpolation(value, owner_id, interpolation) {
            return Ok(());
        }
        shrimply_project::project::commit_edit(&project, "audio-modifier-keyframe");
        drop(project);
        self.refresh_audio_modifier_graph();
        Ok(())
    }

    pub fn seek_audio_modifier_keyframe(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<(), String> {
        let project = self.project.borrow();
        let position = project
            .keyframe_timeline_time(audio_item_address(target)?, time)
            .ok_or_else(|| "audio modifier keyframe time is no longer available".to_string())?;
        drop(project);
        player_state::seek_time(&self.player_state, position);
        Ok(())
    }

    pub fn toggle_keyframe_playback(&self) {
        player_state::toggle_playing(&self.player_state);
    }

    pub fn copy_audio_modifier_keyframes(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        selected: &[Time],
    ) -> Result<usize, String> {
        let project = self.project.borrow();
        let value = audio_modifier_number(&project, target, modifier_id, value_id)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let address = audio_item_address(target)?;
        let timeline_times = clipboard
            .times
            .iter()
            .map(|time| {
                project
                    .keyframe_timeline_time(address, *time)
                    .unwrap_or(*time)
                    .snapped(project.frame_step())
            })
            .collect::<Vec<_>>();
        let Some(origin) = timeline_times.first().copied() else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        clipboard.times = timeline_times
            .into_iter()
            .map(|time| Time {
                seconds: time.seconds - origin.seconds,
            })
            .collect();
        let count = clipboard.len();
        self.keyframe_clipboard.replace(Some(clipboard));
        Ok(count)
    }

    pub fn paste_audio_modifier_keyframes(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        value_id: uuid::Uuid,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let mut project = self.project.borrow_mut();
        let address = audio_item_address(target)?;
        let anchor = project
            .keyframe_timeline_time(address, time)
            .unwrap_or(time)
            .snapped(project.frame_step());
        let times = clipboard
            .times
            .iter()
            .filter_map(|offset| {
                project.keyframe_time(
                    address,
                    Time {
                        seconds: anchor.seconds + offset.seconds,
                    },
                )
            })
            .collect::<Vec<_>>();
        if times.len() != clipboard.len() {
            return Err("audio modifier keyframes cannot be pasted at this time".to_string());
        }
        let value = audio_modifier_number_mut(&mut project, target, modifier_id, value_id)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(value, &clipboard, &times) else {
            return Ok(0);
        };
        shrimply_project::project::commit_edit(&project, "audio-modifier-keyframe");
        drop(project);
        self.refresh_audio_modifier(true, true, true);
        Ok(pasted.len())
    }

    fn audio_modifier_path(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        relative_path: &str,
    ) -> Result<String, String> {
        if !relative_path.starts_with('/') {
            return Err("audio modifier path must be relative to the modifier".to_string());
        }
        let project = self.project.borrow();
        let index = project
            .audio_item(audio_item_address(target)?)
            .and_then(|item| item.modifiers.iter().position(|modifier| modifier.id == id))
            .ok_or_else(|| "audio modifier is no longer available".to_string())?;
        Ok(format!("/modifiers/{index}{relative_path}"))
    }

    pub fn reset_audio_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        effect: Value,
    ) -> Result<(), String> {
        let effect: AudioModifierEffect = deserialize(effect)?;
        self.reset_audio_modifier_effect(target, id, effect)
    }

    pub fn reset_audio_modifier_effect(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        effect: AudioModifierEffect,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let modifier = project
            .audio_item_mut(audio_item_address(target)?)
            .and_then(|item| item.modifiers.iter_mut().find(|modifier| modifier.id == id))
            .ok_or_else(|| "audio modifier is no longer available".to_string())?;
        if crate::audio_modifiers::same_audio_modifier_effect(&modifier.effect, &effect) {
            return Ok(());
        }
        modifier.effect = effect;
        shrimply_project::project::commit_edit(&project, "reset-audio-modifier");
        drop(project);
        self.refresh_audio_inspector();
        Ok(())
    }

    pub fn copy_audio_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> Result<String, String> {
        let project = self.project.borrow();
        let modifier = project
            .audio_item(audio_item_address(target)?)
            .and_then(|item| item.modifiers.iter().find(|modifier| modifier.id == id))
            .ok_or_else(|| "audio modifier is no longer available".to_string())?;
        let name = modifier.effect.display_name().to_string();
        clipboard.borrow_mut().copy_audio_modifier(modifier);
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                inspector: true,
                ..Default::default()
            },
        );
        Ok(name)
    }

    pub fn move_audio_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        offset: isize,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let item = project
            .audio_item_mut(audio_item_address(target)?)
            .ok_or_else(|| "audio item is no longer available".to_string())?;
        let index = item
            .modifiers
            .iter()
            .position(|modifier| modifier.id == id)
            .ok_or_else(|| "audio modifier is no longer available".to_string())?;
        let destination = index
            .checked_add_signed(offset)
            .filter(|destination| *destination < item.modifiers.len())
            .ok_or_else(|| "audio modifier cannot be moved outside the chain".to_string())?;
        item.modifiers.swap(index, destination);
        shrimply_project::project::commit_edit(&project, "edit-audio-modifier-chain");
        drop(project);
        self.refresh_audio_inspector();
        Ok(())
    }

    pub fn remove_audio_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let item = project
            .audio_item_mut(audio_item_address(target)?)
            .ok_or_else(|| "audio item is no longer available".to_string())?;
        let index = item
            .modifiers
            .iter()
            .position(|modifier| modifier.id == id)
            .ok_or_else(|| "audio modifier is no longer available".to_string())?;
        if matches!(item.modifiers[index].effect, AudioModifierEffect::Cache(_)) {
            shrimply_audio::modifier_cache::invalidate(id)?;
        }
        item.modifiers.remove(index);
        shrimply_project::project::commit_edit(&project, "edit-audio-modifier-chain");
        drop(project);
        self.refresh_audio_inspector();
        Ok(())
    }

    pub fn add_audio_modifier(&self, target: &InspectorTarget, kind: &str) -> Result<(), String> {
        let effect = AudioModifierEffect::CATALOG
            .iter()
            .map(|new| new())
            .find(|effect| audio_modifier_key(effect) == kind)
            .ok_or_else(|| format!("unknown audio modifier: {kind}"))?;
        let mut project = self.project.borrow_mut();
        project
            .audio_item_mut(audio_item_address(target)?)
            .ok_or_else(|| "audio item is no longer available".to_string())?
            .modifiers
            .push(AudioModifier::new(effect));
        shrimply_project::project::commit_edit(&project, "add-audio-modifier");
        drop(project);
        self.refresh_audio_inspector();
        Ok(())
    }

    pub fn can_paste_audio_modifiers(
        &self,
        target: &InspectorTarget,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> bool {
        let Ok(address) = audio_item_address(target) else {
            return false;
        };
        clipboard
            .borrow()
            .can_append_modifiers(&self.project.borrow(), std::slice::from_ref(address))
    }

    pub fn paste_audio_modifiers(
        &self,
        target: &InspectorTarget,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> Result<usize, String> {
        let address = audio_item_address(target)?.clone();
        let mut project = self.project.borrow_mut();
        let result = clipboard
            .borrow()
            .append_modifiers(&mut project, std::slice::from_ref(&address));
        if !result.changed {
            return Ok(0);
        }
        shrimply_project::project::commit_edit(&project, "paste-item-modifiers");
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                audio: result.audio,
                audio_waveforms: result.audio_waveforms,
                inspector: true,
                ..Default::default()
            },
        );
        Ok(result.modifiers_added)
    }

    pub fn set_audio_cache_preset(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        preset: &str,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let cache = project
            .audio_item_mut(audio_item_address(target)?)
            .and_then(|item| item.modifiers.iter_mut().find(|modifier| modifier.id == id))
            .and_then(|modifier| match &mut modifier.effect {
                AudioModifierEffect::Cache(cache) => Some(cache),
                _ => None,
            })
            .ok_or_else(|| "audio cache modifier is no longer available".to_string())?;
        let preset = crate::AudioCachePreset::from_key(preset)
            .ok_or_else(|| format!("unknown audio cache preset: {preset}"))?;
        if !preset.apply(cache) {
            return Ok(());
        }
        shrimply_project::project::commit_edit(&project, "audio-cache-format");
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                inspector: true,
                ..Default::default()
            },
        );
        Ok(())
    }

    pub fn toggle_audio_cache(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
    ) -> Result<(), String> {
        if matches!(audio_cache_status(id), AudioCacheStatus::Baking { .. }) {
            shrimply_audio::modifier_cache::invalidate(id)?;
        } else {
            shrimply_audio::modifier_cache::bake(
                self.project.borrow().clone(),
                audio_item_address(target)?.clone(),
                id,
            )?;
        }
        self.refresh_audio_inspector();
        Ok(())
    }

    pub fn refresh_audio_cache(&self) {
        self.refresh_audio_inspector();
    }

    fn refresh_audio_inspector(&self) {
        self.refresh_audio_modifier(true, true, false);
    }

    fn refresh_audio_modifier_graph(&self) {
        self.refresh_audio_modifier(false, true, true);
    }

    fn refresh_audio_modifier(&self, inspector: bool, audio_waveforms: bool, live_preview: bool) {
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                audio: true,
                audio_waveforms,
                inspector,
                live_preview,
                ..Default::default()
            },
        );
    }

    pub fn copy_array_item(
        &self,
        target: &InspectorTarget,
        path: &str,
        index: usize,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> Result<(), String> {
        if path != "/modifiers" {
            return Err("this inspector list does not support copying".to_string());
        }
        let project = self.project.borrow();
        match target {
            InspectorTarget::Item(address @ ItemAddress::Video { .. }) => {
                let modifier = project
                    .video_item(address)
                    .and_then(|item| item.modifiers.get(index))
                    .ok_or_else(|| "visual modifier is no longer available".to_string())?;
                clipboard.borrow_mut().copy_visual_modifier(modifier);
            }
            InspectorTarget::Item(address @ ItemAddress::Audio { .. }) => {
                let modifier = project
                    .audio_item(address)
                    .and_then(|item| item.modifiers.get(index))
                    .ok_or_else(|| "audio modifier is no longer available".to_string())?;
                clipboard.borrow_mut().copy_audio_modifier(modifier);
            }
            _ => return Err("only audio and visual modifiers can be copied".to_string()),
        }
        Ok(())
    }

    pub fn move_array_item(
        &self,
        target: &InspectorTarget,
        path: &str,
        index: usize,
        offset: isize,
    ) -> Result<(), String> {
        self.edit_array(target, path, |values| {
            let destination = index.checked_add_signed(offset).ok_or_else(|| {
                "inspector array item cannot be moved outside the list".to_string()
            })?;
            if index >= values.len() || destination >= values.len() {
                return Err("inspector array item is no longer available".to_string());
            }
            values.swap(index, destination);
            Ok(())
        })
    }

    pub fn remove_array_item(
        &self,
        target: &InspectorTarget,
        path: &str,
        index: usize,
    ) -> Result<(), String> {
        self.edit_array(target, path, |values| {
            if index >= values.len() {
                return Err("inspector array item is no longer available".to_string());
            }
            values.remove(index);
            Ok(())
        })
    }

    fn edit_array(
        &self,
        target: &InspectorTarget,
        path: &str,
        edit: impl FnOnce(&mut Vec<Value>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.edit_value_if_changed(target, EditKind::Structural, false, |root| {
            let values = root
                .pointer_mut(path)
                .and_then(Value::as_array_mut)
                .ok_or_else(|| "inspector value is not an array".to_string())?;
            edit(values)?;
            Ok(true)
        })
    }

    fn edit_value(
        &self,
        target: &InspectorTarget,
        kind: EditKind,
        affects_duration: bool,
        edit: impl FnOnce(&mut Value) -> Result<(), String>,
    ) -> Result<(), String> {
        self.edit_value_if_changed(target, kind, affects_duration, |value| {
            edit(value)?;
            Ok(true)
        })
    }

    pub(crate) fn edit_value_if_changed(
        &self,
        target: &InspectorTarget,
        kind: EditKind,
        affects_duration: bool,
        edit: impl FnOnce(&mut Value) -> Result<bool, String>,
    ) -> Result<(), String> {
        self.edit_value_if_changed_with_refresh(target, kind, affects_duration, None, edit)
    }

    pub(crate) fn edit_value_if_changed_with_refresh(
        &self,
        target: &InspectorTarget,
        kind: EditKind,
        affects_duration: bool,
        refresh: Option<player_state::ProjectChange>,
        edit: impl FnOnce(&mut Value) -> Result<bool, String>,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let mut value = target_value(&project, target)
            .ok_or_else(|| "inspector target is no longer available".to_string())?
            .1;
        let original = value.clone();
        if !edit(&mut value)? {
            return Ok(());
        }
        replace_target(&mut project, target, value)?;
        if kind == EditKind::Asset
            && let Err(error) = project.watch_assets()
        {
            replace_target(&mut project, target, original)
                .expect("the previous inspector value must remain valid");
            project
                .watch_assets()
                .expect("restoring the previous inspector assets must succeed");
            return Err(format!("could not refresh edited project assets: {error}"));
        }
        match kind {
            EditKind::Live | EditKind::AudioModifierLive => {
                shrimply_project::project::commit_coalesced_edit(&project, "qt-inspector-edit")
            }
            EditKind::Structural | EditKind::Asset | EditKind::AudioModifierStructural => {
                shrimply_project::project::commit_edit(&project, "qt-inspector-edit")
            }
        };
        let duration = affects_duration.then(|| project.duration());
        let mut change = refresh.unwrap_or_else(|| match kind {
            EditKind::AudioModifierLive => player_state::ProjectChange {
                audio: true,
                audio_waveforms: true,
                live_preview: true,
                ..Default::default()
            },
            EditKind::AudioModifierStructural => player_state::ProjectChange {
                audio: true,
                audio_waveforms: true,
                inspector: true,
                ..Default::default()
            },
            EditKind::Live | EditKind::Structural | EditKind::Asset => {
                crate::refresh::target_change(target, duration, kind != EditKind::Live)
            }
        });
        if affects_duration {
            change.duration = duration;
        }
        if matches!(target, InspectorTarget::Project) && affects_duration {
            change.frame_rate = Some(project.fps);
        }
        drop(project);
        player_state::refresh_project(&self.player_state, change);
        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum EditKind {
    Live,
    Structural,
    Asset,
    AudioModifierLive,
    AudioModifierStructural,
}

fn target_details(
    project: &Project,
    target: &InspectorTarget,
    track: Option<&crate::track::TrackPresentation>,
) -> Vec<InspectorDetail> {
    match target {
        InspectorTarget::Project => vec![
            InspectorDetail {
                label: "Tracks",
                value: format!(
                    "{} video, {} audio, {} caption",
                    project.video_tracks.len(),
                    project.audio_tracks.len(),
                    project.caption_tracks.len()
                ),
            },
            InspectorDetail {
                label: "Duration",
                value: shrimply_project::time_format::project_duration(project.duration()),
            },
            InspectorDetail {
                label: "Project File",
                value: shrimply_project::project::active_project_path()
                    .to_string_lossy()
                    .into_owned(),
            },
        ],
        InspectorTarget::Track(_) => track
            .expect("resolved track inspector target must have cached presentation")
            .details(),
        InspectorTarget::Item(address) => {
            let Some(item) = project.item(address) else {
                return Vec::new();
            };
            let (kind, start, end, natural_duration, source_offset, dimensions, file) = match item {
                ItemRef::Caption(item) => ("Caption", item.start, item.end, None, None, None, None),
                ItemRef::Video(item) => (
                    video_title(item),
                    item.start,
                    item.end,
                    (!(item.is_static_visual_media() || item.is_generated()))
                        .then_some(item.source_duration),
                    Some(item.time_offset),
                    (item.source_width > 0 && item.source_height > 0)
                        .then_some((item.source_width, item.source_height)),
                    (!matches!(&item.content, VideoItemContent::FoldedSequence(_)))
                        .then_some(item.file.path()),
                ),
                ItemRef::Audio(item) => (
                    audio_title(item),
                    item.start,
                    item.end,
                    (!matches!(&item.source, AudioSource::Generator(_)))
                        .then_some(item.source_duration),
                    Some(item.time_offset),
                    None,
                    item.uses_file_asset().then_some(item.file.path()),
                ),
            };
            let mut details = vec![
                InspectorDetail {
                    label: "Type",
                    value: kind.to_string(),
                },
                InspectorDetail {
                    label: "Item ID",
                    value: address.item_id().to_string(),
                },
                InspectorDetail {
                    label: "Track ID",
                    value: address.track_id().to_string(),
                },
            ];
            if !address.sequence_path().is_empty() {
                details.push(InspectorDetail {
                    label: "Sequence Path",
                    value: address
                        .sequence_path()
                        .iter()
                        .map(uuid::Uuid::to_string)
                        .collect::<Vec<_>>()
                        .join(" / "),
                });
            }
            if let Some((timeline_start, timeline_end)) = project.projected_item_times(address) {
                details.push(InspectorDetail {
                    label: "Timeline Start",
                    value: shrimply_project::time_format::playback_time(timeline_start),
                });
                details.push(InspectorDetail {
                    label: "Timeline End",
                    value: shrimply_project::time_format::playback_time(timeline_end),
                });
            }
            details.push(InspectorDetail {
                label: "Local Start",
                value: shrimply_project::time_format::playback_time(start),
            });
            details.push(InspectorDetail {
                label: "Local End",
                value: shrimply_project::time_format::playback_time(end),
            });
            if let Some(natural_duration) = natural_duration {
                details.push(InspectorDetail {
                    label: "Natural Duration",
                    value: shrimply_project::time_format::playback_time(natural_duration),
                });
            }
            details.push(InspectorDetail {
                label: "Timeline Duration",
                value: shrimply_project::time_format::playback_time(end.saturating_sub(start)),
            });
            if let Some(source_offset) = source_offset {
                details.push(InspectorDetail {
                    label: "Source Offset",
                    value: shrimply_project::time_format::playback_time(source_offset),
                });
            }
            if let Some((width, height)) = dimensions {
                details.push(InspectorDetail {
                    label: "Dimensions",
                    value: format!("{width} × {height}"),
                });
            }
            if let Some(file) = file.filter(|file| !file.as_os_str().is_empty()) {
                details.push(InspectorDetail {
                    label: "File Location",
                    value: file.to_string_lossy().into_owned(),
                });
            }
            details
        }
        InspectorTarget::Transition { .. } => Vec::new(),
    }
}

pub(crate) fn target_runtime(
    project: &Project,
    player: &SharedPlayerState,
    target: &InspectorTarget,
) -> InspectorRuntime {
    let snapshot = player_state::snapshot(player);
    let address = match target {
        InspectorTarget::Item(address) | InspectorTarget::Transition { item: address, .. } => {
            Some(address)
        }
        InspectorTarget::Project | InspectorTarget::Track(_) => None,
    };
    let local_time = address.and_then(|address| {
        let sequence_time =
            project.timeline_time_to_sequence(&address.track(), snapshot.position)?;
        match project.item(address)? {
            ItemRef::Video(item) => {
                shrimply_project::project::generated_item_time(item, sequence_time)
            }
            ItemRef::Audio(item) => Some(sequence_time.saturating_sub(item.start)),
            ItemRef::Caption(item) => Some(sequence_time.saturating_sub(item.start)),
        }
    });
    let keyframe_range =
        address.and_then(|address| crate::target::keyframe_range(project, address));
    let duration = keyframe_range.map(|(start, end)| end.saturating_sub(start));
    let frame_step = address
        .and_then(|address| project.keyframe_step(address))
        .filter(|step| *step > Time::ZERO)
        .unwrap_or_else(|| project.frame_step());
    let keyframe_playhead =
        address.and_then(|address| project.keyframe_time(address, snapshot.position));
    InspectorRuntime {
        position: snapshot.position,
        local_time,
        duration,
        keyframe_range,
        frame_step,
        keyframe_playhead,
        frame_rate: snapshot.frame_rate,
    }
}

pub(crate) fn video_title(item: &shrimply_project::project::VideoItem) -> &'static str {
    if item.video_generation.is_some() {
        return "Video Generation";
    }
    match &item.content {
        VideoItemContent::Text(_) => "Text",
        VideoItemContent::Shape(_) => "Shape",
        VideoItemContent::Paint(_) => "Paint",
        VideoItemContent::Background(_) => "Background",
        VideoItemContent::Media => "Video",
        VideoItemContent::Image => "Image",
        VideoItemContent::Gif => "GIF",
        VideoItemContent::Svg => "SVG",
        VideoItemContent::Pdf(_) => "PDF",
        VideoItemContent::Manim(_) => "Manim",
        VideoItemContent::Blender(_) => "Blender",
        VideoItemContent::LayeredImage(_) => "Layered Image",
        VideoItemContent::Obj(_) => "OBJ",
        VideoItemContent::Gaussian(_) => "3D Gaussian Splat",
        VideoItemContent::FoldedSequence(_) => "Folded Sequence",
    }
}

pub(crate) fn target_value(project: &Project, target: &InspectorTarget) -> Option<(String, Value)> {
    match target {
        InspectorTarget::Project => Some(("Project".to_string(), serialize(project))),
        InspectorTarget::Item(address) => {
            let title = match project.item(address)? {
                ItemRef::Caption(_) => "Caption",
                ItemRef::Video(item) => video_title(item),
                ItemRef::Audio(item) => audio_title(item),
            };
            let value = match project.item(address)? {
                ItemRef::Caption(item) => serialize(item),
                ItemRef::Video(item) => serialize(item),
                ItemRef::Audio(item) => serialize(item),
            };
            Some((title.to_string(), value))
        }
        InspectorTarget::Transition { item, side } => {
            let (title, value) = match project.item(item)? {
                ItemRef::Video(item) => {
                    if *side == TransitionSide::Outro
                        && let Some(transition) = item.transitions.to_next.as_ref()
                    {
                        ("Transition", serialize(transition))
                    } else {
                        (
                            match side {
                                TransitionSide::Intro => "Intro",
                                TransitionSide::Outro => "Outro",
                            },
                            serialize(match side {
                                TransitionSide::Intro => item.transitions.intro.as_ref()?,
                                TransitionSide::Outro => item.transitions.outro.as_ref()?,
                            }),
                        )
                    }
                }
                ItemRef::Audio(item) => {
                    if *side == TransitionSide::Outro
                        && let Some(transition) = item.transitions.to_next.as_ref()
                    {
                        ("Transition", serialize(transition))
                    } else {
                        (
                            match side {
                                TransitionSide::Intro => "Intro",
                                TransitionSide::Outro => "Outro",
                            },
                            serialize(match side {
                                TransitionSide::Intro => item.transitions.intro.as_ref()?,
                                TransitionSide::Outro => item.transitions.outro.as_ref()?,
                            }),
                        )
                    }
                }
                ItemRef::Caption(_) => return None,
            };
            Some((title.to_string(), value))
        }
        InspectorTarget::Track(address) => {
            let (title, value) = match project.track(address)? {
                TrackRef::Caption(track) => ("Caption Track", serialize(track)),
                TrackRef::Video(track) => ("Video Track", serialize(track)),
                TrackRef::Audio(track) => ("Audio Track", serialize(track)),
            };
            Some((title.to_string(), value))
        }
    }
}

pub(crate) fn replace_target(
    project: &mut Project,
    target: &InspectorTarget,
    value: Value,
) -> Result<(), String> {
    match target {
        InspectorTarget::Project => *project = deserialize(value)?,
        InspectorTarget::Item(address) => {
            match project
                .item_mut(address)
                .ok_or_else(|| "inspector item is no longer available".to_string())?
            {
                ItemMut::Caption(item) => *item = deserialize(value)?,
                ItemMut::Video(item) => *item = deserialize(value)?,
                ItemMut::Audio(item) => *item = deserialize(value)?,
            }
        }
        InspectorTarget::Transition { item, side } => match project
            .item_mut(item)
            .ok_or_else(|| "inspector transition is no longer available".to_string())?
        {
            ItemMut::Video(item) => {
                if *side == TransitionSide::Outro && item.transitions.to_next.is_some() {
                    item.transitions.to_next = Some(deserialize(value)?);
                } else {
                    *match side {
                        TransitionSide::Intro => &mut item.transitions.intro,
                        TransitionSide::Outro => &mut item.transitions.outro,
                    } = Some(deserialize(value)?);
                }
            }
            ItemMut::Audio(item) => {
                if *side == TransitionSide::Outro && item.transitions.to_next.is_some() {
                    item.transitions.to_next = Some(Box::new(deserialize(value)?));
                } else {
                    *match side {
                        TransitionSide::Intro => &mut item.transitions.intro,
                        TransitionSide::Outro => &mut item.transitions.outro,
                    } = Some(deserialize(value)?);
                }
            }
            ItemMut::Caption(_) => {
                return Err("captions do not have inspector transitions".to_string());
            }
        },
        InspectorTarget::Track(address) => match project
            .track_mut(address)
            .ok_or_else(|| "inspector track is no longer available".to_string())?
        {
            TrackMut::Caption(track) => *track = deserialize(value)?,
            TrackMut::Video(track) => *track = deserialize(value)?,
            TrackMut::Audio(track) => *track = deserialize(value)?,
        },
    }
    Ok(())
}

fn serialize(value: &impl Serialize) -> Value {
    serde_json::to_value(value).expect("inspector target must serialize")
}

fn deserialize<T: DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|error| format!("invalid inspector value: {error}"))
}

fn vector_keys(values: &Map<String, Value>) -> Option<Vec<&'static str>> {
    if ["r", "g", "b", "a"]
        .iter()
        .all(|key| values.get(*key).is_some_and(Value::is_number))
    {
        Some(vec!["r", "g", "b", "a"])
    } else if ["x", "y", "z"]
        .iter()
        .all(|key| values.get(*key).is_some_and(Value::is_number))
    {
        Some(vec!["x", "y", "z"])
    } else if ["x", "y"]
        .iter()
        .all(|key| values.get(*key).is_some_and(Value::is_number))
    {
        Some(vec!["x", "y"])
    } else {
        None
    }
}

pub(crate) fn parsed_value(current: &Value, text: &str) -> Result<Value, String> {
    match current {
        Value::Bool(_) => text
            .parse()
            .map(Value::Bool)
            .map_err(|_| "boolean inspector values must be true or false".to_string()),
        Value::Number(_) => text
            .parse::<Number>()
            .map(Value::Number)
            .map_err(|_| format!("invalid numeric inspector value: {text}")),
        Value::String(_) => Ok(Value::String(text.to_string())),
        Value::Null => Ok(if text.is_empty() {
            Value::Null
        } else {
            Value::String(text.to_string())
        }),
        Value::Array(_) | Value::Object(_) => {
            Err("compound inspector values must be edited as JSON".to_string())
        }
    }
}

fn editable_path(path: &str) -> bool {
    !path
        .rsplit('/')
        .next()
        .is_some_and(|name| matches!(name, "id" | "format_version" | "target_item_id"))
}

fn structural_path(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(|name| {
        matches!(
            name,
            "kind"
                | "shape"
                | "model"
                | "enabled"
                | "beat_detection"
                | "track_id"
                | "speed_method"
                | "repeat_strategy"
                | "waveform"
                | "effect_evolve_seed"
                | "keyframes"
                | "expression"
                | "preserve_formants"
                | "engine"
                | "mode"
        )
    })
}

fn asset_path(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|name| matches!(name, "file" | "image_path" | "environment_file"))
}

pub(crate) fn duration_path(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(|name| {
        matches!(
            name,
            "start" | "end" | "duration" | "playback_speed" | "fps" | "canvas_size"
        )
    })
}
