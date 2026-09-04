use shrimply_core::{
    Color, FontFamily, FontVariation,
    timeline_value::{TimelineStep, TimelineValue},
};
use shrimply_project::project::{
    CanvasSize, TEXT_APPEARANCE_PREVIEW_FACET, TextItem, Time, VideoItem, VideoItemContent,
};

use crate::{
    ControlKind, InspectorControl, InspectorRuntime, InspectorSection, LayeredState, NumberSpec,
    TextKeyframeCommits, VideoCard,
};

pub const TEXT_EDIT_COMMIT: &str = "edit-text-item";
pub const TEXT_EXPRESSION_COMMIT: &str = "text-expression";
pub const TEXT_KEYFRAME_COMMITS: TextKeyframeCommits = TextKeyframeCommits {
    toggle: "text-keyframes",
    add: "add-text-keyframe",
    delete: "delete-text-keyframe",
    move_keyframe: "move-text-keyframe",
    paste: "paste-text-keyframes",
    interpolation: "text-interpolation",
    text_interpolation: "text-change-interpolation",
};
pub const FONT_EDIT_COMMIT: &str = "edit-text-font";
pub const FONT_VARIATION_EDIT_COMMIT: &str = "edit-text-font-variation";
pub const SCALAR_EDIT_COMMIT: &str = "edit-text-scalar";
pub const VECTOR_EDIT_COMMIT: &str = "edit-text-background-padding";
pub const COLOR_EDIT_COMMIT: &str = "edit-text-color";

pub const TEXT_PATH: &str = "/content/text";
pub const FONT_FAMILIES_PATH: &str = "/content/font_families";
pub const HORIZONTAL_ALIGN_PATH: &str = "/content/h_align";
pub const VERTICAL_ALIGN_PATH: &str = "/content/v_align";
pub const DIRECTION_PATH: &str = "/content/direction";
pub const FONT_STYLE_PATH: &str = "/content/font_style";

const PERCENT: f64 = 100.0;

pub fn cards(
    text: &TextItem,
    canvas_size: CanvasSize,
    runtime: InspectorRuntime,
    default_font: Option<&FontFamily>,
) -> [VideoCard; 2] {
    let mut default = default_text(canvas_size);
    if let Some(default_font) = default_font {
        default.font_families = vec![default_font.clone()];
    }
    [
        content_card(text, &default, runtime),
        appearance_card(text, &default, runtime),
    ]
}

fn content_card(text: &TextItem, default: &TextItem, runtime: InspectorRuntime) -> VideoCard {
    let mut section = InspectorSection::default();
    section.add(text_control(TEXT_PATH, "Text", &text.text, runtime));
    section.add(font_families_control(&text.font_families));
    section.add(step_control(
        HORIZONTAL_ALIGN_PATH,
        "Align",
        &text.h_align,
        runtime,
        "edit-text-horizontal-align",
    ));
    section.add(step_control(
        VERTICAL_ALIGN_PATH,
        "Vertical",
        &text.v_align,
        runtime,
        "edit-text-vertical-align",
    ));
    section.add(step_control(
        DIRECTION_PATH,
        "Direction",
        &text.direction,
        runtime,
        "edit-text-direction",
    ));
    VideoCard::new("text", "Text", section).reset_fields(
        [
            (FONT_FAMILIES_PATH, json(&default.font_families)),
            (HORIZONTAL_ALIGN_PATH, json(&default.h_align)),
            (VERTICAL_ALIGN_PATH, json(&default.v_align)),
            (DIRECTION_PATH, json(&default.direction)),
        ],
        "reset-text-content",
    )
}

fn appearance_card(text: &TextItem, default: &TextItem, runtime: InspectorRuntime) -> VideoCard {
    let mut section = InspectorSection::default();
    section.add(number(
        "/content/font_size",
        "Font size",
        &text.font_size,
        runtime,
        pixels(1.0),
    ));
    section.add(number(
        "/content/font_weight",
        "Font weight",
        &text.font_weight,
        runtime,
        NumberSpec {
            minimum: 1.0,
            maximum: 1000.0,
            drag_step: 10.0,
            digits: 0,
            unit: "",
        },
    ));
    section.add(number(
        "/content/tracking",
        "Tracking",
        &text.tracking,
        runtime,
        NumberSpec {
            drag_step: 0.1,
            digits: 1,
            unit: "px",
            ..NumberSpec::default()
        },
    ));
    section.add(
        number(
            "/content/line_height",
            "Line height",
            &text.line_height,
            runtime,
            NumberSpec {
                minimum: 1.0,
                drag_step: 1.0,
                digits: 0,
                unit: "%",
                ..NumberSpec::default()
            },
        )
        .store_multiplier(PERCENT.recip()),
    );
    section.add(step_control(
        FONT_STYLE_PATH,
        "Style",
        &text.font_style,
        runtime,
        "edit-text-font-style",
    ));
    for axis in font_axes(text) {
        let current = text
            .font_variations
            .iter()
            .find(|variation| variation.axis == axis.tag)
            .map_or(axis.default, |variation| variation.value);
        section.add(
            InspectorControl::new(
                ControlKind::Number,
                format!("/content/font_variations/{}", axis.tag),
                axis.tag.clone(),
            )
            .tooltip(format!("{} variation axis", axis.tag))
            .value(current.to_string())
            .number(NumberSpec {
                minimum: f64::from(axis.minimum),
                maximum: f64::from(axis.maximum),
                drag_step: f64::from(((axis.maximum - axis.minimum).abs() / 100.0).max(0.01)),
                digits: 2,
                unit: "",
            })
            .immediate_commit(FONT_VARIATION_EDIT_COMMIT),
        );
    }
    section.add(color("/content/color", "Color", &text.color, runtime));
    section.add(color(
        "/content/background_color",
        "Background fill",
        &text.background_color,
        runtime,
    ));
    section.add(number(
        "/content/background_roundness",
        "Background roundness",
        &text.background_roundness,
        runtime,
        pixels(0.0),
    ));
    section.add(vector(
        "/content/background_padding",
        "Background padding",
        &text.background_padding,
        runtime,
    ));
    section.add(color(
        "/content/outline_color",
        "Outline",
        &text.outline_color,
        runtime,
    ));
    section.add(number(
        "/content/outline_width",
        "Outline width",
        &text.outline_width,
        runtime,
        pixels(0.0),
    ));
    section.add(color(
        "/content/shadow_color",
        "Shadow color",
        &text.shadow_color,
        runtime,
    ));
    section.add(number(
        "/content/shadow_distance",
        "Shadow distance",
        &text.shadow_distance,
        runtime,
        pixels(0.0),
    ));
    section.add(
        number(
            "/content/shadow_direction_degrees",
            "Shadow direction",
            &text.shadow_direction_degrees,
            runtime,
            NumberSpec {
                drag_step: 1.0,
                digits: 0,
                unit: "deg",
                ..NumberSpec::default()
            },
        )
        .rotating_icon("rotation.svg", 90.0),
    );
    section.add(number(
        "/content/shadow_width",
        "Shadow width",
        &text.shadow_width,
        runtime,
        pixels(0.0),
    ));
    section.add(number(
        "/content/shadow_blur",
        "Shadow blur",
        &text.shadow_blur,
        runtime,
        pixels(0.0),
    ));

    VideoCard::new("text-appearance", "Appearance", section)
        .reset_fields(
            [
                ("/content/font_size", json(&default.font_size)),
                ("/content/font_weight", json(&default.font_weight)),
                ("/content/tracking", json(&default.tracking)),
                ("/content/line_height", json(&default.line_height)),
                (FONT_STYLE_PATH, json(&default.font_style)),
                ("/content/font_variations", json(&default.font_variations)),
                ("/content/color", json(&default.color)),
                ("/content/background_color", json(&default.background_color)),
                (
                    "/content/background_roundness",
                    json(&default.background_roundness),
                ),
                (
                    "/content/background_padding",
                    json(&default.background_padding),
                ),
                ("/content/outline_color", json(&default.outline_color)),
                ("/content/outline_width", json(&default.outline_width)),
                ("/content/shadow_color", json(&default.shadow_color)),
                ("/content/shadow_distance", json(&default.shadow_distance)),
                (
                    "/content/shadow_direction_degrees",
                    json(&default.shadow_direction_degrees),
                ),
                ("/content/shadow_width", json(&default.shadow_width)),
                ("/content/shadow_blur", json(&default.shadow_blur)),
            ],
            "reset-text-appearance",
        )
        .preview_facet(TEXT_APPEARANCE_PREVIEW_FACET)
}

fn text_control(
    path: &'static str,
    label: &'static str,
    timeline: &TimelineValue<String>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    InspectorControl::new(ControlKind::LayeredText, path, label)
        .value(crate::timeline_text::value_at(
            timeline,
            runtime.local_time.unwrap_or(Time::ZERO),
        ))
        .layered(path, LayeredState::from(timeline))
        .timeline(
            timeline.id,
            crate::timeline_text::speed_graph(timeline, runtime),
        )
        .live_commit(TEXT_EDIT_COMMIT)
        .timeline_commits(TEXT_KEYFRAME_COMMITS.toggle, TEXT_EXPRESSION_COMMIT)
        .text_keyframe_commits(TEXT_KEYFRAME_COMMITS)
}

fn step_control<T: TimelineStep>(
    path: &'static str,
    label: &'static str,
    timeline: &TimelineValue<T>,
    runtime: InspectorRuntime,
    commit: &'static str,
) -> InspectorControl {
    crate::selector::layered_step_selector(path, label, timeline, runtime)
        .live_commit(commit)
        .timeline_commits(commit, commit)
}

fn number(
    path: &'static str,
    label: &'static str,
    timeline: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    spec: NumberSpec,
) -> InspectorControl {
    let current = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    InspectorControl::new(ControlKind::LayeredNumber, path, label)
        .value(current.to_string())
        .number(spec)
        .width_characters(9)
        .layered(path, LayeredState::from(timeline))
        .timeline(
            timeline.id,
            crate::transform::scalar_graph(timeline, current, runtime),
        )
        .live_commit(SCALAR_EDIT_COMMIT)
        .timeline_commits(SCALAR_EDIT_COMMIT, SCALAR_EDIT_COMMIT)
}

fn vector(
    path: &'static str,
    label: &'static str,
    timeline: &TimelineValue<glam::Vec2>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    let current = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    InspectorControl::new(ControlKind::LayeredVector2, path, label)
        .components([current.x.to_string(), current.y.to_string()].into())
        .number(NumberSpec {
            minimum: 0.0,
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        })
        .width_characters(7)
        .prefixes(["X", "Y"])
        .layered(path, LayeredState::from(timeline))
        .timeline(
            timeline.id,
            crate::transform::vector_speed_graph(timeline, runtime),
        )
        .live_commit(VECTOR_EDIT_COMMIT)
        .timeline_commits(VECTOR_EDIT_COMMIT, VECTOR_EDIT_COMMIT)
}

fn color(
    path: &'static str,
    label: &'static str,
    timeline: &TimelineValue<Color<u8>>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    InspectorControl::new(ControlKind::LayeredColor, path, label)
        .components(
            [
                value.r.to_string(),
                value.g.to_string(),
                value.b.to_string(),
                value.a.to_string(),
            ]
            .into(),
        )
        .layered(path, LayeredState::from(timeline))
        .timeline(
            timeline.id,
            crate::timeline_color::speed_graph(timeline, runtime),
        )
        .live_commit(COLOR_EDIT_COMMIT)
        .timeline_commits(COLOR_EDIT_COMMIT, COLOR_EDIT_COMMIT)
}

fn font_families_control(selected: &[FontFamily]) -> InspectorControl {
    InspectorControl::new(ControlKind::FontFamilies, FONT_FAMILIES_PATH, "Fonts")
        .value(serde_json::to_string(selected).expect("text font families must serialize"))
        .immediate_commit(FONT_EDIT_COMMIT)
}

pub fn font_axes(text: &TextItem) -> Vec<crate::font_cache::FontAxis> {
    let Some(family) = text.font_families.first() else {
        return Vec::new();
    };
    let capabilities = match family {
        FontFamily::GoogleFonts { name } => {
            crate::font_cache::cached_capabilities(name).unwrap_or_default()
        }
        FontFamily::Local { name } => crate::font_cache::local_capabilities(name),
    };
    capabilities
        .axes
        .into_iter()
        .filter(|axis| !matches!(axis.tag.as_str(), "wght" | "ital"))
        .collect()
}

fn pixels(minimum: f64) -> NumberSpec {
    NumberSpec {
        minimum,
        drag_step: 1.0,
        digits: 0,
        unit: "px",
        ..NumberSpec::default()
    }
}

fn default_text(canvas_size: CanvasSize) -> TextItem {
    let VideoItemContent::Text(text) =
        VideoItem::text_item(canvas_size, Time::ZERO, Time::ZERO).content
    else {
        unreachable!("text item constructor must create text content")
    };
    *text
}

fn json(value: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).expect("text inspector value must serialize")
}

pub fn is_timeline_path(path: &str) -> bool {
    matches!(
        path,
        TEXT_PATH
            | HORIZONTAL_ALIGN_PATH
            | VERTICAL_ALIGN_PATH
            | DIRECTION_PATH
            | FONT_STYLE_PATH
            | "/content/font_size"
            | "/content/font_weight"
            | "/content/tracking"
            | "/content/line_height"
            | "/content/color"
            | "/content/background_color"
            | "/content/background_roundness"
            | "/content/background_padding"
            | "/content/outline_color"
            | "/content/outline_width"
            | "/content/shadow_color"
            | "/content/shadow_distance"
            | "/content/shadow_direction_degrees"
            | "/content/shadow_width"
            | "/content/shadow_blur"
    )
}

pub(crate) fn text_value<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<&'a TimelineValue<String>> {
    let VideoItemContent::Text(text) = &item.content else {
        return None;
    };
    (path == TEXT_PATH && text.text.id == id).then_some(&text.text)
}

pub(crate) fn color_value<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<&'a TimelineValue<Color<u8>>> {
    let VideoItemContent::Text(text) = &item.content else {
        return None;
    };
    let value = match path {
        "/content/color" => &text.color,
        "/content/background_color" => &text.background_color,
        "/content/outline_color" => &text.outline_color,
        "/content/shadow_color" => &text.shadow_color,
        _ => return None,
    };
    (value.id == id).then_some(value)
}

pub fn set_field(item: &mut VideoItem, path: &str, value: &str) -> Option<Result<bool, String>> {
    let VideoItemContent::Text(text) = &mut item.content else {
        return None;
    };
    if path == FONT_FAMILIES_PATH {
        return Some(set_font_families(text, value));
    }
    let axis = path.strip_prefix("/content/font_variations/")?;
    Some(set_font_variation(text, axis, value))
}

fn set_font_families(text: &mut TextItem, value: &str) -> Result<bool, String> {
    let mut next: Vec<FontFamily> =
        serde_json::from_str(value).map_err(|error| format!("invalid font families: {error}"))?;
    next.retain(|family| !family.name().trim().is_empty());
    for family in &mut next {
        match family {
            FontFamily::Local { name } | FontFamily::GoogleFonts { name } => {
                *name = name.trim().to_string();
            }
        }
    }
    let mut names = hashbrown::HashSet::new();
    next.retain(|family| names.insert(family.name().to_lowercase()));
    if text.font_families == next {
        return Ok(false);
    }
    text.font_families = next;
    Ok(true)
}

fn set_font_variation(text: &mut TextItem, axis: &str, value: &str) -> Result<bool, String> {
    let next = value
        .parse::<f32>()
        .map_err(|_| format!("invalid font variation value: {value}"))?;
    let specification = font_axes(text)
        .into_iter()
        .find(|specification| specification.tag == axis)
        .ok_or_else(|| format!("font variation axis is no longer available: {axis}"))?;
    if !next.is_finite() || !(specification.minimum..=specification.maximum).contains(&next) {
        return Err(format!(
            "font variation {axis} must be between {} and {}",
            specification.minimum, specification.maximum
        ));
    }
    if let Some(variation) = text
        .font_variations
        .iter_mut()
        .find(|variation| variation.axis == axis)
    {
        if variation.value.to_bits() == next.to_bits() {
            return Ok(false);
        }
        variation.value = next;
    } else {
        text.font_variations.push(FontVariation {
            axis: axis.to_string(),
            value: next,
        });
    }
    Ok(true)
}
