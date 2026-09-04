use std::path::Path;

use shrimply_core::{
    Color,
    timeline_value::{TimelineBase, TimelineBool, TimelineValue, TimelineValueType},
};
use shrimply_project::project::{
    CanvasSize, PaintDrawing, PaintItem, PaintPaletteEntry, PaintStrokeOptions, PaintTaper,
    PaintTextureOptions, PaintTransform, Project, Time, VideoItem, VideoItemContent,
};

use crate::{
    ControlKind, GraphPoint, GraphSegment, InspectorCommit, InspectorControl,
    InspectorControlAction, InspectorController, InspectorRuntime, InspectorSection,
    InspectorTarget, LayeredState, NumberSpec, ScalarGraph, VideoCard,
};

#[derive(Clone, Copy)]
pub struct PaintCardMetadata {
    pub key: &'static str,
    pub title: &'static str,
}

const STROKE_TRANSFORM_ID: &str = "paint-stroke-transform";

pub const PALETTE_CARD: PaintCardMetadata = PaintCardMetadata {
    key: "paint-palette",
    title: "Textures",
};
pub const STROKE_CARD: PaintCardMetadata = PaintCardMetadata {
    key: "paint-strokes",
    title: "Strokes",
};
pub const STROKE_TRANSFORM_CARD: PaintCardMetadata = PaintCardMetadata {
    key: STROKE_TRANSFORM_ID,
    title: "Stroke Transform",
};

pub const PALETTE_COLOR_COMMIT: &str = "paint-palette-color";
pub const TEXTURE_SCALE_COMMIT: &str = "paint-texture-scale";
pub const TEXTURE_ROTATION_COMMIT: &str = "paint-texture-rotation";
pub const DRAWING_LIVE_COMMIT: &str = "paint-drawing";
pub const DRAWING_KEYFRAME_COMMIT: &str = "paint-drawing-keyframes";
pub const DRAWING_EXPRESSION_COMMIT: &str = "paint-drawing-expression";
pub const STROKE_WIDTH_COMMIT: &str = "paint-stroke-width";
pub const STROKE_THINNING_COMMIT: &str = "paint-stroke-thinning";
pub const STROKE_SMOOTHING_COMMIT: &str = "paint-stroke-smoothing";
pub const STROKE_STREAMLINE_COMMIT: &str = "paint-stroke-streamline";
pub const STROKE_SIMPLIFICATION_COMMIT: &str = "paint-stroke-simplification";
pub const STROKE_SUBDIVISION_COMMIT: &str = "paint-stroke-subdivision";
pub const STROKE_START_CAP_COMMIT: &str = "paint-stroke-start-cap";
pub const STROKE_END_CAP_COMMIT: &str = "paint-stroke-end-cap";
pub const STROKE_START_TAPER_COMMIT: &str = "paint-stroke-start-taper";
pub const STROKE_END_TAPER_COMMIT: &str = "paint-stroke-end-taper";
pub const STROKE_START_TAPER_DISTANCE_COMMIT: &str = "paint-stroke-start-taper-distance";
pub const STROKE_END_TAPER_DISTANCE_COMMIT: &str = "paint-stroke-end-taper-distance";
pub const STROKE_TRANSFORM_COMMIT: &str = STROKE_TRANSFORM_ID;

pub const DRAWING_PATH: &str = "/content/drawing";
const PALETTE_PATH: &str = "/content/palette";
const STROKE_PATH: &str = "/content/stroke";
const STROKE_TRANSFORM_PATH: &str = "/content/stroke_transform";
const MIN_TEXTURE_SCALE: f64 = 0.01;

pub fn cards(
    paint: &PaintItem,
    canvas_size: CanvasSize,
    runtime: InspectorRuntime,
) -> Vec<VideoCard> {
    vec![
        palette_card(paint, runtime),
        stroke_card(paint, runtime),
        stroke_transform_card(paint, canvas_size, runtime),
    ]
}

fn palette_card(paint: &PaintItem, runtime: InspectorRuntime) -> VideoCard {
    let mut section = InspectorSection::default();
    for (index, entry) in paint.palette.iter().enumerate() {
        let color_path = format!("{PALETTE_PATH}/{index}/color");
        let color = entry
            .color
            .value_at(runtime.local_time.unwrap_or(Time::ZERO));
        let mut color_control = InspectorControl::new(
            ControlKind::LayeredColor,
            &color_path,
            format!("Color {}", index + 1),
        )
        .components(vec![
            color.r.to_string(),
            color.g.to_string(),
            color.b.to_string(),
            color.a.to_string(),
        ])
        .layered(&color_path, LayeredState::from(&entry.color))
        .timeline(entry.color.id, color_graph(&entry.color, runtime))
        .live_commit(PALETTE_COLOR_COMMIT)
        .timeline_commits(PALETTE_COLOR_COMMIT, PALETTE_COLOR_COMMIT)
        .action(InspectorControlAction::RemovePaintPaletteColor {
            color_id: entry.color.id,
        })
        .action_sensitive(paint.palette.len() > 1)
        .tooltip("Remove color");
        color_control.prefix_icon = "user-trash-symbolic".to_string();
        section.add(color_control);
        texture_controls(&mut section, index, entry, runtime);
    }
    let mut add = InspectorControl::new(ControlKind::Action, "/content/palette/add", "")
        .value("Add")
        .action(InspectorControlAction::AddPaintPaletteColor);
    add.prefix_icon = "list-add-symbolic".to_string();
    section.add(add);
    VideoCard::new(PALETTE_CARD.key, PALETTE_CARD.title, section)
        .reset(
            PALETTE_PATH,
            json(&PaintItem::default().palette),
            "reset-paint-palette",
        )
        .paint_palette_reset()
}

fn texture_controls(
    section: &mut InspectorSection,
    index: usize,
    entry: &PaintPaletteEntry,
    runtime: InspectorRuntime,
) {
    let base = format!("{PALETTE_PATH}/{index}/texture");
    let filename = entry
        .texture
        .as_ref()
        .and_then(|texture| texture.image_path.file_name())
        .and_then(|filename| filename.to_str())
        .unwrap_or("None");
    let choose_label = if entry.texture.is_some() {
        "Replace"
    } else {
        "Select"
    };
    let mut select =
        InspectorControl::new(ControlKind::Action, format!("{base}/select"), "Texture")
            .value(if entry.texture.is_some() {
                filename
            } else {
                choose_label
            })
            .components(vec![filename.to_string(), choose_label.to_string()])
            .tooltip(
                entry
                    .texture
                    .as_ref()
                    .and_then(|texture| texture.image_path.to_str())
                    .unwrap_or_default(),
            )
            .action(InspectorControlAction::SelectPaintTexture {
                color_id: entry.color.id,
            })
            .secondary_action(InspectorControlAction::ClearPaintTexture {
                color_id: entry.color.id,
            });
    select.prefix_icon = "folder-open-symbolic".to_string();
    select.action_icon = "window-close-symbolic".to_string();
    select.action_tooltip = "Clear texture".to_string();
    if entry.texture.is_none() {
        select.action_icon.clear();
    }
    section.add(select);
    if let Some(texture) = &entry.texture {
        section.add(number(
            format!("{base}/repeat_scale"),
            "Texture scale",
            &texture.repeat_scale,
            runtime,
            NumberSpec {
                minimum: MIN_TEXTURE_SCALE,
                drag_step: 0.01,
                digits: 2,
                ..NumberSpec::default()
            },
            TEXTURE_SCALE_COMMIT,
        ));
        section.add(
            number(
                format!("{base}/rotation_degrees"),
                "Texture rotation",
                &texture.rotation_degrees,
                runtime,
                NumberSpec {
                    drag_step: 0.1,
                    digits: 1,
                    unit: "°",
                    ..NumberSpec::default()
                },
                TEXTURE_ROTATION_COMMIT,
            )
            .rotating_icon("arrow3-up-symbolic", 0.0),
        );
    }
}

fn stroke_card(paint: &PaintItem, runtime: InspectorRuntime) -> VideoCard {
    let value = &paint.stroke;
    let mut section = InspectorSection::default();
    let drawing = paint
        .drawing
        .value_at(runtime.local_time.unwrap_or(Time::ZERO));
    section.add(
        InspectorControl::new(ControlKind::LayeredDrawing, DRAWING_PATH, "Drawing")
            .value(format!(
                "{} strokes, {} fills",
                drawing.strokes.len(),
                drawing.fills.len()
            ))
            .layered(DRAWING_PATH, LayeredState::from(&paint.drawing))
            .timeline(paint.drawing.id, drawing_graph(&paint.drawing, runtime))
            .live_commit(DRAWING_LIVE_COMMIT)
            .timeline_commits(DRAWING_KEYFRAME_COMMIT, DRAWING_EXPRESSION_COMMIT),
    );
    for (field, label, timeline, spec, commit) in [
        (
            "width",
            "Width",
            &value.width,
            pixels_spec(),
            STROKE_WIDTH_COMMIT,
        ),
        (
            "thinning",
            "Thinning",
            &value.thinning,
            factor_spec(),
            STROKE_THINNING_COMMIT,
        ),
        (
            "smoothing",
            "Smoothing",
            &value.smoothing,
            factor_spec(),
            STROKE_SMOOTHING_COMMIT,
        ),
        (
            "streamline",
            "Streamline",
            &value.streamline,
            factor_spec(),
            STROKE_STREAMLINE_COMMIT,
        ),
        (
            "simplification_tolerance",
            "Simplify tolerance",
            &value.simplification_tolerance,
            pixels_spec(),
            STROKE_SIMPLIFICATION_COMMIT,
        ),
        (
            "maximum_subdivision_spacing",
            "Subdivision spacing",
            &value.maximum_subdivision_spacing,
            pixels_spec(),
            STROKE_SUBDIVISION_COMMIT,
        ),
    ] {
        section.add(number(
            format!("{STROKE_PATH}/{field}"),
            label,
            timeline,
            runtime,
            spec,
            commit,
        ));
    }
    stroke_end_controls(&mut section, "start", "Start", &value.start, runtime);
    stroke_end_controls(&mut section, "end", "End", &value.end, runtime);
    VideoCard::new(STROKE_CARD.key, STROKE_CARD.title, section).reset(
        STROKE_PATH,
        json(&PaintStrokeOptions::default()),
        "reset-paint-strokes",
    )
}

fn stroke_end_controls(
    section: &mut InspectorSection,
    field: &str,
    label: &str,
    value: &shrimply_project::project::PaintStrokeEndOptions,
    runtime: InspectorRuntime,
) {
    let base = format!("{STROKE_PATH}/{field}");
    section.add(boolean(
        format!("{base}/cap"),
        format!("{label} cap"),
        &value.cap,
        runtime,
        if field == "start" {
            STROKE_START_CAP_COMMIT
        } else {
            STROKE_END_CAP_COMMIT
        },
    ));
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/taper"),
            format!("{label} taper"),
            &value.taper,
            runtime,
        )
        .live_commit(if field == "start" {
            STROKE_START_TAPER_COMMIT
        } else {
            STROKE_END_TAPER_COMMIT
        }),
    );
    if value
        .taper
        .value_at(runtime.local_time.unwrap_or(Time::ZERO))
        == PaintTaper::Distance
    {
        section.add(number(
            format!("{base}/taper_distance"),
            format!("{label} taper distance"),
            &value.taper_distance,
            runtime,
            pixels_spec(),
            if field == "start" {
                STROKE_START_TAPER_DISTANCE_COMMIT
            } else {
                STROKE_END_TAPER_DISTANCE_COMMIT
            },
        ));
    }
}

fn stroke_transform_card(
    paint: &PaintItem,
    canvas_size: CanvasSize,
    runtime: InspectorRuntime,
) -> VideoCard {
    let value = &paint.stroke_transform;
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("position", "Position", &value.position),
        ("anchor", "Anchor", &value.anchor),
        ("scale", "Scale", &value.scale),
    ] {
        let path = format!("{STROKE_TRANSFORM_PATH}/{field}");
        let current = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
        let mut control = InspectorControl::new(ControlKind::LayeredVector2, &path, label)
            .components(vec![current.x.to_string(), current.y.to_string()])
            .prefixes(["X", "Y"])
            .number(NumberSpec {
                minimum: if field == "scale" {
                    0.0
                } else {
                    NumberSpec::default().minimum
                },
                drag_step: if field == "scale" { 0.01 } else { 1.0 },
                digits: if field == "scale" { 2 } else { 0 },
                unit: if field == "position" || field == "anchor" {
                    "px"
                } else {
                    "x"
                },
                ..NumberSpec::default()
            })
            .width_characters(7)
            .layered(&path, LayeredState::from(timeline))
            .timeline(
                timeline.id,
                crate::transform::vector_speed_graph(timeline, runtime),
            )
            .live_commit(STROKE_TRANSFORM_COMMIT);
        control.lock = field == "scale";
        section.add(control);
    }
    let path = format!("{STROKE_TRANSFORM_PATH}/rotation_degrees");
    section.add(
        number(
            &path,
            "Rotation",
            &value.rotation_degrees,
            runtime,
            NumberSpec {
                drag_step: 0.1,
                digits: 1,
                unit: "°",
                ..NumberSpec::default()
            },
            STROKE_TRANSFORM_COMMIT,
        )
        .rotating_icon("arrow3-up-symbolic", 0.0),
    );
    VideoCard::new(
        STROKE_TRANSFORM_CARD.key,
        STROKE_TRANSFORM_CARD.title,
        section,
    )
    .reset(
        STROKE_TRANSFORM_PATH,
        json(&PaintTransform::fill(canvas_size)),
        "reset-paint-stroke-transform",
    )
}

fn number(
    path: impl Into<String>,
    label: impl Into<String>,
    timeline: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    spec: NumberSpec,
    commit: &'static str,
) -> InspectorControl {
    let path = path.into();
    InspectorControl::new(ControlKind::LayeredNumber, &path, label)
        .value(
            timeline
                .value_at(runtime.local_time.unwrap_or(Time::ZERO))
                .to_string(),
        )
        .number(spec)
        .width_characters(9)
        .layered(&path, LayeredState::from(timeline))
        .timeline(
            timeline.id,
            crate::transform::scalar_graph(
                timeline,
                timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO)),
                runtime,
            ),
        )
        .live_commit(commit)
}

fn boolean(
    path: impl Into<String>,
    label: impl Into<String>,
    timeline: &TimelineValue<TimelineBool>,
    runtime: InspectorRuntime,
    commit: &'static str,
) -> InspectorControl {
    let path = path.into();
    InspectorControl::new(ControlKind::LayeredBoolean, &path, label)
        .value(
            timeline
                .value_at(runtime.local_time.unwrap_or(Time::ZERO))
                .get()
                .to_string(),
        )
        .layered(&path, LayeredState::from(timeline))
        .timeline(timeline.id, crate::selector::step_graph(timeline, runtime))
        .live_commit(commit)
}

fn pixels_spec() -> NumberSpec {
    NumberSpec {
        minimum: 0.0,
        drag_step: 1.0,
        digits: 1,
        unit: "px",
        ..NumberSpec::default()
    }
}

fn factor_spec() -> NumberSpec {
    NumberSpec {
        minimum: 0.0,
        maximum: 1.0,
        drag_step: 0.01,
        digits: 2,
        unit: "",
    }
}

pub fn drawing_graph(
    value: &TimelineValue<PaintDrawing>,
    runtime: InspectorRuntime,
) -> Option<ScalarGraph> {
    let TimelineBase::Keyframes(keyframes) = &value.base else {
        return None;
    };
    Some(ScalarGraph {
        points: keyframes
            .iter()
            .map(|keyframe| GraphPoint {
                time: keyframe.time,
                value: 0.0,
            })
            .collect(),
        segments: keyframes
            .windows(2)
            .filter_map(|pair| {
                let seconds = pair[1].time.signed_sub(pair[0].time).as_secs_f64();
                (seconds > f64::EPSILON).then(|| GraphSegment {
                    owner_id: pair[0].id,
                    start: pair[0].time,
                    end: pair[1].time,
                    start_value: 1.0 / seconds,
                    end_value: 1.0 / seconds,
                    interpolation: shrimply_core::timeline_value::Interpolation::KEYFRAME
                        .iter()
                        .position(|candidate| *candidate == pair[0].interpolation_to_next)
                        .expect("paint drawing interpolation must be available"),
                })
            })
            .collect(),
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}

fn color_graph(value: &TimelineValue<Color<u8>>, runtime: InspectorRuntime) -> Option<ScalarGraph> {
    crate::timeline_color::speed_graph(value, runtime)
}

pub fn is_timeline_path(path: &str) -> bool {
    path == DRAWING_PATH
        || path.starts_with(STROKE_PATH)
        || path.starts_with(STROKE_TRANSFORM_PATH)
        || path.starts_with(PALETTE_PATH)
}

pub(crate) fn color_value<'a>(
    item: &'a VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<Color<u8>>> {
    let VideoItemContent::Paint(paint) = &item.content else {
        return None;
    };
    let index = path
        .strip_prefix(PALETTE_PATH)?
        .strip_prefix('/')?
        .strip_suffix("/color")?
        .parse::<usize>()
        .ok()?;
    paint
        .palette
        .get(index)
        .map(|entry| &entry.color)
        .filter(|value| value.id == timeline_id)
}

pub(crate) fn paint_taper_path(path: &str) -> bool {
    matches!(
        path,
        "/content/stroke/start/taper" | "/content/stroke/end/taper"
    )
}

impl InspectorController {
    pub fn reset_paint_palette(&self, target: &InspectorTarget) -> Result<(), String> {
        self.update_paint(target, "reset-paint-palette", |paint| {
            paint.palette = PaintItem::default().palette;
            let last = paint.palette.len() - 1;
            visit_drawings_mut(&mut paint.drawing, |drawing| {
                for stroke in &mut drawing.strokes {
                    stroke.color_index = stroke.color_index.min(last);
                }
                for fill in &mut drawing.fills {
                    fill.color_index = fill.color_index.min(last);
                }
            });
            true
        })
    }

    pub fn set_paint_drawing_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.paint_drawing_timeline(target, timeline_id)?;
        let current = value.value_at(runtime.local_time.unwrap_or(Time::ZERO));
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "paint drawing keyframe time is unavailable".to_string())?;
        if !crate::keyframe_model::set_keyframes_enabled(&mut value, time, current, enabled) {
            return Ok(());
        }
        self.replace_paint_drawing(
            target,
            value,
            crate::model::EditKind::Structural,
            DRAWING_KEYFRAME_COMMIT,
        )
    }

    pub fn set_paint_drawing_expression_enabled(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        self.ensure_timeline(target, DRAWING_PATH, timeline_id)?;
        self.set_timeline_mode_with_commit(
            target,
            DRAWING_PATH,
            crate::TimelineModeChange {
                keyframes: false,
                enabled,
                current: serde_json::Value::Null,
                default_expression: crate::timeline_value::SCALAR_EXPRESSION_DEFAULT,
            },
            InspectorCommit::Immediate(DRAWING_EXPRESSION_COMMIT),
        )
    }

    pub fn set_paint_drawing_expression_source(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        source: &str,
    ) -> Result<(), String> {
        self.ensure_timeline(target, DRAWING_PATH, timeline_id)?;
        self.set_expression_source_with_commit(
            target,
            DRAWING_PATH,
            source,
            InspectorCommit::Coalesced(DRAWING_EXPRESSION_COMMIT),
        )
    }

    pub fn paint_drawing_graph(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
    ) -> Result<Option<ScalarGraph>, String> {
        let (value, runtime) = self.paint_drawing_timeline(target, timeline_id)?;
        Ok(drawing_graph(&value, runtime))
    }

    pub fn paint_drawing_expression_output(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
    ) -> Result<crate::InspectorExpressionOutput<String>, String> {
        self.ensure_timeline(target, DRAWING_PATH, timeline_id)?;
        let outcome = self.video_expression_output::<PaintDrawing>(target, DRAWING_PATH)?;
        Ok(crate::InspectorExpressionOutput {
            value: format!(
                "{} strokes, {} fills",
                outcome.value.strokes.len(),
                outcome.value.fills.len()
            ),
            error: outcome.error,
        })
    }

    pub fn move_paint_drawing_keyframes(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        moves: &[(Time, Time)],
    ) -> Result<Vec<Time>, String> {
        let project = self.project.borrow();
        let InspectorTarget::Item(address) = target else {
            return Err("paint drawing target is not an item".to_string());
        };
        let moves = moves
            .iter()
            .map(|&(old, time)| {
                crate::keyframe_model::canonical_keyframe_time(&project, Some(address), time)
                    .map(|time| (old, time))
                    .ok_or_else(|| "paint drawing keyframe time is unavailable".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        drop(project);
        let (mut value, _) = self.paint_drawing_timeline(target, timeline_id)?;
        if !crate::keyframe_model::move_discrete_keyframes(&mut value, &moves) {
            return Err("paint drawing keyframes are no longer available".to_string());
        }
        self.replace_paint_drawing(
            target,
            value,
            crate::model::EditKind::Live,
            "move-paint-drawing-keyframe",
        )?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_paint_drawing_keyframe(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        time: Time,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.paint_drawing_timeline(target, timeline_id)?;
        if !crate::keyframe_model::delete_discrete_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.replace_paint_drawing(
            target,
            value,
            crate::model::EditKind::Structural,
            "delete-paint-drawing-keyframe",
        )
    }

    pub fn add_paint_drawing_keyframe(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        time: Time,
    ) -> Result<(), String> {
        let project = self.project.borrow();
        let InspectorTarget::Item(address) = target else {
            return Err("paint drawing target is not an item".to_string());
        };
        let time = crate::keyframe_model::canonical_keyframe_time(&project, Some(address), time)
            .ok_or_else(|| "paint drawing keyframe time is unavailable".to_string())?;
        drop(project);
        let (mut value, runtime) = self.paint_drawing_timeline(target, timeline_id)?;
        let TimelineBase::Keyframes(keyframes) = &mut value.base else {
            return Ok(());
        };
        if let Some(keyframe) = keyframes.iter_mut().find(|keyframe| {
            crate::keyframe_model::same_frame(keyframe.time, time, runtime.frame_step)
        }) {
            if keyframe.time == time {
                return Ok(());
            }
            keyframe.time = time;
        } else {
            keyframes.push(PaintDrawing::keyframe(time, PaintDrawing::default()));
        }
        keyframes.sort_by_key(|keyframe| keyframe.time);
        self.replace_paint_drawing(
            target,
            value,
            crate::model::EditKind::Structural,
            "add-paint-drawing-keyframe",
        )
    }

    pub fn copy_paint_drawing_keyframes(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        times: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.paint_drawing_timeline(target, timeline_id)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&value, times) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let InspectorTarget::Item(address) = target else {
            return Err("paint drawing target is not an item".to_string());
        };
        if !crate::keyframe_model::normalize_clipboard_times(
            &project,
            Some(address),
            &mut clipboard,
        ) {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        }
        let count = clipboard.len();
        self.keyframe_clipboard.replace(Some(clipboard));
        Ok(count)
    }

    pub fn paste_paint_drawing_keyframes(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let InspectorTarget::Item(address) = target else {
            return Err("paint drawing target is not an item".to_string());
        };
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time)
                .ok_or_else(|| {
                    "paint drawing keyframes cannot be pasted at this time".to_string()
                })?;
        drop(project);
        let (mut value, _) = self.paint_drawing_timeline(target, timeline_id)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        if let TimelineBase::Keyframes(keyframes) = &mut value.base {
            for keyframe in keyframes
                .iter_mut()
                .filter(|keyframe| pasted.contains(&keyframe.time))
            {
                regenerate_drawing_edit_ids(&mut keyframe.value);
            }
        }
        let count = pasted.len();
        self.replace_paint_drawing(
            target,
            value,
            crate::model::EditKind::Structural,
            "paste-paint-drawing-keyframes",
        )?;
        Ok(count)
    }

    pub fn set_paint_drawing_interpolation(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation: usize,
    ) -> Result<(), String> {
        let interpolation = crate::keyframe_model::interpolation(interpolation)?;
        let (mut value, _) = self.paint_drawing_timeline(target, timeline_id)?;
        let TimelineBase::Keyframes(keyframes) = &mut value.base else {
            return Ok(());
        };
        let Some(keyframe) = keyframes
            .iter_mut()
            .find(|keyframe| keyframe.id == owner_id)
        else {
            return Ok(());
        };
        if keyframe.interpolation_to_next == interpolation {
            return Ok(());
        }
        keyframe.interpolation_to_next = interpolation;
        self.replace_paint_drawing(
            target,
            value,
            crate::model::EditKind::Structural,
            "paint-drawing-easing",
        )
    }

    fn paint_drawing_timeline(
        &self,
        target: &InspectorTarget,
        timeline_id: uuid::Uuid,
    ) -> Result<(TimelineValue<PaintDrawing>, InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let value: TimelineValue<PaintDrawing> = serde_json::from_value(
            snapshot
                .value
                .pointer(DRAWING_PATH)
                .cloned()
                .ok_or_else(|| "paint drawing is no longer available".to_string())?,
        )
        .map_err(|error| format!("invalid paint drawing: {error}"))?;
        if value.id != timeline_id {
            return Err("paint drawing is no longer available".to_string());
        }
        Ok((value, snapshot.runtime))
    }

    fn replace_paint_drawing(
        &self,
        target: &InspectorTarget,
        value: TimelineValue<PaintDrawing>,
        kind: crate::model::EditKind,
        commit: &'static str,
    ) -> Result<(), String> {
        self.replace_value_with_commit(
            target,
            kind,
            DRAWING_PATH,
            serde_json::to_value(value).expect("paint drawing must serialize"),
            Some(shrimply_state::player_state::ProjectChange {
                video: true,
                inspector: kind != crate::model::EditKind::Live,
                ..Default::default()
            }),
            InspectorCommit::Immediate(commit),
        )
    }

    pub fn add_paint_palette_color(&self, target: &InspectorTarget) -> Result<(), String> {
        self.update_paint(target, "add-paint-palette-color", |paint| {
            paint.palette.push(PaintPaletteEntry {
                color: TimelineValue::new_const(Color::<u8>::WHITE),
                texture: None,
            });
            true
        })
    }

    pub fn remove_paint_palette_color(
        &self,
        target: &InspectorTarget,
        color_id: uuid::Uuid,
    ) -> Result<(), String> {
        self.update_paint(target, "remove-paint-palette-color", move |paint| {
            let Some(index) = paint
                .palette
                .iter()
                .position(|entry| entry.color.id == color_id)
                .filter(|_| paint.palette.len() > 1)
            else {
                return false;
            };
            paint.palette.remove(index);
            let replacement = index.min(paint.palette.len() - 1);
            visit_drawings_mut(&mut paint.drawing, |drawing| {
                for stroke in &mut drawing.strokes {
                    stroke.color_index = match stroke.color_index.cmp(&index) {
                        std::cmp::Ordering::Less => stroke.color_index,
                        std::cmp::Ordering::Equal => replacement,
                        std::cmp::Ordering::Greater => stroke.color_index - 1,
                    };
                }
                for fill in &mut drawing.fills {
                    fill.color_index = match fill.color_index.cmp(&index) {
                        std::cmp::Ordering::Less => fill.color_index,
                        std::cmp::Ordering::Equal => replacement,
                        std::cmp::Ordering::Greater => fill.color_index - 1,
                    };
                }
            });
            true
        })
    }

    pub fn set_paint_texture(
        &self,
        target: &InspectorTarget,
        color_id: uuid::Uuid,
        path: &Path,
    ) -> Result<(), String> {
        let path = path.to_owned();
        self.update_paint(target, "paint-texture-path", move |paint| {
            let Some(texture) = paint
                .palette
                .iter_mut()
                .find(|entry| entry.color.id == color_id)
                .map(|entry| &mut entry.texture)
            else {
                return false;
            };
            match texture {
                Some(texture) if texture.image_path.path() == path => false,
                Some(texture) => {
                    texture.image_path = path.into();
                    true
                }
                texture @ None => {
                    *texture = Some(PaintTextureOptions::new(path));
                    true
                }
            }
        })
    }

    pub fn clear_paint_texture(
        &self,
        target: &InspectorTarget,
        color_id: uuid::Uuid,
    ) -> Result<(), String> {
        self.update_paint(target, "paint-texture-clear", move |paint| {
            let Some(texture) = paint
                .palette
                .iter_mut()
                .find(|entry| entry.color.id == color_id)
                .map(|entry| &mut entry.texture)
            else {
                return false;
            };
            texture.take().is_some()
        })
    }

    fn update_paint(
        &self,
        target: &InspectorTarget,
        commit: &'static str,
        update: impl FnOnce(&mut PaintItem) -> bool,
    ) -> Result<(), String> {
        let InspectorTarget::Item(address) = target else {
            return Err("paint inspector target is not an item".to_string());
        };
        if self.target() != *target {
            return Err("paint inspector target is no longer active".to_string());
        }
        let mut project = self.project.borrow_mut();
        let paint = selected_paint_mut(&mut project, address)?;
        if !update(paint) {
            return Ok(());
        }
        bump_revision(paint)?;
        shrimply_project::project::commit_edit(&project, commit);
        drop(project);
        shrimply_state::player_state::refresh_project(
            &self.player_state,
            shrimply_state::player_state::ProjectChange {
                video: true,
                inspector: true,
                ..Default::default()
            },
        );
        Ok(())
    }
}

fn regenerate_drawing_edit_ids(drawing: &mut PaintDrawing) {
    for stroke in &mut drawing.strokes {
        stroke.id = uuid::Uuid::new_v4();
    }
    for fill in &mut drawing.fills {
        fill.id = uuid::Uuid::new_v4();
    }
}

pub(crate) fn bump_serialized_revision(value: &mut serde_json::Value) -> Result<(), String> {
    let Some(revision) = value.pointer_mut("/content/revision") else {
        return Ok(());
    };
    let current = revision
        .as_u64()
        .ok_or_else(|| "paint revision is invalid".to_string())?;
    *revision = serde_json::Value::from(
        current
            .checked_add(1)
            .ok_or_else(|| "paint revision overflow".to_string())?,
    );
    Ok(())
}

fn selected_paint_mut<'a>(
    project: &'a mut Project,
    address: &shrimply_project::project::ItemAddress,
) -> Result<&'a mut PaintItem, String> {
    let item = project
        .video_item_mut(address)
        .ok_or_else(|| "paint item is no longer available".to_string())?;
    let VideoItemContent::Paint(paint) = &mut item.content else {
        return Err("video item is not paint".to_string());
    };
    Ok(paint)
}

fn bump_revision(paint: &mut PaintItem) -> Result<(), String> {
    paint.revision = paint
        .revision
        .checked_add(1)
        .ok_or_else(|| "paint revision overflow".to_string())?;
    Ok(())
}

fn visit_drawings_mut(
    value: &mut TimelineValue<PaintDrawing>,
    mut visit: impl FnMut(&mut PaintDrawing),
) {
    match &mut value.base {
        TimelineBase::Const(drawing) => visit(drawing),
        TimelineBase::Keyframes(keyframes) => {
            for keyframe in keyframes {
                visit(&mut keyframe.value)
            }
        }
    }
}

fn json(value: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).expect("paint inspector value must serialize")
}
