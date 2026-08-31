use shrimply_math_core::Fraction;
use shrimply_project_core::{COMMON_FRAME_RATES, CanvasSize, PROJECT_PRESETS};

pub const CUSTOM_PRESET_INDEX: usize = PROJECT_PRESETS.len();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectSettings {
    pub preset: usize,
    pub width: u32,
    pub height: u32,
    pub frame_rate: usize,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        let preset = PROJECT_PRESETS
            .iter()
            .position(|preset| preset.label == "1080p 30 FPS")
            .unwrap_or_default();
        let selected = &PROJECT_PRESETS[preset];
        let frame_rate = COMMON_FRAME_RATES
            .iter()
            .position(|rate| rate.value == selected.fps)
            .unwrap_or_default();
        Self {
            preset,
            width: selected.canvas_size.width,
            height: selected.canvas_size.height,
            frame_rate,
        }
    }
}

impl ProjectSettings {
    pub fn select_preset(&mut self, index: usize) {
        let Some(preset) = PROJECT_PRESETS.get(index) else {
            self.preset = CUSTOM_PRESET_INDEX;
            return;
        };
        self.preset = index;
        self.width = preset.canvas_size.width;
        self.height = preset.canvas_size.height;
        self.frame_rate = COMMON_FRAME_RATES
            .iter()
            .position(|rate| rate.value == preset.fps)
            .expect("project preset frame rate must be listed");
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width.clamp(1, 16_384);
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height.clamp(1, 16_384);
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn set_frame_rate(&mut self, index: usize) {
        assert!(
            index < COMMON_FRAME_RATES.len(),
            "frame rate index out of range"
        );
        self.frame_rate = index;
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn settings(self) -> Option<(CanvasSize, Fraction)> {
        Some((
            CanvasSize {
                width: self.width,
                height: self.height,
            },
            COMMON_FRAME_RATES.get(self.frame_rate)?.value,
        ))
    }
}
