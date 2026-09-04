use shrimply_core::Color;
use shrimply_core::timeline_value::{
    CurveEditPolicy, CurveKeyframeInsert, Interpolation, TimelineBase, TimelineValue,
    TimelineValueType, edit_curve_value, insert_curve_keyframe,
};
use shrimply_project::project::Time;
use uuid::Uuid;

use crate::keyframe_graph::{KeyframeGraph, SpeedSegment};
use crate::{GraphPoint, GraphSegment, InspectorRuntime, ScalarGraph};

pub type ColorTimeline = TimelineValue<Color<u8>>;

pub fn video_value(
    item: &shrimply_project::project::VideoItem,
    timeline_id: Uuid,
) -> Option<&ColorTimeline> {
    use shrimply_project::project::VideoItemContent;

    let content = match &item.content {
        VideoItemContent::Background(background) => background.generator.color(timeline_id),
        VideoItemContent::Paint(paint) => paint
            .palette
            .iter()
            .map(|entry| &entry.color)
            .find(|value| value.id == timeline_id),
        VideoItemContent::Obj(scene) => crate::scene_3d::color(scene, timeline_id),
        VideoItemContent::Shape(shape) => [&shape.fill, &shape.outline_color, &shape.shadow_color]
            .into_iter()
            .find(|value| value.id == timeline_id),
        VideoItemContent::Text(text) => [
            &text.color,
            &text.background_color,
            &text.outline_color,
            &text.shadow_color,
        ]
        .into_iter()
        .find(|value| value.id == timeline_id),
        _ => None,
    };
    content.or_else(|| crate::visual_modifiers::visual_modifier_color_by_id(item, timeline_id))
}

pub fn validate_timeline(value: &ColorTimeline, timeline_id: Uuid) -> Result<(), String> {
    (value.id == timeline_id)
        .then_some(())
        .ok_or_else(|| "color timeline changed".to_string())
}

pub fn value_at(value: &ColorTimeline, time: Time) -> Color<u8> {
    value.value_at(time)
}

pub fn set_value(
    value: &mut ColorTimeline,
    evaluation_time: Time,
    keyframe_time: Time,
    next: Color<u8>,
) -> bool {
    let current = value_at(value, evaluation_time);
    edit_curve_value(
        value,
        keyframe_time,
        next,
        PartialEq::eq,
        CurveEditPolicy {
            unchanged_keyframe_is_noop: true,
            insert: if current == next {
                CurveKeyframeInsert::Skip
            } else {
                CurveKeyframeInsert::Default
            },
        },
    )
}

pub fn set_keyframes_enabled(
    value: &mut ColorTimeline,
    evaluation_time: Time,
    keyframe_time: Time,
    enabled: bool,
) -> bool {
    let current = value_at(value, evaluation_time);
    crate::keyframe_model::set_keyframes_enabled(value, keyframe_time, current, enabled)
}

pub fn set_expression_enabled(value: &mut ColorTimeline, enabled: bool) -> bool {
    crate::keyframe_model::set_expression_enabled(
        value,
        enabled,
        crate::timeline_value::SCALAR_EXPRESSION_DEFAULT,
    )
}

pub fn set_expression_source(value: &mut ColorTimeline, source: String) -> bool {
    let Some(expression) = value.expression.as_mut() else {
        return false;
    };
    if expression.source == source {
        return false;
    }
    expression.source = source;
    true
}

pub fn add_keyframe(value: &mut ColorTimeline, time: Time) -> bool {
    let current = value_at(value, time);
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    if let Some(keyframe) = keyframes
        .iter_mut()
        .find(|keyframe| keyframe.time.approx_eq(time))
    {
        if keyframe.time == time {
            return false;
        }
        keyframe.time = time;
        keyframes.sort_by_key(|keyframe| keyframe.time);
    } else {
        insert_curve_keyframe(
            keyframes,
            Color::<u8>::keyframe(time, current),
            CurveKeyframeInsert::InheritPreviousInterpolation,
        );
    }
    true
}

pub fn delete_keyframe(value: &mut ColorTimeline, time: Time) -> bool {
    crate::keyframe_model::delete_discrete_keyframe(value, time, Time::ZERO)
}

pub fn move_keyframes(value: &mut ColorTimeline, moves: &[(Time, Time)]) -> bool {
    crate::keyframe_model::move_discrete_keyframes(value, moves)
}

pub fn copy_keyframes(
    value: &ColorTimeline,
    selected: &[Time],
) -> Option<crate::keyframe_model::KeyframeClipboard> {
    crate::keyframe_model::copy_keyframes(value, selected)
}

pub fn paste_keyframes(
    value: &mut ColorTimeline,
    clipboard: &crate::keyframe_model::KeyframeClipboard,
    times: &[Time],
) -> Option<Vec<Time>> {
    crate::keyframe_model::paste_keyframes(value, clipboard, times)
}

pub fn set_interpolation(
    value: &mut ColorTimeline,
    owner_id: Uuid,
    interpolation: Interpolation,
) -> Result<bool, String> {
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return Err("color keyframes are no longer enabled".to_string());
    };
    let keyframe = keyframes
        .iter_mut()
        .find(|keyframe| keyframe.id == owner_id)
        .ok_or_else(|| "color keyframe owner is no longer available".to_string())?;
    if keyframe.interpolation_to_next == interpolation {
        return Ok(false);
    }
    keyframe.interpolation_to_next = interpolation;
    Ok(true)
}

pub fn keyframe_graph(value: &ColorTimeline) -> KeyframeGraph {
    let TimelineBase::Keyframes(keyframes) = &value.base else {
        return KeyframeGraph::Speed {
            segments: Vec::new(),
            keys: Vec::new(),
            static_value: 0.0,
        };
    };
    KeyframeGraph::Speed {
        segments: keyframes
            .windows(2)
            .filter_map(|pair| {
                let seconds = pair[1].time.signed_sub(pair[0].time).as_secs_f64();
                (seconds > f64::EPSILON).then(|| SpeedSegment {
                    owner_id: pair[0].id,
                    start: pair[0].time,
                    end: pair[1].time,
                    value: f64::from(pair[0].value.oklaba_distance(pair[1].value)) / seconds,
                    interpolation: pair[0].interpolation_to_next,
                })
            })
            .collect(),
        keys: keyframes.iter().map(|keyframe| keyframe.time).collect(),
        static_value: 0.0,
    }
}

pub fn speed_graph(value: &ColorTimeline, runtime: InspectorRuntime) -> Option<ScalarGraph> {
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
                (seconds > f64::EPSILON).then(|| {
                    let speed = f64::from(pair[0].value.oklaba_distance(pair[1].value)) / seconds;
                    GraphSegment {
                        owner_id: pair[0].id,
                        start: pair[0].time,
                        end: pair[1].time,
                        start_value: speed,
                        end_value: speed,
                        interpolation: Interpolation::KEYFRAME
                            .iter()
                            .position(|candidate| *candidate == pair[0].interpolation_to_next)
                            .expect("color interpolation must be available"),
                    }
                })
            })
            .collect(),
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}
