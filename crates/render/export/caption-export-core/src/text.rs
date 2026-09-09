use crate::{MILLIS_PER_SECOND, math};
use shrimply_project_document::caption::{
    CaptionEdgeStyle, CaptionFont, CaptionItem, CaptionWritingDirection, HorizontalAlign,
    VerticalAlign, markup,
};
use std::fmt::Write;

pub fn font_name(font: CaptionFont) -> &'static str {
    match font {
        CaptionFont::Roboto => "Roboto",
        CaptionFont::MonospaceSerif => "Courier New",
        CaptionFont::Serif => "Times New Roman",
        CaptionFont::MonospaceSans => "Lucida Console",
        CaptionFont::Casual => "Comic Sans MS",
        CaptionFont::Cursive => "Monotype Corsiva",
        CaptionFont::SmallCapitals => "Arial",
    }
}

pub fn document(items: &[&CaptionItem], webvtt: bool) -> Result<String, String> {
    let mut output = if webvtt {
        String::from("WEBVTT\n\n")
    } else {
        String::new()
    };
    if webvtt && items.iter().any(|item| item.styling_enabled) {
        output.push_str("STYLE\n");
        for (index, item) in items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.styling_enabled)
        {
            let color = item.text_color;
            let background = item.background_color;
            writeln!(output,
                "::cue(.cue{index}) {{ color: rgba({}, {}, {}, {}); background-color: rgba({}, {}, {}, {}); font-family: \"{}\"; font-size: {}%;{}{} }}",
                color.r, color.g, color.b, math::opacity(color.a),
                background.r, background.g, background.b, math::opacity(background.a),
                font_name(item.font), item.font_scale.max(1),
                if item.font == CaptionFont::SmallCapitals { " font-variant: small-caps;" } else { "" },
                edge_css(item),
            ).unwrap();
        }
        output.push('\n');
    }
    for (index, item) in items.iter().enumerate() {
        let (start, end) = math::cue_ticks(item, MILLIS_PER_SECOND)?;
        let separator = if webvtt { '.' } else { ',' };
        writeln!(
            output,
            "{}\n{} --> {}{}",
            index + 1,
            math::timestamp(start, MILLIS_PER_SECOND, separator),
            math::timestamp(end, MILLIS_PER_SECOND, separator),
            if webvtt {
                settings(item)
            } else {
                String::new()
            },
        )
        .unwrap();
        let mut payload = String::new();
        if webvtt && item.styling_enabled {
            write!(payload, "<c.cue{index}>").unwrap();
        }
        let mut previous_offset = 0;
        for span in markup::parse(&item.text) {
            if webvtt {
                if span.start_millis < previous_offset {
                    return Err(format!("Caption {} has out-of-order timed spans.", item.id));
                }
                if span.start_millis > previous_offset {
                    previous_offset = span.start_millis;
                    let timestamp = start
                        .checked_add(u64::from(span.start_millis))
                        .ok_or("Timed span exceeds the timestamp range.")?;
                    if timestamp >= end {
                        break;
                    }
                    write!(
                        payload,
                        "<{}>",
                        math::timestamp(timestamp, MILLIS_PER_SECOND, '.')
                    )
                    .unwrap();
                }
            }
            for (enabled, tag) in [(span.bold, "b"), (span.italic, "i"), (span.underline, "u")] {
                if enabled {
                    write!(payload, "<{tag}>").unwrap();
                }
            }
            if let Some(ruby) = span.ruby {
                if webvtt {
                    write!(
                        payload,
                        "<ruby>{}<rt>{}</rt></ruby>",
                        escape(&ruby.base),
                        escape(&ruby.annotation)
                    )
                    .unwrap();
                } else {
                    write!(
                        payload,
                        "{} ({})",
                        escape(&ruby.base),
                        escape(&ruby.annotation)
                    )
                    .unwrap();
                }
            } else {
                payload.push_str(&escape(&span.text));
            }
            for (enabled, tag) in [(span.underline, "u"), (span.italic, "i"), (span.bold, "b")] {
                if enabled {
                    write!(payload, "</{tag}>").unwrap();
                }
            }
        }
        if webvtt && item.styling_enabled {
            payload.push_str("</c>");
        }
        // Empty physical lines terminate cues in both formats. A zero-width space
        // preserves an intentional blank display line without ending the cue.
        for line in payload.split('\n') {
            writeln!(
                output,
                "{}",
                if line.trim().is_empty() {
                    "\u{200b}"
                } else {
                    line
                }
            )
            .unwrap();
        }
        output.push('\n');
    }
    Ok(output)
}

fn escape(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn settings(item: &CaptionItem) -> String {
    if !item.layout_enabled {
        return String::new();
    }
    let (align, anchor) = match item.h_align {
        HorizontalAlign::Left => ("left", "line-left"),
        HorizontalAlign::Center => ("center", "center"),
        HorizontalAlign::Right => ("right", "line-right"),
    };
    let line_anchor = match item.v_align {
        VerticalAlign::Top => "start",
        VerticalAlign::Middle => "center",
        VerticalAlign::Bottom => "end",
    };
    let (direction, line, position) = match item.writing_direction {
        CaptionWritingDirection::VerticalRightToLeft => {
            (" vertical:rl", item.position_x, item.position_y)
        }
        CaptionWritingDirection::VerticalLeftToRight => {
            (" vertical:lr", item.position_x, item.position_y)
        }
        _ => ("", item.position_y, item.position_x),
    };
    format!(
        " align:{align} position:{}%,{anchor} line:{}%,{line_anchor}{direction}",
        position.min(100),
        line.min(100)
    )
}

fn edge_css(item: &CaptionItem) -> String {
    let offset = match item.edge_style {
        CaptionEdgeStyle::None => return String::new(),
        CaptionEdgeStyle::HardShadow => "1px 1px 0",
        CaptionEdgeStyle::Bevel => "-1px -1px 0",
        CaptionEdgeStyle::Glow => "0 0 2px",
        CaptionEdgeStyle::SoftShadow => "1px 1px 2px",
    };
    let color = item.edge_color;
    format!(
        " text-shadow: {offset} rgba({}, {}, {}, {});",
        color.r,
        color.g,
        color.b,
        math::opacity(color.a)
    )
}
