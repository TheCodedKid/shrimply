use std::path::PathBuf;

use shrimply_project::project::{CanvasSize, Project, Time};

pub use shrimply_project::project::{MAX_CANVAS_DIMENSION, MIN_CANVAS_DIMENSION};

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

pub fn presentation(project: &Project) -> ProjectPresentation {
    ProjectPresentation {
        name: project.name.clone(),
        canvas_size: project.canvas_size,
        frame_rate: project.fps,
        video_track_count: project.video_tracks.len(),
        audio_track_count: project.audio_tracks.len(),
        caption_track_count: project.caption_tracks.len(),
        duration: project.duration(),
        file: shrimply_project::project::active_project_path(),
    }
}
