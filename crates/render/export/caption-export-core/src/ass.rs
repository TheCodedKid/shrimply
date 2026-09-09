use crate::{CENTIS_PER_SECOND, math, text::font_name};
use shrimply_project_document::{
    caption::{CaptionEdgeStyle, CaptionItem, CaptionWritingDirection, markup},
    project::{CanvasSize, Color},
};
use std::fmt::Write;

const DEFAULT_MARGIN: u32 = 10;
const EDGE_WIDTH: u8 = 1;
const BACKGROUND_PADDING: u8 = 2;

pub fn document(items: &[&CaptionItem], canvas: CanvasSize) -> Result<String, String> {
    if canvas.width == 0 || canvas.height == 0 {
        return Err("ASS export requires a nonzero project canvas size.".into());
    }
    let mut output = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {}\nPlayResY: {}\nWrapStyle: 0\nScaledBorderAndShadow: yes\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n",
        canvas.width, canvas.height,
    );
    for (index, item) in items.iter().enumerate() {
        let (font, size, foreground, background, edge, effect) = if item.styling_enabled {
            (
                font_name(item.font),
                math::font_size(item.font_scale),
                item.text_color,
                item.background_color,
                item.edge_color,
                item.edge_style,
            )
        } else {
            (
                "Roboto",
                math::DEFAULT_FONT_SIZE,
                Color::<u8>::WHITE,
                Color::<u8>::TRANSPARENT,
                Color::<u8>::BLACK,
                CaptionEdgeStyle::None,
            )
        };
        let alignment = if item.layout_enabled {
            math::ass_alignment(item)
        } else {
            2
        };
        let outline = if matches!(effect, CaptionEdgeStyle::Glow | CaptionEdgeStyle::Bevel) {
            EDGE_WIDTH
        } else {
            0
        };
        let shadow = if matches!(
            effect,
            CaptionEdgeStyle::HardShadow | CaptionEdgeStyle::SoftShadow
        ) {
            EDGE_WIDTH
        } else {
            0
        };
        // An opaque box uses the outline color as its fill in ASS. Edge effects
        // are approximated by the remaining shadow when a background is present.
        let (border, outline, outline_color) = if background.a > 0 {
            (3, BACKGROUND_PADDING, background)
        } else {
            (1, outline, edge)
        };
        writeln!(output,
            "Style: Cue{index},{font},{size},{},{},{},{},0,0,0,0,100,100,0,0,{border},{outline},{shadow},{alignment},{DEFAULT_MARGIN},{DEFAULT_MARGIN},{DEFAULT_MARGIN},1",
            color(foreground), color(foreground), color(outline_color), color(edge),
        ).unwrap();
    }
    output.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    for (index, item) in items.iter().enumerate() {
        let (start, end) = math::cue_ticks(item, CENTIS_PER_SECOND)?;
        let spans = markup::parse(&item.text);
        let intervals =
            math::reveal_intervals(spans.iter().map(|span| span.start_millis), end - start);
        if intervals.is_empty() {
            return Err(format!(
                "Caption {} has no timed text within its time range.",
                item.id
            ));
        }
        // Emit only text that has appeared in each interval. Unlike karaoke fill
        // alpha, this also prevents future backgrounds, outlines and shadows.
        for (visible_at, visible_until) in intervals {
            write!(
                output,
                "Dialogue: 0,{},{},Cue{index},,0,0,0,,",
                math::timestamp(start + visible_at, CENTIS_PER_SECOND, '.'),
                math::timestamp(start + visible_until, CENTIS_PER_SECOND, '.'),
            )
            .unwrap();
            if item.layout_enabled {
                write!(
                    output,
                    "{{\\an{}\\pos({},{})}}",
                    math::ass_alignment(item),
                    math::coordinate(item.position_x, canvas.width),
                    math::coordinate(item.position_y, canvas.height),
                )
                .unwrap();
                match item.writing_direction {
                    CaptionWritingDirection::RotatedLeftToRight => output.push_str("{\\frz270}"),
                    CaptionWritingDirection::RotatedRightToLeft => output.push_str("{\\frz90}"),
                    _ => {}
                }
            }
            if item.styling_enabled && item.edge_style == CaptionEdgeStyle::SoftShadow {
                output.push_str("{\\blur1}");
            }
            for span in spans.iter().filter(|span| {
                u64::from(span.start_millis) / math::MILLIS_PER_CENTISECOND <= visible_at
            }) {
                write!(
                    output,
                    "{{\\b{}\\i{}\\u{}}}",
                    u8::from(span.bold),
                    u8::from(span.italic),
                    u8::from(span.underline)
                )
                .unwrap();
                let visible = span.ruby.as_ref().map_or_else(
                    || span.text.clone(),
                    |ruby| format!("{} ({})", ruby.base, ruby.annotation),
                );
                // ASS has no portable escape for a literal backslash immediately
                // before N/n/h. A zero-width separator keeps it literal in libass.
                output.push_str(
                    &visible
                        .replace("\r\n", "\n")
                        .replace('\r', "\n")
                        .replace('\\', "\\\u{200b}")
                        .replace('{', "\\{")
                        .replace('}', "\\}")
                        .replace('\n', "\\N"),
                );
            }
            output.push('\n');
        }
    }
    Ok(output)
}

fn color(color: Color<u8>) -> String {
    format!(
        "&H{:02X}{:02X}{:02X}{:02X}",
        u8::MAX - color.a,
        color.b,
        color.g,
        color.r
    )
}
