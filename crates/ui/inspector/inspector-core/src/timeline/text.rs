use serde_json::Value;
use shrimply_project_document::project::Time;
use shrimply_property_model::timeline_value::{TextInterpolation, TimelineValue};

use super::video_item_address;
use crate::{
    InspectorCommit, InspectorController, InspectorExpressionOutput, InspectorTarget,
    TextKeyframeCommits,
};

impl InspectorController {
    pub fn set_text_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        next: String,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "text keyframe time is no longer available".to_string())?;
        if !crate::timeline_text::set_value(&mut value, time, next, runtime.frame_step) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Live,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Live),
            commit,
        )
    }

    pub fn set_text_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        let evaluation_time = runtime
            .local_time
            .ok_or_else(|| "text evaluation time is no longer available".to_string())?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "text keyframe time is no longer available".to_string())?;
        if !crate::timeline_text::set_keyframes_enabled(&mut value, evaluation_time, time, enabled)
        {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn text_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<InspectorExpressionOutput<String>, String> {
        self.video_modifier_expression_output(
            target,
            path,
            timeline_id,
            crate::timeline_text::video_value,
        )
    }

    pub fn set_text_expression_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_expression_enabled(&mut value, enabled) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn set_text_expression_source(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        source: String,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_expression_source(&mut value, source) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Live,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Live),
            commit,
        )
    }

    pub fn move_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        moves: &[(Time, Time)],
        commits: TextKeyframeCommits,
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::move_keyframes(&mut value, &moves) {
            return Err("text keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value_with_commit(
            target,
            path,
            serialize_text_timeline(value),
            InspectorCommit::Coalesced(commits.move_keyframe),
        )?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_text_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commits: TextKeyframeCommits,
    ) -> Result<(), String> {
        self.delete_text_keyframes(
            target,
            path,
            timeline_id,
            std::slice::from_ref(&time),
            commits,
        )
    }

    pub fn delete_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        times: &[Time],
        commits: TextKeyframeCommits,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        if !crate::keyframe_model::edit_keyframe_selection(&mut value, times, |value, time| {
            Ok(crate::timeline_text::delete_keyframe(
                value,
                time,
                runtime.frame_step,
            ))
        })? {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commits.delete),
        )
    }

    pub fn add_text_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commits: TextKeyframeCommits,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::add_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commits.add),
        )
    }

    pub fn copy_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.text_timeline(target, path, timeline_id)?;
        let Some(mut clipboard) = crate::timeline_text::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        if !crate::keyframe_model::normalize_clipboard_times(
            &project,
            Some(address),
            &mut clipboard,
        ) {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        }
        let count = clipboard.len();
        self.keyframe_clipboard.replace(Some(clipboard));
        Ok(count)
    }

    pub fn paste_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commits: TextKeyframeCommits,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("text keyframes cannot be pasted at this time".to_string());
        };
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        let Some(pasted) = crate::timeline_text::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commits.paste),
        )?;
        Ok(pasted.len())
    }

    pub fn set_text_keyframe_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
        commit_name: &str,
    ) -> Result<(), String> {
        let interpolation = crate::keyframe_model::interpolation(interpolation_index)?;
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_interpolation(&mut value, owner_id, interpolation)
            .unwrap_or(false)
        {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commit_name),
        )
    }

    pub fn text_keyframe_text_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
    ) -> Result<usize, String> {
        let (value, _) = self.text_timeline(target, path, timeline_id)?;
        let interpolation = crate::timeline_text::text_interpolation(&value, owner_id)
            .ok_or_else(|| "text keyframe is no longer available".to_string())?;
        TextInterpolation::ALL
            .iter()
            .position(|candidate| *candidate == interpolation)
            .ok_or_else(|| "text interpolation is unavailable".to_string())
    }

    pub fn set_text_keyframe_text_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
        commit_name: &str,
    ) -> Result<(), String> {
        let interpolation = TextInterpolation::ALL
            .get(interpolation_index)
            .copied()
            .ok_or_else(|| "text interpolation is invalid".to_string())?;
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_text_interpolation(&mut value, owner_id, interpolation)? {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commit_name),
        )
    }

    fn text_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<(TimelineValue<String>, crate::InspectorRuntime), String> {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "text item is no longer available".to_string())?;
        let value = crate::timeline_text::video_value(item, path, timeline_id)
            .cloned()
            .ok_or_else(|| format!("text timeline is no longer available: {path}"))?;
        let runtime = crate::model::target_runtime(&project, &self.player_state, target);
        Ok((value, runtime))
    }
}

fn serialize_text_timeline(value: TimelineValue<String>) -> Value {
    serde_json::to_value(value).expect("text timeline must serialize")
}
