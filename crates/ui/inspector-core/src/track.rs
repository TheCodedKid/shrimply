use shrimply_project::project::{
    ItemKind, Project, TrackAddress, TrackMut, TrackRef, caption_languages,
    supported_caption_language,
};
use shrimply_state::player_state::{self, ProjectChange};

use crate::{InspectorController, InspectorDetail};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackPresentation {
    pub address: TrackAddress,
    pub kind: ItemKind,
    pub ordinal: usize,
    pub enabled: bool,
    pub language: Option<String>,
    pub item_count: usize,
}

impl TrackPresentation {
    pub fn title(&self) -> &'static str {
        match self.kind {
            ItemKind::Video => "Video Track",
            ItemKind::Caption => "Caption Track",
            ItemKind::Audio => "Audio Track",
        }
    }

    pub fn details(&self) -> Vec<InspectorDetail> {
        vec![
            InspectorDetail {
                label: "Type",
                value: self.title().to_string(),
            },
            InspectorDetail {
                label: "Track",
                value: (self.ordinal + 1).to_string(),
            },
            InspectorDetail {
                label: "Items",
                value: self.item_count.to_string(),
            },
        ]
    }
}

pub fn presentation(project: &Project, address: TrackAddress) -> Option<TrackPresentation> {
    let (enabled, language, item_count) = match project.track(&address)? {
        TrackRef::Caption(track) => (
            track.enabled,
            supported_caption_language(&track.language),
            track.items.len(),
        ),
        TrackRef::Video(track) => (track.enabled, None, track.items.len()),
        TrackRef::Audio(track) => (track.enabled, None, track.items.len()),
    };
    let ordinal = match &address {
        TrackAddress::Caption { track_id } => project
            .caption_tracks
            .iter()
            .position(|track| track.id == *track_id)?,
        TrackAddress::Video {
            sequence_path,
            track_id,
        } => project
            .video_tracks_for_path(sequence_path)?
            .iter()
            .position(|track| track.id == *track_id)?,
        TrackAddress::Audio {
            sequence_path,
            track_id,
        } => project
            .audio_tracks_for_path(sequence_path)?
            .iter()
            .position(|track| track.id == *track_id)?,
    };
    Some(TrackPresentation {
        kind: address.kind(),
        address,
        ordinal,
        enabled,
        language,
        item_count,
    })
}

impl InspectorController {
    pub fn set_track_enabled(&self, address: &TrackAddress, next: bool) -> Result<(), String> {
        let kind = address.kind();
        let mut project = self.project.borrow_mut();
        let Some(track) = project.track_mut(address) else {
            return Ok(());
        };
        let enabled = match track {
            TrackMut::Caption(track) => &mut track.enabled,
            TrackMut::Video(track) => &mut track.enabled,
            TrackMut::Audio(track) => &mut track.enabled,
        };
        if *enabled == next {
            return Ok(());
        }
        *enabled = next;
        shrimply_project::project::commit_edit(&project, "toggle-track-enabled");
        let duration = project.duration();
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            ProjectChange {
                duration: Some(duration),
                audio: kind == ItemKind::Audio,
                audio_beats: kind == ItemKind::Audio,
                audio_waveforms: kind == ItemKind::Audio,
                video: kind == ItemKind::Video,
                captions: kind == ItemKind::Caption,
                inspector: true,
                ..ProjectChange::default()
            },
        );
        Ok(())
    }

    pub fn set_caption_track_language(
        &self,
        address: &TrackAddress,
        language: Option<&str>,
    ) -> Result<(), String> {
        if let Some(language) = language
            && !caption_languages().contains(language)
        {
            return Err(format!("unsupported caption language: {language}"));
        }
        let mut project = self.project.borrow_mut();
        let Some(track) = project.track_mut(address) else {
            return Ok(());
        };
        let TrackMut::Caption(track) = track else {
            return Err("inspector track is not a caption track".to_string());
        };
        if track.language.as_deref() == language {
            return Ok(());
        }
        track.language = language.map(str::to_string);
        shrimply_project::project::commit_edit(&project, "set-caption-track-language");
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            ProjectChange {
                captions: true,
                inspector: true,
                ..ProjectChange::default()
            },
        );
        Ok(())
    }
}
