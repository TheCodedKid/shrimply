use std::str::FromStr;

use shrimply_math_core::{
    FRACTION_ZERO, Fraction, fraction_as_f64, fraction_from_f64, fraction_from_integer,
    fraction_is_finite,
};

pub const DEFAULT_DRAG_PIXELS: f64 = 3.0;
pub const DRAG_THRESHOLD_PIXELS: f64 = 2.0;
pub const MAX_DRAG_STEPS: i64 = 1_000_000;
pub const DEFAULT_MINIMUM: i64 = -1_000_000;
pub const DEFAULT_MAXIMUM: i64 = 1_000_000;

#[derive(Clone, Copy)]
pub struct NumberConfig {
    pub minimum: Fraction,
    pub maximum: Fraction,
    pub drag_step: Fraction,
    pub drag_pixels: f64,
    pub digits: usize,
    pub fallback: Fraction,
}

impl NumberConfig {
    pub fn new(value: Fraction) -> Self {
        Self {
            minimum: fraction_from_integer(DEFAULT_MINIMUM),
            maximum: fraction_from_integer(DEFAULT_MAXIMUM),
            drag_step: fraction_from_integer(1),
            drag_pixels: DEFAULT_DRAG_PIXELS,
            digits: 2,
            fallback: finite_fraction_or(value, FRACTION_ZERO),
        }
    }
}

pub fn accepted_value(config: &NumberConfig, value: Fraction) -> Fraction {
    let value = finite_fraction_or(value, config.fallback);
    clamped_value(value, config.minimum, config.maximum)
}

pub fn clamped_value(value: Fraction, minimum: Fraction, maximum: Fraction) -> Fraction {
    let value = finite_fraction_or(value, FRACTION_ZERO);
    if minimum <= maximum {
        value.clamp(minimum, maximum)
    } else {
        value.clamp(maximum, minimum)
    }
}

pub fn drag_steps(offset_x: f64, drag_pixels: f64) -> i64 {
    (offset_x / drag_pixels)
        .round()
        .clamp(-(MAX_DRAG_STEPS as f64), MAX_DRAG_STEPS as f64) as i64
}

pub fn dragged_value(config: &NumberConfig, start: Fraction, offset_x: f64) -> Fraction {
    accepted_value(
        config,
        start + config.drag_step * fraction_from_integer(drag_steps(offset_x, config.drag_pixels)),
    )
}

pub fn format_value(config: &NumberConfig, value: Fraction) -> String {
    let value = fraction_as_f64(value);
    let value = if value.abs() < 10.0_f64.powi(-(config.digits as i32)) / 2.0 {
        0.0
    } else {
        value
    };
    format!("{:.*}", config.digits, value)
}

pub fn parse_fraction(value: &str) -> Option<Fraction> {
    Fraction::from_str(value)
        .ok()
        .filter(|value| fraction_is_finite(*value))
}

pub fn finite_fraction_or(value: Fraction, fallback: Fraction) -> Fraction {
    if fraction_is_finite(value) {
        value
    } else {
        fallback
    }
}

pub fn positive_fraction_or(value: Fraction, fallback: Fraction) -> Fraction {
    if fraction_as_f64(value) > 0.0 {
        value
    } else {
        fallback
    }
}

pub fn pair_ratio(first: Fraction, second: Fraction) -> Fraction {
    if second == FRACTION_ZERO {
        fraction_from_integer(1)
    } else {
        first / second
    }
}

pub fn pair_second(first: Fraction, ratio: Fraction) -> Fraction {
    if ratio == FRACTION_ZERO {
        first
    } else {
        first / ratio
    }
}

pub fn locked_pair(axis: usize, value: Fraction, ratio: Fraction) -> [Fraction; 2] {
    match axis {
        0 => [value, pair_second(value, ratio)],
        1 => [value * ratio, value],
        _ => [value; 2],
    }
}

pub fn triple_ratios(values: [Fraction; 3]) -> [Fraction; 2] {
    if values[0] == FRACTION_ZERO {
        [fraction_from_integer(1); 2]
    } else {
        [values[1] / values[0], values[2] / values[0]]
    }
}

pub fn locked_triple(axis: usize, value: Fraction, ratios: [Fraction; 2]) -> [Fraction; 3] {
    let first = match axis {
        0 => value,
        1 if ratios[0] != FRACTION_ZERO => value / ratios[0],
        2 if ratios[1] != FRACTION_ZERO => value / ratios[1],
        _ => value,
    };
    [first, first * ratios[0], first * ratios[1]]
}

pub fn from_f64(value: f64) -> Fraction {
    fraction_from_f64(value)
}
