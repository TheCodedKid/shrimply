use super::*;
use shrimply_project::project::{ItemAddress, VideoItem, VisualTrack};
use shrimply_video_core::clip_transition::{ActiveClipTransition, held_item};
use shrimply_video_core::sequence::morph_endpoint;
use std::borrow::Cow;

pub(super) struct PreparedItem<'a> {
    pub item: Cow<'a, VideoItem>,
    pub motion_blur_source: Option<Cow<'a, VideoItem>>,
    pub address: ItemAddress,
    pub time: Time,
    pub scope_positions: Vec<Time>,
    pub audio: Option<FrameAudioAnalysis>,
    pub clip_transition: Option<ActiveClipTransition>,
    pub morph_peer: Option<uuid::Uuid>,
    pub children: Option<Vec<PreparedItem<'a>>>,
    pub video_mask: Option<shrimply_project::project::CanvasSize>,
}

#[derive(Clone, Copy)]
pub(super) struct Target<'a> {
    pub address: &'a ItemAddress,
    pub scope_positions: Option<&'a [Time]>,
}

#[derive(Default)]
pub(super) struct Scope {
    pub time: Time,
    pub path: Vec<uuid::Uuid>,
    pub ancestors: Vec<uuid::Uuid>,
    pub positions: Vec<Time>,
}

impl Scene {
    pub(super) fn items<'a>(
        &mut self,
        project: &'a Project,
        tracks: &'a [VisualTrack],
        audio: &FrameAudioAnalysis,
        scope: &Scope,
        target: Option<Target<'_>>,
        requests: &mut Vec<media::Request>,
    ) -> Result<Vec<PreparedItem<'a>>, String> {
        let position = target
            .and_then(|target| {
                target
                    .scope_positions
                    .and_then(|positions| positions.get(scope.path.len()).copied())
            })
            .unwrap_or(scope.time);
        let mut scope_positions = scope.positions.clone();
        scope_positions.push(position);
        let mut active_items = shrimply_video_core::sequence::active_tracks(tracks, position, None);
        if target.is_none() {
            active_items.retain(|active| Some(active.item.id) != self.excluded_item_id);
        }
        if let Some(target) = target {
            let target = target.address;
            let depth = scope.path.len();
            if depth <= target.sequence_path().len() {
                let target_item_id = target
                    .sequence_path()
                    .get(depth)
                    .copied()
                    .unwrap_or_else(|| target.item_id());
                let final_item = depth == target.sequence_path().len();
                active_items.retain(|active| {
                    active.item.id == target_item_id
                        && (!final_item || active.track_id == target.track_id())
                });
            }
        }
        let mut items = Vec::new();
        for (active_index, active) in active_items.iter().enumerate() {
            let morph = morph_endpoint(&active_items, active_index);
            let endpoint_time = morph.map(|endpoint| endpoint.sample_time);
            let item_time = endpoint_time.unwrap_or(position);
            let mut item_scope_positions = scope_positions.clone();
            *item_scope_positions
                .last_mut()
                .expect("prepared item has an active sequence scope") = item_time;
            let endpoint_audio = endpoint_time.map(|time| {
                let audio = self
                    .audio_sampler
                    .sample(project, time, self.audio_revision);
                self.sampled_audio.push(audio.clone());
                audio
            });
            let item_audio = endpoint_audio.as_ref().unwrap_or(audio);
            let item = if endpoint_time.is_some() {
                Cow::Borrowed(active.item)
            } else {
                held_item(active.item, position, active.clip_transition.is_some())
            };
            let address = ItemAddress::Video {
                sequence_path: scope.path.clone(),
                track_id: active.track_id,
                item_id: item.id,
            };
            let (item, motion_blur_source) =
                match shrimply_video_core::modifier_cache::effective_item(
                    &address,
                    &item,
                    project.canvas_size,
                )? {
                    Some(cached) => (Cow::Owned(cached), Some(item)),
                    None => (item, None),
                };
            if target.is_none()
                && !resolve_bool(
                    &item.visibility,
                    &VisualEvaluation::for_item_with_audio(project, &item, item_time, item_audio),
                    &mut self.expressions,
                )
            {
                continue;
            }
            let source_time = match item.content {
                VideoItemContent::Media | VideoItemContent::Gif => {
                    let content_position = shrimply_video_core::transparent_fill::render_position(
                        project, &item, item_time,
                    );
                    let Some(time) = video_source_time_at(&item, content_position) else {
                        continue;
                    };
                    time
                }
                VideoItemContent::Background(_)
                | VideoItemContent::Shape(_)
                | VideoItemContent::Text(_)
                | VideoItemContent::Paint(_) => {
                    if shrimply_project::project::generated_item_time(&item, item_time).is_none() {
                        continue;
                    }
                    Time::ZERO
                }
                _ => Time::ZERO,
            };
            let children = if let VideoItemContent::FoldedSequence(reference) = item.content {
                let Some((sequence, time)) = shrimply_video_core::sequence::resolve(
                    project,
                    &item,
                    reference,
                    item_time,
                    &scope.ancestors,
                )?
                else {
                    continue;
                };
                let mut child_scope = Scope {
                    time,
                    path: scope.path.clone(),
                    ancestors: scope.ancestors.clone(),
                    positions: item_scope_positions.clone(),
                };
                child_scope.path.push(item.id);
                child_scope.ancestors.push(reference.sequence_id);
                let children = self.items(
                    project,
                    &sequence.video_tracks,
                    item_audio,
                    &child_scope,
                    target,
                    requests,
                )?;
                if children.is_empty() {
                    continue;
                }
                Some(children)
            } else {
                if let Some(request) = media::Request::new(
                    &item,
                    address.clone(),
                    source_time,
                    self.requested_accuracy,
                    media::Plane::Content,
                ) {
                    requests.push(request);
                }
                None
            };
            let video_mask = item
                .alpha_mask_video
                .map(|stream| {
                    let (source, size) = shrimply_video_core::alpha_mask::video_source(
                        &item,
                        stream,
                        project.canvas_size,
                    );
                    let request = media::Request::new(
                        &source,
                        address.clone(),
                        source_time,
                        self.requested_accuracy,
                        media::Plane::Alpha,
                    )
                    .ok_or("This source does not provide an alpha video stream")?;
                    requests.push(request);
                    Ok::<_, String>(size)
                })
                .transpose()?;
            items.push(PreparedItem {
                item,
                motion_blur_source,
                address,
                time: item_time,
                scope_positions: item_scope_positions,
                audio: endpoint_audio,
                clip_transition: active.clip_transition,
                morph_peer: morph.map(|endpoint| endpoint.peer_id),
                children,
                video_mask,
            });
        }
        let prepared_ids = items
            .iter()
            .map(|prepared| prepared.address.item_id())
            .collect::<std::collections::HashSet<_>>();
        items.retain(|prepared| {
            prepared
                .morph_peer
                .is_none_or(|peer| prepared_ids.contains(&peer))
        });
        Ok(items)
    }
}
