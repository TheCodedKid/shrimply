use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use shrimply_core::timeline_value::{
    Interpolation, TimelineBool, TimelineExpressionValue, TimelineStep, TimelineValue,
};
use shrimply_project::project::{ItemAddress, Time};

use crate::audio_modifiers::{audio_item_address, audio_modifier_evaluation_time};
use crate::{
    AudioModifierKeyframeMove, InspectorController, InspectorExpressionOutput, InspectorTarget,
};

impl InspectorController {
    pub fn current_keyframe_time(&self, target: &InspectorTarget) -> Result<Time, String> {
        let address = match target {
            InspectorTarget::Item(address) | InspectorTarget::Transition { item: address, .. } => {
                address
            }
            InspectorTarget::Project | InspectorTarget::Track(_) => {
                return Err("inspector target has no keyframe time".to_string());
            }
        };
        self.project
            .borrow()
            .keyframe_time(
                address,
                shrimply_state::player_state::current_time(&self.player_state),
            )
            .ok_or_else(|| "the current item keyframe time is no longer available".to_string())
    }

    pub fn set_bool_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        next: bool,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.bool_timeline(target, path)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "boolean keyframe time is no longer available".to_string())?;
        if !crate::keyframe_model::set_discrete_value(
            &mut value,
            time,
            next.into(),
            runtime.frame_step,
        ) {
            return Ok(());
        }
        self.set_live_value(target, path, serialize_bool_timeline(value))
    }

    pub fn set_bool_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.set_video_step_keyframes_enabled::<TimelineBool>(target, path, enabled)
    }

    pub fn set_step_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String> {
        match path {
            "/sample_method" => self
                .set_video_step_keyframes_enabled::<shrimply_core::VideoSampleMethod>(
                    target, path, enabled,
                ),
            "/compositing/blend_mode" => self
                .set_video_step_keyframes_enabled::<shrimply_core::LayerBlendMode>(
                    target, path, enabled,
                ),
            _ => Err(format!("unknown step timeline: {path}")),
        }
    }

    fn set_video_step_keyframes_enabled<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String>
    where
        T: TimelineStep + DeserializeOwned + Serialize,
    {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let mut value: TimelineValue<T> = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("step timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid step timeline: {error}"))?;
        let current = value.value_at(snapshot.runtime.local_time.unwrap_or(Time::ZERO));
        let time = snapshot
            .runtime
            .keyframe_playhead
            .ok_or_else(|| "step keyframe time is no longer available".to_string())?;
        if !shrimply_core::timeline_value::set_keyframes_enabled(&mut value, time, current, enabled)
        {
            return Ok(());
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(value).expect("step timeline must serialize"),
        )
    }

    pub fn bool_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<bool>, String> {
        let outcome = self.video_expression_output::<TimelineBool>(target, path)?;
        Ok(InspectorExpressionOutput {
            value: outcome.value.get(),
            error: outcome.error,
        })
    }

    pub fn step_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<String>, String> {
        match path {
            "/sample_method" => {
                self.step_expression_output_as::<shrimply_core::VideoSampleMethod>(target, path)
            }
            "/compositing/blend_mode" => {
                self.step_expression_output_as::<shrimply_core::LayerBlendMode>(target, path)
            }
            _ => Err(format!("unknown step timeline: {path}")),
        }
    }

    fn step_expression_output_as<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<String>, String>
    where
        T: TimelineExpressionValue + TimelineStep + DeserializeOwned,
    {
        let outcome = self.video_expression_output::<T>(target, path)?;
        let value = T::variants()
            .iter()
            .find(|variant| variant.value == outcome.value)
            .expect("evaluated timeline step must be one of its declared variants")
            .key
            .to_string();
        Ok(InspectorExpressionOutput {
            value,
            error: outcome.error,
        })
    }

    pub(crate) fn video_expression_output<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<T>, String>
    where
        T: TimelineExpressionValue + DeserializeOwned,
    {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let player = shrimply_state::player_state::snapshot(&self.player_state);
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let position = project
            .timeline_time_to_sequence(&address.track(), player.position)
            .ok_or_else(|| "expression time is no longer available".to_string())?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "expression item is no longer available".to_string())?;
        let serialized = transform_expression_value(item, path).unwrap_or_else(|| {
            serde_json::to_value(item)
                .expect("video item must serialize")
                .pointer(path)
                .cloned()
                .unwrap_or(Value::Null)
        });
        let value: TimelineValue<T> = serde_json::from_value(serialized)
            .map_err(|error| format!("invalid timeline expression value at {path}: {error}"))?;
        let audio =
            self.audio_sampler
                .borrow_mut()
                .sample(&project, player.position, player.revision);
        let evaluation = shrimply_evaluation::VisualEvaluation::for_item_with_audio(
            &project, item, position, &audio,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            &value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub fn move_bool_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        moves: &[(Time, Time)],
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.bool_timeline(target, path)?;
        if !crate::keyframe_model::move_discrete_keyframes(&mut value, &moves) {
            return Err("boolean keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value(target, path, serialize_bool_timeline(value))?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_bool_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.bool_timeline(target, path)?;
        if !crate::keyframe_model::delete_discrete_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.set_value(target, path, serialize_bool_timeline(value))
    }

    pub fn add_bool_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, runtime) = self.bool_timeline(target, path)?;
        if !crate::keyframe_model::add_discrete_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.set_value(target, path, serialize_bool_timeline(value))
    }

    pub fn copy_bool_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.bool_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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

    pub fn paste_bool_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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
        drop(project);
        if times.len() != clipboard.len() {
            return Err("boolean keyframes cannot be pasted at this time".to_string());
        }
        let (mut value, _) = self.bool_timeline(target, path)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.set_value(target, path, serialize_bool_timeline(value))?;
        Ok(pasted.len())
    }

    pub fn seek_discrete_keyframe(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<(), String> {
        let address = video_item_address(target)?;
        let project = self.project.borrow();
        let position = project
            .keyframe_timeline_time(address, time)
            .ok_or_else(|| "boolean keyframe time is no longer available".to_string())?;
        drop(project);
        shrimply_state::player_state::seek_time(&self.player_state, position);
        Ok(())
    }

    pub fn move_step_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        moves: &[(Time, Time)],
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.json_step_timeline(target, path)?;
        if !crate::keyframe_model::move_json_discrete_keyframes(&mut value, &moves)? {
            return Err("selector keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value(target, path, value)?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_step_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.json_step_timeline(target, path)?;
        if !crate::keyframe_model::delete_json_discrete_keyframe(
            &mut value,
            time,
            runtime.frame_step,
        )? {
            return Ok(());
        }
        self.set_value(target, path, value)
    }

    pub fn add_step_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, runtime) = self.json_step_timeline(target, path)?;
        if !crate::keyframe_model::add_json_discrete_keyframe(&mut value, time, runtime.frame_step)?
        {
            return Ok(());
        }
        self.set_value(target, path, value)
    }

    pub fn copy_step_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.json_step_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_json_discrete_keyframes(
            &value,
            selected,
            step_timeline_type(path)?,
        )?
        else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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

    pub fn paste_step_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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
        drop(project);
        if times.len() != clipboard.len() {
            return Err("step keyframes cannot be pasted at this time".to_string());
        }
        let (mut value, _) = self.json_step_timeline(target, path)?;
        let Some(pasted) = crate::keyframe_model::paste_json_discrete_keyframes(
            &mut value,
            &clipboard,
            &times,
            step_timeline_type(path)?,
        )?
        else {
            return Ok(0);
        };
        self.set_value(target, path, value)?;
        Ok(pasted.len())
    }

    fn json_step_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<(Value, crate::InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let value = snapshot
            .value
            .pointer(path)
            .cloned()
            .ok_or_else(|| format!("step timeline is no longer available: {path}"))?;
        Ok((value, snapshot.runtime))
    }

    fn canonical_video_keyframe_time(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<Time, String> {
        let address = video_item_address(target)?;
        let project = self.project.borrow();
        project
            .keyframe_timeline_time(address, time)
            .and_then(|time| project.keyframe_time(address, time))
            .ok_or_else(|| "keyframe time is no longer available".to_string())
    }

    fn canonical_video_keyframe_moves(
        &self,
        target: &InspectorTarget,
        moves: &[(Time, Time)],
    ) -> Result<Vec<(Time, Time)>, String> {
        moves
            .iter()
            .map(|&(old_time, time)| {
                Ok((old_time, self.canonical_video_keyframe_time(target, time)?))
            })
            .collect()
    }

    fn bool_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<(TimelineValue<TimelineBool>, crate::InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let value = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("boolean timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid boolean timeline: {error}"))?;
        Ok((value, snapshot.runtime))
    }

    pub fn set_scalar_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let mut value: TimelineValue<f32> = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("inspector timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid inspector timeline: {error}"))?;
        let current = value.value_at(snapshot.runtime.local_time.unwrap_or(Time::ZERO));
        let keyframe_time = snapshot
            .runtime
            .keyframe_playhead
            .ok_or_else(|| "scalar keyframe time is no longer available".to_string())?;
        if !shrimply_core::timeline_value::set_keyframes_enabled(
            &mut value,
            keyframe_time,
            current,
            enabled,
        ) {
            return Ok(());
        }
        self.set_value(target, path, serialize_timeline(value))
    }

    pub fn scalar_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<TimelineValue<f32>, String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("inspector timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid inspector timeline: {error}"))
    }

    pub fn scalar_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput, String> {
        if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. })) {
            return self.video_expression_output(target, path);
        }
        let value = self.scalar_timeline(target, path)?;
        let project = self.project.borrow();
        let address = audio_item_address(target)?;
        let local_time = audio_modifier_evaluation_time(&project, &self.player_state, target)?;
        let item = project
            .audio_item(address)
            .ok_or_else(|| "audio item is no longer available".to_string())?;
        let evaluation = shrimply_evaluation::VisualEvaluation::for_audio_item_local_time(
            &project, item, local_time,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            &value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub fn move_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        change: AudioModifierKeyframeMove,
    ) -> Result<(), String> {
        let time = self.canonical_scalar_keyframe_time(target, change.time)?;
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::keyframe_model::update_scalar_keyframe(
            &mut value,
            change.old_time,
            time,
            (change.displayed_value * change.store_multiplier) as f32,
        ) {
            return Ok(());
        }
        self.set_live_keyframe_graph_value(target, path, serialize_timeline(value))
    }

    pub fn delete_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::keyframe_model::delete_scalar_keyframe(&mut value, time) {
            return Ok(());
        }
        self.set_value(target, path, serialize_timeline(value))
    }

    pub fn add_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_scalar_keyframe_time(target, time)?;
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::keyframe_model::add_scalar_keyframe(&mut value, time) {
            return Ok(());
        }
        self.set_value(target, path, serialize_timeline(value))
    }

    pub fn set_scalar_keyframe_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
    ) -> Result<(), String> {
        let interpolation = Interpolation::KEYFRAME
            .get(interpolation_index)
            .copied()
            .ok_or_else(|| "keyframe interpolation is invalid".to_string())?;
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::keyframe_model::set_scalar_interpolation(&mut value, owner_id, interpolation) {
            return Ok(());
        }
        self.set_value(target, path, serialize_timeline(value))
    }

    pub fn copy_scalar_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let value = self.scalar_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
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

    pub fn paste_scalar_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
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
        drop(project);
        if times.len() != clipboard.len() {
            return Err("keyframes cannot be pasted at this time".to_string());
        }
        let mut value = self.scalar_timeline(target, path)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.set_value(target, path, serialize_timeline(value))?;
        Ok(pasted.len())
    }

    pub fn seek_scalar_keyframe(&self, target: &InspectorTarget, time: Time) -> Result<(), String> {
        let address = scalar_item_address(target)?;
        let project = self.project.borrow();
        let position = project
            .keyframe_timeline_time(address, time)
            .ok_or_else(|| "keyframe time is no longer available".to_string())?;
        drop(project);
        shrimply_state::player_state::seek_time(&self.player_state, position);
        Ok(())
    }

    fn canonical_scalar_keyframe_time(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<Time, String> {
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
        project
            .keyframe_timeline_time(address, time)
            .and_then(|timeline_time| project.keyframe_time(address, timeline_time))
            .ok_or_else(|| "scalar keyframe time is no longer available".to_string())
    }
}

fn transform_expression_value(
    item: &shrimply_project::project::VideoItem,
    path: &str,
) -> Option<Value> {
    let value = match path {
        "/transform/position" => serde_json::to_value(&item.transform.position),
        "/transform/anchor" => serde_json::to_value(&item.transform.anchor),
        "/transform/scale" => serde_json::to_value(&item.transform.scale),
        "/transform/shear" => serde_json::to_value(&item.transform.shear),
        "/transform/rotation_degrees" => serde_json::to_value(&item.transform.rotation_degrees),
        _ => return None,
    };
    Some(value.expect("transform timeline must serialize"))
}

fn serialize_timeline(value: TimelineValue<f32>) -> Value {
    serde_json::to_value(value).expect("scalar timeline must serialize")
}

fn serialize_bool_timeline(value: TimelineValue<TimelineBool>) -> Value {
    serde_json::to_value(value).expect("boolean timeline must serialize")
}

fn video_item_address(target: &InspectorTarget) -> Result<&ItemAddress, String> {
    let InspectorTarget::Item(address @ ItemAddress::Video { .. }) = target else {
        return Err("boolean keyframe target is not a video item".to_string());
    };
    Ok(address)
}

fn scalar_item_address(target: &InspectorTarget) -> Result<&ItemAddress, String> {
    match target {
        InspectorTarget::Item(
            address @ (ItemAddress::Audio { .. } | ItemAddress::Video { .. }),
        ) => Ok(address),
        _ => Err("scalar keyframe target is not an audio or video item".to_string()),
    }
}

fn step_timeline_type(path: &str) -> Result<&'static str, String> {
    match path {
        "/sample_method" => Ok(std::any::type_name::<shrimply_core::VideoSampleMethod>()),
        "/compositing/blend_mode" => Ok(std::any::type_name::<shrimply_core::LayerBlendMode>()),
        _ => Err(format!("unknown step timeline: {path}")),
    }
}
