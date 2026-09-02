use shrimply_math_core::Fraction;
use shrimply_project_core::{
    COMMON_FRAME_RATES, CanvasSize, MAX_CANVAS_DIMENSION, MIN_CANVAS_DIMENSION, PROJECT_PRESETS,
};

pub const CUSTOM_PRESET_INDEX: usize = PROJECT_PRESETS.len();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectSettings {
    pub preset: usize,
    pub width: u32,
    pub height: u32,
    pub frame_rate: usize,
    pub custom_frame_rate: Option<Fraction>,
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
            custom_frame_rate: None,
        }
    }
}

impl ProjectSettings {
    pub fn from_values(canvas_size: CanvasSize, frame_rate: Fraction) -> Self {
        let preset = PROJECT_PRESETS
            .iter()
            .position(|preset| preset.canvas_size == canvas_size && preset.fps == frame_rate)
            .unwrap_or(CUSTOM_PRESET_INDEX);
        let common = COMMON_FRAME_RATES
            .iter()
            .position(|rate| rate.value == frame_rate);
        Self {
            preset,
            width: canvas_size
                .width
                .clamp(MIN_CANVAS_DIMENSION, MAX_CANVAS_DIMENSION),
            height: canvas_size
                .height
                .clamp(MIN_CANVAS_DIMENSION, MAX_CANVAS_DIMENSION),
            frame_rate: common.unwrap_or(COMMON_FRAME_RATES.len()),
            custom_frame_rate: common.is_none().then_some(frame_rate),
        }
    }

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
        self.custom_frame_rate = None;
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width.clamp(MIN_CANVAS_DIMENSION, MAX_CANVAS_DIMENSION);
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height.clamp(MIN_CANVAS_DIMENSION, MAX_CANVAS_DIMENSION);
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn set_frame_rate(&mut self, index: usize) {
        assert!(
            index < COMMON_FRAME_RATES.len(),
            "frame rate index out of range"
        );
        self.frame_rate = index;
        self.custom_frame_rate = None;
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn set_custom_frame_rate(&mut self, frame_rate: Fraction) {
        if let Some(index) = COMMON_FRAME_RATES
            .iter()
            .position(|rate| rate.value == frame_rate)
        {
            self.set_frame_rate(index);
            return;
        }
        self.frame_rate = COMMON_FRAME_RATES.len();
        self.custom_frame_rate = Some(frame_rate);
        self.preset = CUSTOM_PRESET_INDEX;
    }

    pub fn settings(self) -> Option<(CanvasSize, Fraction)> {
        Some((
            CanvasSize {
                width: self.width,
                height: self.height,
            },
            self.custom_frame_rate.or_else(|| {
                COMMON_FRAME_RATES
                    .get(self.frame_rate)
                    .map(|rate| rate.value)
            })?,
        ))
    }
}
