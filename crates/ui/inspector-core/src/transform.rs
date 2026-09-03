use glam::{Vec2, Vec3};
use shrimply_core::timeline_value::{
    CurveEditPolicy, CurveKeyframeInsert, Interpolation, TimelineBase, TimelineValue,
    edit_curve_value, set_keyframes_enabled,
};
use shrimply_project::project::{
    ItemAddress, Project, ResolvedTransform, Time, VideoItem, VideoItemContent,
};
use shrimply_video_modifiers::{ModifierEffect, RasterModifierEffect, VectorModifierEffect};

use crate::{
    ControlKind, GraphPoint, GraphSegment, InspectorControl, InspectorController,
    InspectorExpressionOutput, InspectorRuntime, InspectorSection, InspectorTarget, LayeredState,
    NumberSpec, ScalarGraph, VideoCard,
};

const POSITION_PATH: &str = "/transform/position";
const ANCHOR_PATH: &str = "/transform/anchor";
const SCALE_PATH: &str = "/transform/scale";
const SHEAR_PATH: &str = "/transform/shear";
const ROTATION_PATH: &str = "/transform/rotation_degrees";

#[derive(Clone, Debug, PartialEq)]
pub struct TransformLivePresentation {
    pub resolved: ResolvedTransform,
    pub vectors: Vec<(String, Vec2)>,
    pub numbers: Vec<(String, f64)>,
    pub graphs: Vec<(String, ScalarGraph)>,
}

impl TransformLivePresentation {
    pub fn vector(&self, path: &str) -> Option<Vec2> {
        self.vectors
            .iter()
            .find_map(|(candidate, value)| (candidate == path).then_some(*value))
    }

    pub fn graph(&self, path: &str) -> Option<&ScalarGraph> {
        self.graphs
            .iter()
            .find_map(|(candidate, graph)| (candidate == path).then_some(graph))
    }

    pub fn number(&self, path: &str) -> Option<f64> {
        self.numbers
            .iter()
            .find_map(|(candidate, value)| (candidate == path).then_some(*value))
    }
}

pub(crate) fn card(
    project: &Project,
    address: &ItemAddress,
    item: &VideoItem,
    runtime: InspectorRuntime,
) -> Option<VideoCard> {
    if matches!(
        item.content,
        VideoItemContent::Obj(_)
            | VideoItemContent::Gaussian(_)
            | VideoItemContent::Manim(_)
            | VideoItemContent::Background(_)
    ) {
        return None;
    }
    let display = project
        .timeline_time_to_sequence(&address.track(), runtime.position)
        .map(|position| shrimply_evaluation::resolve_item_base_transform(project, item, position))
        .unwrap_or_else(|| item.transform.fallback());
    let mut section = InspectorSection::default();
    section.add(vector_control(
        POSITION_PATH,
        "Position",
        &item.transform.position,
        display.position,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(vector_control(
        ANCHOR_PATH,
        "Anchor",
        &item.transform.anchor,
        display.anchor,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(vector_control(
        SCALE_PATH,
        "Scale",
        &item.transform.scale,
        display.scale,
        runtime,
        NumberSpec {
            minimum: 0.0,
            drag_step: 0.01,
            digits: 2,
            unit: "x",
            ..NumberSpec::default()
        },
        true,
    ));
    section.add(vector_control(
        SHEAR_PATH,
        "Shear",
        &item.transform.shear,
        display.shear,
        runtime,
        NumberSpec {
            drag_step: 0.01,
            digits: 2,
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(scalar_control(
        &item.transform.rotation_degrees,
        display.rotation_degrees,
        runtime,
    ));
    Some(
        VideoCard {
            key: "transform",
            title: "Transform",
            section,
            reset: None,
        }
        .reset(
            "/transform",
            serde_json::to_value(item.natural_transform(project.canvas_size))
                .expect("natural video transform must serialize"),
            "reset-transform",
        ),
    )
}

fn vector_control(
    path: &'static str,
    label: &'static str,
    timeline: &TimelineValue<Vec2>,
    display: Vec2,
    runtime: InspectorRuntime,
    number: NumberSpec,
    lock: bool,
) -> InspectorControl {
    let control = InspectorControl::new(ControlKind::LayeredVector2, path, label)
        .components(vec![display.x.to_string(), display.y.to_string()])
        .number(number)
        .width_characters(7)
        .prefixes(["X", "Y"])
        .layered(path, LayeredState::from(timeline))
        .graph(vector_speed_graph(timeline, runtime))
        .live_commit("video-transform");
    if lock { control.lock() } else { control }
}

fn scalar_control(
    timeline: &TimelineValue<f32>,
    display: f32,
    runtime: InspectorRuntime,
) -> InspectorControl {
    InspectorControl::new(ControlKind::LayeredNumber, ROTATION_PATH, "Rotation")
        .value(display.to_string())
        .number(NumberSpec {
            drag_step: 0.1,
            digits: 1,
            unit: "°",
            ..NumberSpec::default()
        })
        .width_characters(9)
        .rotating_icon("rotation.svg", 0.0)
        .layered(ROTATION_PATH, LayeredState::from(timeline))
        .graph(scalar_graph(timeline, display, runtime))
        .live_commit("video-transform")
}

fn graph_shell(
    runtime: InspectorRuntime,
    points: Vec<GraphPoint>,
    segments: Vec<GraphSegment>,
) -> ScalarGraph {
    ScalarGraph {
        points,
        segments,
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    }
}

pub(crate) fn vector_speed_graph(
    timeline: &TimelineValue<Vec2>,
    runtime: InspectorRuntime,
) -> Option<ScalarGraph> {
    let TimelineBase::Keyframes(keyframes) = &timeline.base else {
        return None;
    };
    let points = keyframes
        .iter()
        .map(|keyframe| GraphPoint {
            time: keyframe.time,
            value: 0.0,
        })
        .collect();
    let segments = keyframes
        .windows(2)
        .filter_map(|pair| {
            let seconds = pair[1].time.signed_sub(pair[0].time).as_secs_f64();
            (seconds > f64::EPSILON).then(|| {
                let speed = f64::from((pair[1].value - pair[0].value).length()) / seconds;
                GraphSegment {
                    owner_id: pair[0].id,
                    start: pair[0].time,
                    end: pair[1].time,
                    start_value: speed,
                    end_value: speed,
                    interpolation: interpolation_index(pair[0].interpolation_to_next),
                }
            })
        })
        .collect();
    Some(graph_shell(runtime, points, segments))
}

pub(crate) fn scalar_graph(
    timeline: &TimelineValue<f32>,
    display: f32,
    runtime: InspectorRuntime,
) -> Option<ScalarGraph> {
    let shrimply_keyframe_graph_ui::KeyframeGraph::RawValue {
        points, segments, ..
    } = crate::keyframe_model::scalar_graph(timeline, f64::from(display), f64::from)
    else {
        unreachable!("scalar transform must produce a raw graph")
    };
    matches!(timeline.base, TimelineBase::Keyframes(_)).then(|| {
        graph_shell(
            runtime,
            points
                .into_iter()
                .map(|point| GraphPoint {
                    time: point.time,
                    value: point.value,
                })
                .collect(),
            segments
                .into_iter()
                .map(|segment| GraphSegment {
                    owner_id: segment.owner_id,
                    start: segment.start,
                    end: segment.end,
                    start_value: segment.start_value,
                    end_value: segment.end_value,
                    interpolation: interpolation_index(segment.interpolation),
                })
                .collect(),
        )
    })
}

fn interpolation_index(interpolation: Interpolation) -> usize {
    Interpolation::KEYFRAME
        .iter()
        .position(|candidate| *candidate == interpolation)
        .expect("transform interpolation must be available")
}

impl InspectorController {
    pub fn resolved_transform(&self, target: &InspectorTarget) -> Option<ResolvedTransform> {
        let position = shrimply_state::player_state::snapshot(&self.player_state).position;
        let project = self.project.borrow();
        let (item, position) = transform_item_at(&project, target, position)?;
        Some(shrimply_evaluation::resolve_item_base_transform(
            &project, item, position,
        ))
    }

    pub fn transform_live_presentation(
        &self,
        target: &InspectorTarget,
    ) -> Option<TransformLivePresentation> {
        let project = self.project.borrow();
        let runtime = crate::model::target_runtime(&project, &self.player_state, target);
        let (item, position) = transform_item_at(&project, target, runtime.position)?;
        let resolved = shrimply_evaluation::resolve_item_base_transform(&project, item, position);
        let mut vectors = vec![
            (POSITION_PATH.to_string(), resolved.position),
            (ANCHOR_PATH.to_string(), resolved.anchor),
            (SCALE_PATH.to_string(), resolved.scale),
            (SHEAR_PATH.to_string(), resolved.shear),
        ];
        let mut numbers = vec![(
            ROTATION_PATH.to_string(),
            f64::from(resolved.rotation_degrees),
        )];
        let mut graphs = [
            (POSITION_PATH, &item.transform.position),
            (ANCHOR_PATH, &item.transform.anchor),
            (SCALE_PATH, &item.transform.scale),
            (SHEAR_PATH, &item.transform.shear),
        ]
        .into_iter()
        .filter_map(|(path, value)| {
            vector_speed_graph(value, runtime).map(|graph| (path.to_string(), graph))
        })
        .collect::<Vec<_>>();
        if let Some(graph) = scalar_graph(
            &item.transform.rotation_degrees,
            resolved.rotation_degrees,
            runtime,
        ) {
            graphs.push((ROTATION_PATH.to_string(), graph));
        }
        let player = shrimply_state::player_state::snapshot(&self.player_state);
        let audio =
            self.audio_sampler
                .borrow_mut()
                .sample(&project, player.position, player.revision);
        let evaluation = shrimply_evaluation::VisualEvaluation::for_item_with_audio(
            &project, item, position, &audio,
        );
        let mut expression_cache = self.expression_cache.borrow_mut();
        for (index, modifier) in item.modifiers.iter().enumerate() {
            let transform = match &modifier.effect {
                ModifierEffect::Vector(effect) => match &**effect {
                    VectorModifierEffect::Transform(transform) => Some(transform.as_ref()),
                    _ => None,
                },
                ModifierEffect::Raster(effect) => match &**effect {
                    RasterModifierEffect::Transform(transform) => Some(transform.as_ref()),
                    _ => None,
                },
                ModifierEffect::Scene3d(_)
                | ModifierEffect::Vectorize(_)
                | ModifierEffect::Rasterize(_) => None,
            };
            let Some(transform) = transform else {
                continue;
            };
            let base = format!("/modifiers/{index}/effect/effect/config/transform");
            for (field, timeline) in [
                ("position", transform.position()),
                ("anchor", transform.anchor()),
                ("scale", transform.scale()),
                ("shear", transform.shear()),
            ] {
                let path = format!("{base}/{field}");
                vectors.push((
                    path.clone(),
                    shrimply_evaluation::resolve_with_error(
                        timeline,
                        &evaluation,
                        &mut expression_cache,
                    )
                    .value,
                ));
                if let Some(graph) = vector_speed_graph(timeline, runtime) {
                    graphs.push((path, graph));
                }
            }
            let rotation = shrimply_evaluation::resolve_with_error(
                transform.rotation_degrees(),
                &evaluation,
                &mut expression_cache,
            )
            .value;
            numbers.push((format!("{base}/rotation_degrees"), f64::from(rotation)));
            if let Some(graph) = scalar_graph(transform.rotation_degrees(), rotation, runtime) {
                graphs.push((format!("{base}/rotation_degrees"), graph));
            }
        }
        Some(TransformLivePresentation {
            resolved,
            vectors,
            numbers,
            graphs,
        })
    }

    pub fn set_vector2_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        first: f64,
        second: f64,
    ) -> Result<(), String> {
        if !first.is_finite() || !second.is_finite() {
            return Err("transform vector must be finite".to_string());
        }
        let (mut timeline, runtime) = self.vector2_timeline(target, path)?;
        let mut next = Vec2::new(first as f32, second as f32);
        if path == SCALE_PATH {
            next = next.max(Vec2::ZERO);
        }
        let time = if matches!(timeline.base, TimelineBase::Keyframes(_)) {
            runtime
                .keyframe_playhead
                .ok_or_else(|| "transform keyframe time is no longer available".to_string())?
        } else {
            Time::ZERO
        };
        if !edit_curve_value(
            &mut timeline,
            time,
            next,
            |current, next| current.abs_diff_eq(*next, 0.000_001),
            CurveEditPolicy {
                unchanged_keyframe_is_noop: true,
                insert: CurveKeyframeInsert::InheritPreviousInterpolation,
            },
        ) {
            return Ok(());
        }
        self.set_live_keyframe_graph_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )
    }

    pub fn set_vector3_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        first: f64,
        second: f64,
        third: f64,
    ) -> Result<(), String> {
        if !first.is_finite() || !second.is_finite() || !third.is_finite() {
            return Err("modifier vector must be finite".to_string());
        }
        let (mut timeline, runtime) = self.vector3_timeline(target, path)?;
        runtime
            .local_time
            .ok_or_else(|| "modifier evaluation time is no longer available".to_string())?;
        let time = if matches!(timeline.base, TimelineBase::Keyframes(_)) {
            runtime
                .keyframe_playhead
                .ok_or_else(|| "modifier keyframe time is no longer available".to_string())?
        } else {
            Time::ZERO
        };
        if !edit_curve_value(
            &mut timeline,
            time,
            Vec3::new(first as f32, second as f32, third as f32),
            |current, next| current.abs_diff_eq(*next, 0.000_001),
            CurveEditPolicy {
                unchanged_keyframe_is_noop: true,
                insert: CurveKeyframeInsert::InheritPreviousInterpolation,
            },
        ) {
            return Ok(());
        }
        self.set_live_keyframe_graph_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )
    }

    pub fn set_vector2_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let (mut timeline, runtime) = self.vector2_timeline(target, path)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "transform keyframe time is no longer available".to_string())?;
        let mut current = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
        if path == SCALE_PATH {
            current = current.max(Vec2::ZERO);
        }
        if !set_keyframes_enabled(&mut timeline, time, current, enabled) {
            return Ok(());
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )
    }

    pub fn set_vector3_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let (mut timeline, runtime) = self.vector3_timeline(target, path)?;
        let evaluation_time = runtime
            .local_time
            .ok_or_else(|| "modifier evaluation time is no longer available".to_string())?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "modifier keyframe time is no longer available".to_string())?;
        let current = timeline.value_at(evaluation_time);
        if !set_keyframes_enabled(&mut timeline, time, current, enabled) {
            return Ok(());
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )
    }

    pub fn vector2_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: Option<uuid::Uuid>,
    ) -> Result<InspectorExpressionOutput<Vec2>, String> {
        if let Some(timeline_id) = timeline_id {
            return self.video_modifier_expression_output(
                target,
                path,
                timeline_id,
                crate::visual_modifiers::visual_modifier_vector2,
            );
        }
        self.video_expression_output(target, path)
    }

    pub fn vector3_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<InspectorExpressionOutput<Vec3>, String> {
        self.video_modifier_expression_output(
            target,
            path,
            timeline_id,
            crate::visual_modifiers::visual_modifier_vector3,
        )
    }

    pub fn move_vector2_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        moves: &[(Time, Time)],
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_vector_moves(target, moves)?;
        let (mut timeline, _) = self.vector2_timeline(target, path)?;
        if !crate::keyframe_model::move_discrete_keyframes(&mut timeline, &moves) {
            return Err("transform keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_vector2_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let (mut timeline, _) = self.vector2_timeline(target, path)?;
        let TimelineBase::Keyframes(keyframes) = &mut timeline.base else {
            return Ok(());
        };
        let Some(index) = keyframes
            .iter()
            .position(|keyframe| keyframe.time.approx_eq(time))
        else {
            return Ok(());
        };
        keyframes.remove(index);
        if keyframes.is_empty() {
            timeline.base = TimelineBase::Const(Vec2::ZERO);
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )
    }

    pub fn delete_transform_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        if path != ROTATION_PATH {
            return Err(format!("unknown transform scalar: {path}"));
        }
        let mut timeline = self.scalar_timeline(target, path)?;
        let TimelineBase::Keyframes(keyframes) = &mut timeline.base else {
            return Ok(());
        };
        let Some(index) = keyframes
            .iter()
            .position(|keyframe| keyframe.time.approx_eq(time))
        else {
            return Ok(());
        };
        keyframes.remove(index);
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform scalar must serialize"),
        )
    }

    pub fn add_vector2_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_vector_time(target, time)?;
        let (mut timeline, _) = self.vector2_timeline(target, path)?;
        let current = timeline.value_at(time);
        if !edit_curve_value(
            &mut timeline,
            time,
            current,
            |_, _| false,
            CurveEditPolicy {
                unchanged_keyframe_is_noop: false,
                insert: CurveKeyframeInsert::InheritPreviousInterpolation,
            },
        ) {
            return Ok(());
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )
    }

    pub fn copy_vector2_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (timeline, _) = self.vector2_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&timeline, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_address(target)?;
        let times = clipboard
            .times
            .iter()
            .map(|time| {
                project
                    .keyframe_timeline_time(address, *time)
                    .unwrap_or(*time)
                    .snapped(project.frame_step())
            })
            .collect::<Vec<_>>();
        let Some(origin) = times.first().copied() else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        clipboard.times = times
            .into_iter()
            .map(|time| Time {
                seconds: time.seconds - origin.seconds,
            })
            .collect();
        let count = clipboard.len();
        self.keyframe_clipboard.replace(Some(clipboard));
        Ok(count)
    }

    pub fn paste_vector2_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_address(target)?;
        let anchor = project
            .keyframe_timeline_time(address, time)
            .unwrap_or(time)
            .snapped(project.frame_step());
        let times = clipboard
            .times
            .iter()
            .filter_map(|offset| {
                project.keyframe_time(
                    address,
                    Time {
                        seconds: anchor.seconds + offset.seconds,
                    },
                )
            })
            .collect::<Vec<_>>();
        drop(project);
        if times.len() != clipboard.len() {
            return Err("transform keyframes cannot be pasted at this time".to_string());
        }
        let (mut timeline, _) = self.vector2_timeline(target, path)?;
        let Some(pasted) =
            crate::keyframe_model::paste_keyframes(&mut timeline, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )?;
        Ok(pasted.len())
    }

    pub fn set_vector2_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
    ) -> Result<(), String> {
        let interpolation = Interpolation::KEYFRAME
            .get(interpolation_index)
            .copied()
            .ok_or_else(|| "transform interpolation is invalid".to_string())?;
        let (mut timeline, _) = self.vector2_timeline(target, path)?;
        let TimelineBase::Keyframes(keyframes) = &mut timeline.base else {
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
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("transform vector must serialize"),
        )
    }

    pub fn move_vector3_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        moves: &[(Time, Time)],
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_vector_moves(target, moves)?;
        let (mut timeline, _) = self.vector3_timeline(target, path)?;
        if !crate::keyframe_model::move_discrete_keyframes(&mut timeline, &moves) {
            return Err("modifier keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_vector3_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let (mut timeline, _) = self.vector3_timeline(target, path)?;
        let TimelineBase::Keyframes(keyframes) = &mut timeline.base else {
            return Ok(());
        };
        let Some(index) = keyframes
            .iter()
            .position(|keyframe| keyframe.time.approx_eq(time))
        else {
            return Ok(());
        };
        let removed = keyframes.remove(index);
        if keyframes.is_empty() {
            timeline.base = TimelineBase::Const(removed.value);
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )
    }

    pub fn add_vector3_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_vector_time(target, time)?;
        let (mut timeline, _) = self.vector3_timeline(target, path)?;
        let current = timeline.value_at(time);
        if !edit_curve_value(
            &mut timeline,
            time,
            current,
            |_, _| false,
            CurveEditPolicy {
                unchanged_keyframe_is_noop: false,
                insert: CurveKeyframeInsert::InheritPreviousInterpolation,
            },
        ) {
            return Ok(());
        }
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )
    }

    pub fn copy_vector3_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (timeline, _) = self.vector3_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&timeline, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_address(target)?;
        let times = clipboard
            .times
            .iter()
            .map(|time| {
                project
                    .keyframe_timeline_time(address, *time)
                    .unwrap_or(*time)
                    .snapped(project.frame_step())
            })
            .collect::<Vec<_>>();
        let Some(origin) = times.first().copied() else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        clipboard.times = times
            .into_iter()
            .map(|time| Time {
                seconds: time.seconds - origin.seconds,
            })
            .collect();
        let count = clipboard.len();
        self.keyframe_clipboard.replace(Some(clipboard));
        Ok(count)
    }

    pub fn paste_vector3_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_address(target)?;
        let anchor = project
            .keyframe_timeline_time(address, time)
            .unwrap_or(time)
            .snapped(project.frame_step());
        let times = clipboard
            .times
            .iter()
            .filter_map(|offset| {
                project.keyframe_time(
                    address,
                    Time {
                        seconds: anchor.seconds + offset.seconds,
                    },
                )
            })
            .collect::<Vec<_>>();
        drop(project);
        if times.len() != clipboard.len() {
            return Err("modifier keyframes cannot be pasted at this time".to_string());
        }
        let (mut timeline, _) = self.vector3_timeline(target, path)?;
        let Some(pasted) =
            crate::keyframe_model::paste_keyframes(&mut timeline, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )?;
        Ok(pasted.len())
    }

    pub fn set_vector3_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
    ) -> Result<(), String> {
        let interpolation = Interpolation::KEYFRAME
            .get(interpolation_index)
            .copied()
            .ok_or_else(|| "modifier interpolation is invalid".to_string())?;
        let (mut timeline, _) = self.vector3_timeline(target, path)?;
        let TimelineBase::Keyframes(keyframes) = &mut timeline.base else {
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
        self.set_value(
            target,
            path,
            serde_json::to_value(timeline).expect("modifier vector must serialize"),
        )
    }

    fn vector2_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<(TimelineValue<Vec2>, InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let timeline = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("vector value is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid vector value: {error}"))?;
        Ok((timeline, snapshot.runtime))
    }

    fn vector3_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<(TimelineValue<Vec3>, InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let timeline = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("vector value is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid vector value: {error}"))?;
        Ok((timeline, snapshot.runtime))
    }

    fn canonical_vector_time(&self, target: &InspectorTarget, time: Time) -> Result<Time, String> {
        let project = self.project.borrow();
        let address = video_address(target)?;
        project
            .keyframe_timeline_time(address, time)
            .and_then(|timeline_time| project.keyframe_time(address, timeline_time))
            .ok_or_else(|| "transform keyframe time is no longer available".to_string())
    }

    fn canonical_vector_moves(
        &self,
        target: &InspectorTarget,
        moves: &[(Time, Time)],
    ) -> Result<Vec<(Time, Time)>, String> {
        moves
            .iter()
            .map(|&(old_time, time)| Ok((old_time, self.canonical_vector_time(target, time)?)))
            .collect()
    }
}

fn video_address(target: &InspectorTarget) -> Result<&ItemAddress, String> {
    let InspectorTarget::Item(address @ ItemAddress::Video { .. }) = target else {
        return Err("transform target is not a video item".to_string());
    };
    Ok(address)
}

fn transform_item_at<'a>(
    project: &'a Project,
    target: &InspectorTarget,
    position: Time,
) -> Option<(&'a VideoItem, Time)> {
    let address = video_address(target).ok()?;
    let item = project.video_item(address)?;
    if matches!(
        item.content,
        VideoItemContent::Obj(_)
            | VideoItemContent::Gaussian(_)
            | VideoItemContent::Manim(_)
            | VideoItemContent::Background(_)
    ) {
        return None;
    }
    let position = project.timeline_time_to_sequence(&address.track(), position)?;
    Some((item, position))
}
