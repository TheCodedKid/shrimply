use shrimply_project::project::{
    ItemAddress, ItemRef, Project, Time, TrackAddress, TransitionSide,
};
use shrimply_timeline::selection_state::{self, SharedSelectionState};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InspectorTarget {
    Project,
    Item(ItemAddress),
    Track(TrackAddress),
    Transition {
        item: ItemAddress,
        side: TransitionSide,
    },
}

pub(crate) fn resolve(project: &Project, selection: &SharedSelectionState) -> InspectorTarget {
    let target = if let Some((item, side)) =
        selection_state::focused_transition_address(selection, project)
    {
        InspectorTarget::Transition { item, side }
    } else if let Some(item) = selection_state::focused_item_address(selection, project) {
        InspectorTarget::Item(item)
    } else if let Some(track) = selection_state::focused_track_address(selection, project) {
        InspectorTarget::Track(track)
    } else {
        InspectorTarget::Project
    };
    if available(project, &target) {
        target
    } else {
        InspectorTarget::Project
    }
}

fn available(project: &Project, target: &InspectorTarget) -> bool {
    match target {
        InspectorTarget::Project => true,
        InspectorTarget::Item(address) => project.item(address).is_some(),
        InspectorTarget::Track(address) => project.track(address).is_some(),
        InspectorTarget::Transition { item, side } => match project.item(item) {
            Some(ItemRef::Video(item)) => {
                (*side == TransitionSide::Outro && item.transitions.to_next.is_some())
                    || match side {
                        TransitionSide::Intro => item.transitions.intro.is_some(),
                        TransitionSide::Outro => item.transitions.outro.is_some(),
                    }
            }
            Some(ItemRef::Audio(item)) => {
                (*side == TransitionSide::Outro && item.transitions.to_next.is_some())
                    || match side {
                        TransitionSide::Intro => item.transitions.intro.is_some(),
                        TransitionSide::Outro => item.transitions.outro.is_some(),
                    }
            }
            Some(ItemRef::Caption(_)) | None => false,
        },
    }
}

pub(crate) fn keyframe_range(project: &Project, address: &ItemAddress) -> Option<(Time, Time)> {
    let (start, end) = project.projected_item_times(address)?;
    let track = address.track();
    let local = |time| {
        let time = project.timeline_time_to_sequence(&track, time)?;
        match project.item(address)? {
            ItemRef::Video(item) => Some(shrimply_project::project::generated_item_animation_time(
                item, time,
            )),
            ItemRef::Audio(item) => Some(time.signed_sub(item.start)),
            ItemRef::Caption(item) => Some(time.signed_sub(item.start)),
        }
    };
    let (start, end) = (local(start)?, local(end)?);
    Some(if start <= end {
        (start, end)
    } else {
        (end, start)
    })
}
