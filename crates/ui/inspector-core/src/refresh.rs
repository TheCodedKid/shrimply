use shrimply_project::project::{ItemAddress, TrackAddress};
use shrimply_state::player_state::ProjectChange;

use crate::{InspectorTarget, model::EditKind};

pub(crate) fn target_change(
    target: &InspectorTarget,
    duration: Option<shrimply_project::project::Time>,
    inspector: bool,
) -> ProjectChange {
    let mut change = ProjectChange {
        duration,
        inspector,
        ..Default::default()
    };
    match target {
        InspectorTarget::Project => {
            if inspector {
                change.audio = true;
                change.audio_beats = true;
                change.audio_waveforms = true;
                change.video = true;
                change.live_preview = true;
                change.captions = true;
            }
        }
        InspectorTarget::Item(ItemAddress::Audio { .. })
        | InspectorTarget::Track(TrackAddress::Audio { .. }) => {
            change.audio = true;
            change.audio_beats = true;
            change.audio_waveforms = true;
        }
        InspectorTarget::Item(ItemAddress::Video { .. })
        | InspectorTarget::Track(TrackAddress::Video { .. }) => change.video = true,
        InspectorTarget::Item(ItemAddress::Caption { .. })
        | InspectorTarget::Track(TrackAddress::Caption { .. }) => change.captions = true,
        InspectorTarget::Transition {
            item: ItemAddress::Audio { .. },
            ..
        } => change.audio = true,
        InspectorTarget::Transition {
            item: ItemAddress::Video { .. },
            ..
        } => change.video = true,
        InspectorTarget::Transition {
            item: ItemAddress::Caption { .. },
            ..
        } => unreachable!("captions do not have transitions"),
    }
    change
}

pub(crate) fn audio_path_change(
    target: &InspectorTarget,
    path: &str,
    kind: EditKind,
) -> Option<ProjectChange> {
    if !audio_target(target) {
        return None;
    }
    let inspector = kind != EditKind::Live;
    Some(match path {
        "/enabled" => ProjectChange {
            audio: true,
            inspector,
            ..Default::default()
        },
        "/beat_detection" => ProjectChange {
            audio_beats: true,
            inspector,
            ..Default::default()
        },
        "/track_id" => ProjectChange {
            audio: true,
            audio_beats: true,
            audio_waveforms: true,
            inspector,
            ..Default::default()
        },
        "/playback_speed" | "/repeat_strategy" => ProjectChange {
            audio: true,
            audio_waveforms: true,
            inspector,
            ..Default::default()
        },
        "/speed_method" => ProjectChange {
            audio: true,
            inspector,
            ..Default::default()
        },
        "/source/waveform" | "/source/frequency_hz" | "/source/pulse_width" | "/source/seed" => {
            ProjectChange {
                audio: true,
                audio_waveforms: true,
                inspector,
                ..Default::default()
            }
        }
        "/source" => ProjectChange {
            inspector,
            ..Default::default()
        },
        "/gain" | "/gain/decibels" if kind == EditKind::Live => ProjectChange {
            audio: true,
            live_preview: true,
            ..Default::default()
        },
        "/gain" | "/gain/decibels" => ProjectChange {
            audio: true,
            audio_waveforms: true,
            inspector,
            ..Default::default()
        },
        _ => return None,
    })
}

pub(crate) fn audio_paths_change<'a>(
    target: &InspectorTarget,
    paths: impl IntoIterator<Item = &'a str>,
    kind: EditKind,
) -> Option<ProjectChange> {
    let mut combined = ProjectChange::default();
    for path in paths {
        merge(&mut combined, audio_path_change(target, path, kind)?);
    }
    Some(combined)
}

pub(crate) fn audio_scalar_expression_change(target: &InspectorTarget) -> Option<ProjectChange> {
    audio_target(target).then_some(ProjectChange {
        audio: true,
        audio_waveforms: true,
        ..Default::default()
    })
}

pub(crate) fn audio_scalar_graph_change(target: &InspectorTarget) -> Option<ProjectChange> {
    match target {
        InspectorTarget::Item(ItemAddress::Audio { .. }) => Some(ProjectChange {
            audio: true,
            audio_waveforms: true,
            live_preview: true,
            ..Default::default()
        }),
        InspectorTarget::Item(ItemAddress::Video { .. }) => Some(ProjectChange {
            video: true,
            live_preview: true,
            ..Default::default()
        }),
        _ => None,
    }
}

fn audio_target(target: &InspectorTarget) -> bool {
    matches!(
        target,
        InspectorTarget::Item(ItemAddress::Audio { .. })
            | InspectorTarget::Track(TrackAddress::Audio { .. })
    )
}

fn merge(combined: &mut ProjectChange, change: ProjectChange) {
    combined.audio |= change.audio;
    combined.audio_beats |= change.audio_beats;
    combined.audio_waveforms |= change.audio_waveforms;
    combined.live_preview |= change.live_preview;
    combined.inspector |= change.inspector;
}
