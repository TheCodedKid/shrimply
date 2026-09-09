use shrimply_math_core::time_ticks;
use shrimply_project_document::caption::{CaptionItem, HorizontalAlign, VerticalAlign};

pub const DEFAULT_FONT_SIZE: u32 = 32;
const PERCENT: u32 = 100;
const SECONDS_PER_MINUTE: u64 = 60;
const MINUTES_PER_HOUR: u64 = 60;
pub const MILLIS_PER_CENTISECOND: u64 = 10;

pub fn reveal_intervals(offsets: impl Iterator<Item = u32>, duration: u64) -> Vec<(u64, u64)> {
    let mut boundaries = offsets
        .map(|millis| u64::from(millis) / MILLIS_PER_CENTISECOND)
        .filter(|offset| *offset < duration)
        .collect::<Vec<_>>();
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries.push(duration);
    boundaries
        .windows(2)
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

pub fn cue_ticks(item: &CaptionItem, scale: u32) -> Result<(u64, u64), String> {
    let start =
        time_ticks(item.start, scale).ok_or("Caption start exceeds the timestamp range.")?;
    let minimum_end = start
        .checked_add(1)
        .ok_or("Caption end exceeds the timestamp range.")?;
    let end = time_ticks(item.end, scale).ok_or("Caption end exceeds the timestamp range.")?;
    Ok((start, end.max(minimum_end)))
}

pub fn timestamp(ticks: u64, scale: u32, separator: char) -> String {
    let scale = u64::from(scale);
    let seconds = ticks / scale;
    let hours = seconds / SECONDS_PER_MINUTE / MINUTES_PER_HOUR;
    let minutes = seconds / SECONDS_PER_MINUTE % MINUTES_PER_HOUR;
    let seconds = seconds % SECONDS_PER_MINUTE;
    let fraction = ticks % scale;
    if scale == u64::from(crate::CENTIS_PER_SECOND) {
        format!("{hours}:{minutes:02}:{seconds:02}{separator}{fraction:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}{separator}{fraction:03}")
    }
}

pub fn font_size(scale: u16) -> u32 {
    (DEFAULT_FONT_SIZE * u32::from(scale) / PERCENT).max(1)
}

pub fn coordinate(percent: u8, dimension: u32) -> u32 {
    (u64::from(percent.min(PERCENT as u8)) * u64::from(dimension) / u64::from(PERCENT)) as u32
}

pub fn ass_alignment(item: &CaptionItem) -> u8 {
    let row = match item.v_align {
        VerticalAlign::Bottom => 0,
        VerticalAlign::Middle => 1,
        VerticalAlign::Top => 2,
    };
    let column = match item.h_align {
        HorizontalAlign::Left => 1,
        HorizontalAlign::Center => 2,
        HorizontalAlign::Right => 3,
    };
    row * 3 + column
}

pub fn opacity(alpha: u8) -> String {
    // CSS alpha as a decimal, calculated entirely with integer arithmetic.
    let thousandths = u32::from(alpha) * crate::MILLIS_PER_SECOND / u32::from(u8::MAX);
    format!(
        "{}.{:03}",
        thousandths / crate::MILLIS_PER_SECOND,
        thousandths % crate::MILLIS_PER_SECOND
    )
}
