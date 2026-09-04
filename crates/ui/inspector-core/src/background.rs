use shrimply_core::timeline_value::{TimelineBool, TimelineStep, TimelineValue};
use shrimply_project::project::{Background, BackgroundGenerator, BackgroundKind, Color, Time};

use crate::{
    ControlKind, InspectorControl, InspectorRuntime, InspectorSection, LayeredState, NumberSpec,
    VideoCard,
};

const GENERATOR_PATH: &str = "/content/generator";
const FIELD_PATH: &str = "/content/generator/";
pub(crate) const INTEGER_EDIT_COMMIT: &str = "edit-background-integer";
pub(crate) const INTEGER_KEYFRAME_COMMIT: &str = "edit-background-integer-keyframes";
pub(crate) const INTEGER_EXPRESSION_COMMIT: &str = "edit-background-integer-expression";

pub fn card(background: &Background, runtime: InspectorRuntime) -> VideoCard {
    let mut section = InspectorSection::default();
    section.add(kind(background.generator.kind()));
    match &background.generator {
        BackgroundGenerator::SolidColor(v) => {
            color(&mut section, "Color", "color", &v.color, runtime)
        }
        BackgroundGenerator::ColorGradient(v) => {
            step(&mut section, "Fill", "mode", &v.mode, runtime);
            color(&mut section, "Color A", "color_a", &v.color_a, runtime);
            color(&mut section, "Color B", "color_b", &v.color_b, runtime);
            vector(
                &mut section,
                "Center",
                "center",
                &v.center,
                (-2.0, 2.0, 0.01),
                false,
                runtime,
            );
            number(
                &mut section,
                "Angle",
                "angle_degrees",
                &v.angle_degrees,
                (-360.0, 360.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Scale",
                "scale",
                &v.scale,
                (0.01, 100.0, 0.01),
                "",
                runtime,
            );
            step(&mut section, "Blend curve", "curve", &v.curve, runtime);
            vector(
                &mut section,
                "Position",
                "position",
                &v.position,
                (-4096.0, 4096.0, 1.0),
                false,
                runtime,
            );
            number(
                &mut section,
                "Color position",
                "cycle_position",
                &v.cycle_position,
                (-10.0, 10.0, 0.01),
                "",
                runtime,
            );
        }
        BackgroundGenerator::Grid(v) => {
            color(
                &mut section,
                "Background",
                "background_color",
                &v.background_color,
                runtime,
            );
            color(
                &mut section,
                "Horizontal",
                "horizontal_color",
                &v.horizontal_color,
                runtime,
            );
            color(
                &mut section,
                "Vertical",
                "vertical_color",
                &v.vertical_color,
                runtime,
            );
            vector(
                &mut section,
                "Spacing",
                "spacing",
                &v.spacing,
                (1.0, 4096.0, 1.0),
                true,
                runtime,
            );
            vector(
                &mut section,
                "Line width",
                "line_width",
                &v.line_width,
                (0.0, 256.0, 0.5),
                true,
                runtime,
            );
            vector(
                &mut section,
                "Position",
                "position",
                &v.position,
                (-4096.0, 4096.0, 1.0),
                false,
                runtime,
            );
            number(
                &mut section,
                "Rotation",
                "rotation_degrees",
                &v.rotation_degrees,
                (-360.0, 360.0, 1.0),
                "",
                runtime,
            );
            step(
                &mut section,
                "Line style",
                "line_style",
                &v.line_style,
                runtime,
            );
            number(
                &mut section,
                "Dash length",
                "dash_length",
                &v.dash_length,
                (0.1, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Dash gap",
                "dash_gap",
                &v.dash_gap,
                (0.1, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Dash position",
                "dash_position",
                &v.dash_position,
                (-4096.0, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Wobble",
                "wobble_amount",
                &v.wobble_amount,
                (0.0, 512.0, 0.5),
                "",
                runtime,
            );
            number(
                &mut section,
                "Wobble scale",
                "wobble_scale",
                &v.wobble_scale,
                (0.1, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Wobble position",
                "wobble_position",
                &v.wobble_position,
                (-20.0, 20.0, 0.01),
                "",
                runtime,
            );
            vector(
                &mut section,
                "Middle padding",
                "middle_padding",
                &v.middle_padding,
                (0.0, 1.0, 0.01),
                true,
                runtime,
            );
            vector(
                &mut section,
                "Padding randomness",
                "padding_randomness",
                &v.padding_randomness,
                (0.0, 1.0, 0.01),
                true,
                runtime,
            );
            integer(&mut section, "Seed", "seed", &v.seed, 0, u32::MAX, runtime);
        }
        BackgroundGenerator::WhiteNoise(v) => {
            step(
                &mut section,
                "Distribution",
                "distribution",
                &v.distribution,
                runtime,
            );
            step(
                &mut section,
                "Coloring",
                "color_mode",
                &v.color_mode,
                runtime,
            );
            color(&mut section, "Color A", "color_a", &v.color_a, runtime);
            color(&mut section, "Color B", "color_b", &v.color_b, runtime);
            integer(
                &mut section,
                "Pixel size",
                "pixel_size",
                &v.pixel_size,
                1,
                512,
                runtime,
            );
            number(
                &mut section,
                "Brightness",
                "brightness",
                &v.brightness,
                (-1.0, 1.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Contrast",
                "contrast",
                &v.contrast,
                (0.0, 8.0, 0.01),
                "",
                runtime,
            );
            boolean(&mut section, "Animated", "animated", &v.animated, runtime);
            number(
                &mut section,
                "Refresh interval",
                "refresh_interval",
                &v.refresh_interval,
                (0.001, 3600.0, 0.01),
                "s",
                runtime,
            );
            integer(&mut section, "Seed", "seed", &v.seed, 0, u32::MAX, runtime);
        }
        BackgroundGenerator::PerlinNoise(v) => {
            step(&mut section, "Mode", "mode", &v.mode, runtime);
            color(&mut section, "Color A", "color_a", &v.color_a, runtime);
            color(&mut section, "Color B", "color_b", &v.color_b, runtime);
            number(
                &mut section,
                "Scale",
                "scale",
                &v.scale,
                (1.0, 8192.0, 1.0),
                "",
                runtime,
            );
            integer(
                &mut section,
                "Octaves",
                "octaves",
                &v.octaves,
                1,
                8,
                runtime,
            );
            number(
                &mut section,
                "Lacunarity",
                "lacunarity",
                &v.lacunarity,
                (1.0, 8.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Persistence",
                "persistence",
                &v.persistence,
                (0.0, 1.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Contrast",
                "contrast",
                &v.contrast,
                (0.0, 8.0, 0.01),
                "",
                runtime,
            );
            vector(
                &mut section,
                "Position",
                "position",
                &v.position,
                (-4096.0, 4096.0, 1.0),
                false,
                runtime,
            );
            number(
                &mut section,
                "Evolution",
                "evolution",
                &v.evolution,
                (-20.0, 20.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Warp amount",
                "warp_amount",
                &v.warp_amount,
                (0.0, 8.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Warp scale",
                "warp_scale",
                &v.warp_scale,
                (1.0, 8192.0, 1.0),
                "",
                runtime,
            );
            integer(&mut section, "Seed", "seed", &v.seed, 0, u32::MAX, runtime);
        }
        BackgroundGenerator::CenteredLines(v) => {
            color(
                &mut section,
                "Background",
                "background_color",
                &v.background_color,
                runtime,
            );
            color(&mut section, "Lines", "line_color", &v.line_color, runtime);
            vector(
                &mut section,
                "Center",
                "center",
                &v.center,
                (-2.0, 2.0, 0.01),
                false,
                runtime,
            );
            let default = NumberSpec::default();
            number(
                &mut section,
                "Rotation",
                "rotation_degrees",
                &v.rotation_degrees,
                (default.minimum, default.maximum, 1.0),
                "°",
                runtime,
            );
            integer(
                &mut section,
                "Line count",
                "line_count",
                &v.line_count,
                1,
                4096,
                runtime,
            );
            number(
                &mut section,
                "Line width",
                "line_width",
                &v.line_width,
                (0.0, 256.0, 0.5),
                "",
                runtime,
            );
            number(
                &mut section,
                "Width randomness",
                "line_width_randomness",
                &v.line_width_randomness,
                (0.0, 1.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Line length",
                "line_length",
                &v.line_length,
                (0.0, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Length randomness",
                "line_length_randomness",
                &v.line_length_randomness,
                (0.0, 1.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Line offset",
                "line_offset",
                &v.line_offset,
                (0.0, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Offset randomness",
                "line_offset_randomness",
                &v.line_offset_randomness,
                (0.0, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Angular randomness",
                "angular_randomness",
                &v.angular_randomness,
                (0.0, 1.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Fade length",
                "fade_length",
                &v.fade_length,
                (0.0, 4096.0, 1.0),
                "",
                runtime,
            );
            integer(&mut section, "Seed", "seed", &v.seed, 0, u32::MAX, runtime);
        }
        BackgroundGenerator::Rainbow(v) => {
            step(&mut section, "Fill", "fill", &v.fill, runtime);
            step(&mut section, "Bands", "bands", &v.bands, runtime);
            integer(
                &mut section,
                "Band count",
                "band_count",
                &v.band_count,
                2,
                256,
                runtime,
            );
            vector(
                &mut section,
                "Center",
                "center",
                &v.center,
                (-2.0, 2.0, 0.01),
                false,
                runtime,
            );
            number(
                &mut section,
                "Angle",
                "angle_degrees",
                &v.angle_degrees,
                (-360.0, 360.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Scale",
                "scale",
                &v.scale,
                (0.01, 100.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Saturation",
                "saturation",
                &v.saturation,
                (0.0, 1.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Brightness",
                "brightness",
                &v.brightness,
                (0.0, 4.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Alpha",
                "alpha",
                &v.alpha,
                (0.0, 1.0, 0.01),
                "",
                runtime,
            );
            vector(
                &mut section,
                "Position",
                "position",
                &v.position,
                (-4096.0, 4096.0, 1.0),
                false,
                runtime,
            );
            number(
                &mut section,
                "Hue position",
                "hue_position",
                &v.hue_position,
                (-10.0, 10.0, 0.01),
                "",
                runtime,
            );
        }
        BackgroundGenerator::Checkerboard(v) => {
            color(&mut section, "Color A", "color_a", &v.color_a, runtime);
            color(&mut section, "Color B", "color_b", &v.color_b, runtime);
            vector(
                &mut section,
                "Cell size",
                "cell_size",
                &v.cell_size,
                (1.0, 4096.0, 1.0),
                true,
                runtime,
            );
            number(
                &mut section,
                "Edge softness",
                "edge_softness",
                &v.edge_softness,
                (0.0, 256.0, 0.5),
                "",
                runtime,
            );
            vector(
                &mut section,
                "Position",
                "position",
                &v.position,
                (-4096.0, 4096.0, 1.0),
                false,
                runtime,
            );
            number(
                &mut section,
                "Rotation",
                "rotation_degrees",
                &v.rotation_degrees,
                (-360.0, 360.0, 1.0),
                "",
                runtime,
            );
        }
        BackgroundGenerator::Voronoi(v) => {
            step(&mut section, "Fill", "fill", &v.fill, runtime);
            step(&mut section, "Metric", "metric", &v.metric, runtime);
            color(&mut section, "Color A", "color_a", &v.color_a, runtime);
            color(&mut section, "Color B", "color_b", &v.color_b, runtime);
            color(
                &mut section,
                "Edge color",
                "edge_color",
                &v.edge_color,
                runtime,
            );
            number(
                &mut section,
                "Cell size",
                "cell_size",
                &v.cell_size,
                (1.0, 4096.0, 1.0),
                "",
                runtime,
            );
            number(
                &mut section,
                "Jitter",
                "jitter",
                &v.jitter,
                (0.0, 2.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Edge width",
                "edge_width",
                &v.edge_width,
                (0.0, 256.0, 0.5),
                "",
                runtime,
            );
            vector(
                &mut section,
                "Position",
                "position",
                &v.position,
                (-4096.0, 4096.0, 1.0),
                false,
                runtime,
            );
            number(
                &mut section,
                "Motion amount",
                "motion_amount",
                &v.motion_amount,
                (0.0, 2.0, 0.01),
                "",
                runtime,
            );
            number(
                &mut section,
                "Motion position",
                "motion_position",
                &v.motion_position,
                (-20.0, 20.0, 0.01),
                "",
                runtime,
            );
            integer(&mut section, "Seed", "seed", &v.seed, 0, u32::MAX, runtime);
        }
        BackgroundGenerator::TestPattern => {}
    }
    VideoCard {
        key: "background",
        title: "Background",
        section,
        reset: None,
        alpha_mask: None,
        preview_facet: None,
        actions: Vec::new(),
    }
    .reset(
        GENERATOR_PATH,
        serde_json::to_value(BackgroundGenerator::default())
            .expect("default background generator must serialize"),
        "reset-background",
    )
}

fn path(field: &str) -> String {
    format!("{FIELD_PATH}{field}")
}

fn kind(current: BackgroundKind) -> InspectorControl {
    crate::selector::selector(
        format!("{GENERATOR_PATH}/kind"),
        "Type",
        kind_key(current),
        BACKGROUND_KINDS
            .iter()
            .map(|&(_, key, label)| (key.to_string(), label.to_string())),
    )
    .immediate_commit("change-background-kind")
}

pub fn kind_key(kind: BackgroundKind) -> &'static str {
    BACKGROUND_KINDS
        .iter()
        .find_map(|&(candidate, key, _)| (candidate == kind).then_some(key))
        .expect("background kind must be presented")
}

const BACKGROUND_KINDS: &[(BackgroundKind, &str, &str)] = &[
    (BackgroundKind::SolidColor, "solid_color", "Solid Color"),
    (
        BackgroundKind::ColorGradient,
        "color_gradient",
        "Color / Gradient",
    ),
    (BackgroundKind::Grid, "grid", "Grid"),
    (BackgroundKind::WhiteNoise, "white_noise", "White Noise"),
    (BackgroundKind::PerlinNoise, "perlin_noise", "Perlin Noise"),
    (
        BackgroundKind::CenteredLines,
        "centered_lines",
        "Centered Lines",
    ),
    (BackgroundKind::Rainbow, "rainbow", "Rainbow"),
    (BackgroundKind::Checkerboard, "checkerboard", "Checkerboard"),
    (BackgroundKind::Voronoi, "voronoi", "Voronoi"),
    (BackgroundKind::TestPattern, "test_pattern", "Test Pattern"),
];

fn number(
    section: &mut InspectorSection,
    label: &str,
    field: &str,
    value: &TimelineValue<f32>,
    range: (f64, f64, f64),
    unit: &'static str,
    runtime: InspectorRuntime,
) {
    let path = path(field);
    let current = value.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    section.add(
        InspectorControl::new(ControlKind::LayeredNumber, &path, label)
            .value(current.to_string())
            .number(NumberSpec {
                minimum: range.0,
                maximum: range.1,
                drag_step: range.2,
                digits: if unit == "s" {
                    3
                } else if range.2 < 1.0 {
                    2
                } else {
                    0
                },
                unit,
            })
            .width_characters(8)
            .layered(&path, LayeredState::from(value))
            .timeline(
                value.id,
                crate::transform::scalar_graph(value, current, runtime),
            )
            .live_commit("edit-background-scalar")
            .timeline_commits("edit-background-scalar", "edit-background-scalar"),
    );
}

fn integer(
    section: &mut InspectorSection,
    label: &str,
    field: &str,
    value: &TimelineValue<u32>,
    minimum: u32,
    maximum: u32,
    runtime: InspectorRuntime,
) {
    let path = path(field);
    section.add(
        InspectorControl::new(ControlKind::LayeredNumber, &path, label)
            .value(
                value
                    .value_at(runtime.local_time.unwrap_or(Time::ZERO))
                    .to_string(),
            )
            .number(NumberSpec {
                minimum: f64::from(minimum),
                maximum: f64::from(maximum),
                drag_step: 1.0,
                digits: 0,
                unit: "",
            })
            .integer()
            .width_characters(8)
            .layered(&path, LayeredState::from(value))
            .timeline(value.id, integer_graph(value, runtime))
            .live_commit(INTEGER_EDIT_COMMIT)
            .timeline_commits(INTEGER_KEYFRAME_COMMIT, INTEGER_EXPRESSION_COMMIT),
    );
}

fn vector(
    section: &mut InspectorSection,
    label: &str,
    field: &str,
    value: &TimelineValue<glam::Vec2>,
    range: (f64, f64, f64),
    lock: bool,
    runtime: InspectorRuntime,
) {
    let path = path(field);
    let current = value.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let control = InspectorControl::new(ControlKind::LayeredVector2, &path, label)
        .components(vec![current.x.to_string(), current.y.to_string()])
        .number(NumberSpec {
            minimum: range.0,
            maximum: range.1,
            drag_step: range.2,
            digits: if range.2 < 1.0 { 2 } else { 0 },
            unit: "",
        })
        .width_characters(7)
        .prefixes(["X", "Y"])
        .layered(&path, LayeredState::from(value))
        .timeline(
            value.id,
            crate::transform::vector_speed_graph(value, runtime),
        )
        .live_commit("edit-background-vector")
        .timeline_commits("edit-background-vector", "edit-background-vector");
    section.add(if lock { control.lock() } else { control });
}

fn color(
    section: &mut InspectorSection,
    label: &str,
    field: &str,
    value: &TimelineValue<Color<u8>>,
    runtime: InspectorRuntime,
) {
    let path = path(field);
    let current = value.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    section.add(
        InspectorControl::new(ControlKind::LayeredColor, &path, label)
            .components(
                [current.r, current.g, current.b, current.a]
                    .map(|channel| channel.to_string())
                    .to_vec(),
            )
            .layered(&path, LayeredState::from(value))
            .timeline(value.id, crate::timeline_color::speed_graph(value, runtime))
            .live_commit("edit-background-color")
            .timeline_commits("edit-background-color", "edit-background-color"),
    );
}

fn boolean(
    section: &mut InspectorSection,
    label: &str,
    field: &str,
    value: &TimelineValue<TimelineBool>,
    runtime: InspectorRuntime,
) {
    let path = path(field);
    section.add(
        InspectorControl::new(ControlKind::LayeredBoolean, &path, label)
            .value(
                value
                    .value_at(runtime.local_time.unwrap_or(Time::ZERO))
                    .get()
                    .to_string(),
            )
            .layered(&path, LayeredState::from(value))
            .timeline(value.id, crate::selector::step_graph(value, runtime))
            .live_commit("edit-background-scalar")
            .timeline_commits("edit-background-scalar", "edit-background-scalar"),
    );
}

fn step<T: TimelineStep>(
    section: &mut InspectorSection,
    label: &str,
    field: &str,
    value: &TimelineValue<T>,
    runtime: InspectorRuntime,
) {
    section.add(
        crate::selector::layered_step_selector(path(field), label, value, runtime)
            .live_commit("edit-background-enum")
            .timeline_commits("edit-background-enum", "edit-background-enum"),
    );
}

pub(crate) fn integer_graph(
    value: &TimelineValue<u32>,
    runtime: InspectorRuntime,
) -> Option<crate::ScalarGraph> {
    let shrimply_core::timeline_value::TimelineBase::Keyframes(keyframes) = &value.base else {
        return None;
    };
    Some(crate::ScalarGraph {
        points: keyframes
            .iter()
            .map(|keyframe| crate::GraphPoint {
                time: keyframe.time,
                value: f64::from(keyframe.value),
            })
            .collect(),
        segments: keyframes
            .windows(2)
            .map(|pair| crate::GraphSegment {
                owner_id: pair[0].id,
                start: pair[0].time,
                end: pair[1].time,
                start_value: f64::from(pair[0].value),
                end_value: f64::from(pair[1].value),
                interpolation: shrimply_core::timeline_value::Interpolation::KEYFRAME
                    .iter()
                    .position(|candidate| *candidate == pair[0].interpolation_to_next)
                    .expect("background interpolation must be available"),
            })
            .collect(),
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}

pub(crate) fn generator(
    item: &shrimply_project::project::VideoItem,
) -> Option<&BackgroundGenerator> {
    let shrimply_project::project::VideoItemContent::Background(background) = &item.content else {
        return None;
    };
    Some(&background.generator)
}

pub(crate) fn number_value(
    item: &shrimply_project::project::VideoItem,
    id: uuid::Uuid,
) -> Option<&TimelineValue<f32>> {
    generator(item)?.number(id)
}
pub(crate) fn integer_value(
    item: &shrimply_project::project::VideoItem,
    id: uuid::Uuid,
) -> Option<&TimelineValue<u32>> {
    generator(item)?.integer(id)
}
pub(crate) fn vector_value(
    item: &shrimply_project::project::VideoItem,
    id: uuid::Uuid,
) -> Option<&TimelineValue<glam::Vec2>> {
    generator(item)?.number2(id)
}
pub(crate) fn color_value(
    item: &shrimply_project::project::VideoItem,
    id: uuid::Uuid,
) -> Option<&TimelineValue<Color<u8>>> {
    generator(item)?.color(id)
}

pub(crate) fn set_kind(
    item: &mut shrimply_project::project::VideoItem,
    path: &str,
    key: &str,
) -> Option<Result<bool, String>> {
    if path != "/content/generator/kind" {
        return None;
    }
    let shrimply_project::project::VideoItemContent::Background(background) = &mut item.content
    else {
        return Some(Err(
            "background generator is no longer available".to_string()
        ));
    };
    Some(
        BACKGROUND_KINDS
            .iter()
            .find_map(|&(kind, candidate, _)| (candidate == key).then_some(kind))
            .ok_or_else(|| format!("unknown background kind: {key}"))
            .map(|kind| {
                if background.generator.kind() == kind {
                    false
                } else {
                    background.generator = kind.generator();
                    true
                }
            }),
    )
}
