//! Shared visual-track traversal and folded-sequence time resolution.
use shrimply_project::project::{
    FoldedSequence, ItemAddress, Project, SequenceReference, Time, VideoItem, VideoItemContent,
    VisualTrack, video_source_time_at,
};
use uuid::Uuid;

pub struct ActiveVideoItem<'a> {
    pub track_index: usize,
    pub track_id: Uuid,
    pub item: &'a VideoItem,
    pub clip_transition: Option<ActiveClipTransition>,
    pub previous: Option<&'a VideoItem>,
}

#[derive(Clone, Copy)]
pub struct MorphEndpoint {
    pub peer_index: usize,
    pub peer_id: Uuid,
    pub sample_time: Time,
}

use crate::clip_transition::ActiveClipTransition;

pub fn active_video_items<'a>(
    track_index: usize,
    track_id: Uuid,
    items: &'a [VideoItem],
    position: Time,
    item_ids: Option<&[Uuid]>,
) -> Vec<ActiveVideoItem<'a>> {
    crate::clip_transition::active_items(items, position, item_ids)
        .into_iter()
        .map(|active| ActiveVideoItem {
            track_index,
            track_id,
            item: active.item,
            clip_transition: active.clip_transition,
            previous: active.previous,
        })
        .collect()
}

pub fn active_tracks<'a>(
    tracks: &'a [VisualTrack],
    position: Time,
    item_ids: Option<&[Uuid]>,
) -> Vec<ActiveVideoItem<'a>> {
    tracks
        .iter()
        .enumerate()
        .filter(|(_, track)| track.enabled)
        .flat_map(|(index, track)| {
            active_video_items(index, track.id, &track.items, position, item_ids)
        })
        .collect()
}

pub fn morph_endpoint(items: &[ActiveVideoItem<'_>], index: usize) -> Option<MorphEndpoint> {
    use crate::clip_transition::ClipTransitionRole;
    use shrimply_project::project::VisualClipTransitionKind;

    let active = &items[index];
    let transition = active
        .clip_transition
        .filter(|transition| transition.definition.kind == VisualClipTransitionKind::Morph)?;
    let (peer_index, sample_time) = match transition.role {
        ClipTransitionRole::Outgoing => {
            let peer_offset = items[index + 1..].iter().position(|candidate| {
                candidate.track_id == active.track_id
                    && candidate
                        .clip_transition
                        .is_some_and(|candidate_transition| {
                            candidate_transition.definition.kind == VisualClipTransitionKind::Morph
                                && candidate_transition.role == ClipTransitionRole::Incoming
                                && candidate_transition.progress == transition.progress
                        })
            })?;
            (index + 1 + peer_offset, active.item.end)
        }
        ClipTransitionRole::Incoming => {
            let peer_index = items[..index].iter().rposition(|candidate| {
                candidate.track_id == active.track_id
                    && candidate
                        .clip_transition
                        .is_some_and(|candidate_transition| {
                            candidate_transition.definition.kind == VisualClipTransitionKind::Morph
                                && candidate_transition.role == ClipTransitionRole::Outgoing
                                && candidate_transition.progress == transition.progress
                        })
            })?;
            (peer_index, items[peer_index].item.end)
        }
    };
    let (source, target) =
        shrimply_math_media::clip_transition_bounds(sample_time, transition.definition.duration);
    Some(MorphEndpoint {
        peer_index,
        peer_id: items[peer_index].item.id,
        sample_time: match transition.role {
            ClipTransitionRole::Outgoing => source,
            ClipTransitionRole::Incoming => target,
        },
    })
}

pub fn resolve<'a>(
    project: &'a Project,
    item: &VideoItem,
    reference: SequenceReference,
    position: Time,
    ancestors: &[Uuid],
) -> Result<Option<(&'a FoldedSequence, Time)>, String> {
    let Some(position) = video_source_time_at(item, position) else {
        return Ok(None);
    };
    if ancestors.contains(&reference.sequence_id) {
        return Err(format!(
            "cyclic folded sequence reference involving {}",
            reference.sequence_id
        ));
    }
    let sequence = project
        .folded_sequence(reference.sequence_id)
        .ok_or_else(|| format!("missing folded sequence {}", reference.sequence_id))?;
    Ok(Some((sequence, position)))
}

pub fn video_item_addresses(project: &Project) -> Result<Vec<ItemAddress>, String> {
    fn collect(
        project: &Project,
        tracks: &[VisualTrack],
        sequence_path: &mut Vec<Uuid>,
        sequence_stack: &mut Vec<Uuid>,
        addresses: &mut Vec<ItemAddress>,
    ) -> Result<(), String> {
        for track in tracks {
            for item in &track.items {
                addresses.push(ItemAddress::Video {
                    sequence_path: sequence_path.clone(),
                    track_id: track.id,
                    item_id: item.id,
                });
                let VideoItemContent::FoldedSequence(reference) = item.content else {
                    continue;
                };
                if sequence_stack.contains(&reference.sequence_id) {
                    return Err(format!(
                        "cyclic folded sequence reference involving {}",
                        reference.sequence_id
                    ));
                }
                let sequence = project
                    .folded_sequence(reference.sequence_id)
                    .ok_or_else(|| format!("missing folded sequence {}", reference.sequence_id))?;
                sequence_stack.push(reference.sequence_id);
                sequence_path.push(item.id);
                let result = collect(
                    project,
                    &sequence.video_tracks,
                    sequence_path,
                    sequence_stack,
                    addresses,
                );
                sequence_path.pop();
                sequence_stack.pop();
                result?;
            }
        }
        Ok(())
    }

    let mut addresses = Vec::new();
    collect(
        project,
        &project.video_tracks,
        &mut Vec::new(),
        &mut Vec::new(),
        &mut addresses,
    )?;
    Ok(addresses)
}
