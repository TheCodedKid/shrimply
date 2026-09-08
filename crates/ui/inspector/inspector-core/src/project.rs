use std::path::PathBuf;

use shrimply_project_document::project::{CanvasSize, Project, Time};

pub use shrimply_project_document::project::{MAX_CANVAS_DIMENSION, MIN_CANVAS_DIMENSION};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectPresentation {
    pub name: String,
    pub canvas_size: CanvasSize,
    pub frame_rate: shrimply_math_core::Fraction,
    pub video_track_count: usize,
    pub audio_track_count: usize,
    pub caption_track_count: usize,
    pub duration: Time,
    pub file: PathBuf,
}

impl ProjectPresentation {
    pub fn details(&self) -> Vec<crate::InspectorDetail> {
        vec![
            crate::InspectorDetail {
                label: "Tracks",
                value: format!(
                    "{} video, {} audio, {} caption",
                    self.video_track_count, self.audio_track_count, self.caption_track_count
                ),
            },
            crate::InspectorDetail {
                label: "Duration",
                value: shrimply_project_document::time_format::project_duration(self.duration),
            },
            crate::InspectorDetail {
                label: "Project File",
                value: self.file.to_string_lossy().into_owned(),
            },
        ]
    }
}

pub fn presentation(project: &Project) -> ProjectPresentation {
    ProjectPresentation {
        name: project.name.clone(),
        canvas_size: project.canvas_size,
        frame_rate: project.fps,
        video_track_count: project.video_tracks.len(),
        audio_track_count: project.audio_tracks.len(),
        caption_track_count: project.caption_tracks.len(),
        duration: project.duration(),
        file: shrimply_project_document::project::active_project_path(),
    }
}
