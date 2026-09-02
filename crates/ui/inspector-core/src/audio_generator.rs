use shrimply_core::timeline_value::TimelineValue;
use shrimply_project::project::{AudioGenerator, AudioWaveform};

const MAX_EXACT_F32_INTEGER: f64 = 16_777_215.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioGeneratorNumber {
    Frequency,
    PulseWidth,
    Seed,
}

impl AudioGeneratorNumber {
    pub const fn field(self) -> &'static str {
        match self {
            Self::Frequency => "frequency_hz",
            Self::PulseWidth => "pulse_width",
            Self::Seed => "seed",
        }
    }
}

#[derive(Clone, Copy)]
pub struct AudioGeneratorNumberPresentation {
    pub field: AudioGeneratorNumber,
    pub label: &'static str,
    pub minimum: f64,
    pub maximum: f64,
    pub drag_step: f64,
    pub digits: usize,
    pub unit: &'static str,
    pub display: fn(f32) -> f64,
    pub store_multiplier: f64,
    pub integer: bool,
}

#[derive(Clone, Copy)]
pub struct AudioWaveformChoice {
    pub value: AudioWaveform,
    pub key: &'static str,
    pub label: &'static str,
}

pub enum AudioGeneratorControl<'a> {
    Waveform {
        value: AudioWaveform,
        choices: &'static [AudioWaveformChoice],
    },
    Number {
        value: &'a TimelineValue<f32>,
        presentation: AudioGeneratorNumberPresentation,
    },
}

pub fn controls(generator: &AudioGenerator) -> Vec<AudioGeneratorControl<'_>> {
    let mut controls = vec![AudioGeneratorControl::Waveform {
        value: generator.waveform,
        choices: WAVEFORMS,
    }];
    if !generator.waveform.is_noise() {
        controls.push(number(&generator.frequency_hz, FREQUENCY));
    }
    if generator.waveform == AudioWaveform::SquarePulse {
        controls.push(number(&generator.pulse_width, PULSE_WIDTH));
    }
    if generator.waveform.is_noise() {
        controls.push(number(&generator.seed, SEED));
    }
    controls
}

fn number<'a>(
    value: &'a TimelineValue<f32>,
    presentation: AudioGeneratorNumberPresentation,
) -> AudioGeneratorControl<'a> {
    AudioGeneratorControl::Number {
        value,
        presentation,
    }
}

fn percent(value: f32) -> f64 {
    f64::from(value) * 100.0
}

const FREQUENCY: AudioGeneratorNumberPresentation = AudioGeneratorNumberPresentation {
    field: AudioGeneratorNumber::Frequency,
    label: "Frequency",
    minimum: 1.0,
    maximum: 20_000.0,
    drag_step: 0.01,
    digits: 2,
    unit: "Hz",
    display: f64::from,
    store_multiplier: 1.0,
    integer: false,
};

const PULSE_WIDTH: AudioGeneratorNumberPresentation = AudioGeneratorNumberPresentation {
    field: AudioGeneratorNumber::PulseWidth,
    label: "Pulse width",
    minimum: 1.0,
    maximum: 99.0,
    drag_step: 1.0,
    digits: 0,
    unit: "%",
    display: percent,
    store_multiplier: 0.01,
    integer: false,
};

const SEED: AudioGeneratorNumberPresentation = AudioGeneratorNumberPresentation {
    field: AudioGeneratorNumber::Seed,
    label: "Seed",
    minimum: 0.0,
    maximum: MAX_EXACT_F32_INTEGER,
    drag_step: 1.0,
    digits: 0,
    unit: "",
    display: f64::from,
    store_multiplier: 1.0,
    integer: true,
};

const WAVEFORMS: &[AudioWaveformChoice] = &[
    AudioWaveformChoice {
        value: AudioWaveform::Sine,
        key: "sine",
        label: "Sine",
    },
    AudioWaveformChoice {
        value: AudioWaveform::SquarePulse,
        key: "square_pulse",
        label: "Square / Pulse",
    },
    AudioWaveformChoice {
        value: AudioWaveform::Triangle,
        key: "triangle",
        label: "Triangle",
    },
    AudioWaveformChoice {
        value: AudioWaveform::Sawtooth,
        key: "sawtooth",
        label: "Sawtooth",
    },
    AudioWaveformChoice {
        value: AudioWaveform::WhiteNoise,
        key: "white_noise",
        label: "White Noise",
    },
    AudioWaveformChoice {
        value: AudioWaveform::PinkNoise,
        key: "pink_noise",
        label: "Pink Noise",
    },
    AudioWaveformChoice {
        value: AudioWaveform::BrownNoise,
        key: "brown_noise",
        label: "Brown Noise",
    },
];
