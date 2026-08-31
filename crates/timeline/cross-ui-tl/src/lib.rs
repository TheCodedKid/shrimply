use shrimply_state::preferences::{self, PreferencesSnapshot, SharedPreferences};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum CursorTool {
    #[default]
    Pointer,
    Cut,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum DragCollisionMode {
    #[default]
    Overwrite,
    Block,
    NewTrack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolState {
    pub magnet: bool,
    pub beat_grid: bool,
    pub cursor: CursorTool,
    pub drag_collision: DragCollisionMode,
}

impl ToolState {
    pub fn from_preferences(preferences: &PreferencesSnapshot) -> Self {
        Self {
            magnet: preferences.timeline_magnet == "true",
            beat_grid: preferences.timeline_beat_grid == "true",
            cursor: if preferences.timeline_cursor == "cut" {
                CursorTool::Cut
            } else {
                CursorTool::Pointer
            },
            drag_collision: match preferences.timeline_drag_collision_mode.as_str() {
                "block" => DragCollisionMode::Block,
                "new_track" => DragCollisionMode::NewTrack,
                _ => DragCollisionMode::Overwrite,
            },
        }
    }
}

#[derive(Clone)]
pub struct TimelineTools {
    preferences: SharedPreferences,
}

impl TimelineTools {
    pub fn new(preferences: SharedPreferences) -> Self {
        Self { preferences }
    }

    pub fn state(&self) -> ToolState {
        ToolState::from_preferences(&preferences::snapshot(&self.preferences))
    }

    pub fn set_magnet(&self, enabled: bool) {
        preferences::set_timeline_magnet(&self.preferences, enabled);
    }

    pub fn set_beat_grid(&self, enabled: bool) {
        preferences::set_timeline_beat_grid(&self.preferences, enabled);
    }

    pub fn set_cursor(&self, cursor: CursorTool) {
        preferences::set_timeline_cursor(
            &self.preferences,
            match cursor {
                CursorTool::Pointer => "pointer",
                CursorTool::Cut => "cut",
            },
        );
    }

    pub fn set_drag_collision(&self, mode: DragCollisionMode) {
        preferences::set_timeline_drag_collision_mode(
            &self.preferences,
            match mode {
                DragCollisionMode::Overwrite => "overwrite",
                DragCollisionMode::Block => "block",
                DragCollisionMode::NewTrack => "new_track",
            },
        );
    }
}
