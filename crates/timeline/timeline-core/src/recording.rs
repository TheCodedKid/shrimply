use crate::{
    TrackKey,
    draw_state::LiveRecordingDraw,
    items,
    metrics::RECORDING_DURATION_HEADROOM_SECONDS,
    project::{AudioItem, Project, Time},
};
use shrimply_state::player_state::{self, ProjectChange, SharedPlayerState};

pub struct AudioRecording {
    pub key: TrackKey,
    pub start: Time,
    recording: shrimply_audio::recording::MicRecording,
}

impl AudioRecording {
    pub fn start(
        key: TrackKey,
        project: &Project,
        player: &SharedPlayerState,
    ) -> Result<Self, String> {
        if project.audio_tracks.get(key.track_index).is_none() {
            return Err("Audio track no longer exists".into());
        }
        let start = player_state::snapshot(player)
            .position
            .snapped(project.frame_step());
        let recording = shrimply_audio::recording::MicRecording::start()?;
        ensure_duration(player, start);
        player_state::set_playing(player, true);
        Ok(Self {
            key,
            start,
            recording,
        })
    }

    pub fn finish(self, project: &mut Project, player: &SharedPlayerState) -> Result<(), String> {
        let finished = self.recording.finish()?;
        if finished.duration <= Time::ZERO {
            return Ok(());
        }
        let end = self
            .start
            .saturating_add(finished.duration)
            .snapped(project.frame_step());
        if end <= self.start {
            let _ = std::fs::remove_file(&finished.path);
            return Err("The recording is shorter than one project frame".into());
        }
        let Some(track) = project.audio_tracks.get_mut(self.key.track_index) else {
            let _ = std::fs::remove_file(&finished.path);
            return Err("Audio track no longer exists".into());
        };
        let item = AudioItem::builder(self.start, end)
            .source_duration(finished.duration)
            .file(finished.path)
            .build();
        items::overwrite_items(&mut track.items, self.start, end);
        items::insert_sorted(&mut track.items, item);
        crate::project::commit_edit(project, "record-audio");
        player_state::refresh_project(
            player,
            ProjectChange {
                duration: Some(project.duration()),
                audio: true,
                audio_waveforms: true,
                inspector: true,
                ..Default::default()
            },
        );
        Ok(())
    }

    pub fn draw(&self, waveform_chunks_per_second: u32) -> Option<LiveRecordingDraw> {
        let snapshot = self.recording.snapshot();
        const CHANNELS: usize = 2;
        let frames = snapshot.samples.len() / CHANNELS;
        if frames == 0 {
            return None;
        }
        let duration = Time::from_fraction(
            i64::try_from(frames).ok()?,
            i64::from(snapshot.sample_rate.max(1)),
        );
        Some(LiveRecordingDraw {
            key: self.key,
            item: AudioItem::builder(self.start, self.start.saturating_add(duration))
                .source_duration(duration)
                .file(std::path::PathBuf::from("live-recording.opus"))
                .build(),
            waveform: shrimply_audio::waveform::from_stereo_samples(
                &snapshot.samples,
                snapshot.sample_rate,
                waveform_chunks_per_second,
            ),
        })
    }
}

fn ensure_duration(player: &SharedPlayerState, position: Time) {
    let duration = position.saturating_add(Time::from_seconds(RECORDING_DURATION_HEADROOM_SECONDS));
    if duration > player_state::snapshot(player).duration {
        player_state::set_duration(player, duration);
    }
}
