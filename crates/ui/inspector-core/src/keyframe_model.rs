use std::{any::Any, rc::Rc};

use shrimply_core::timeline_value::{
    CurveEditPolicy, CurveKeyframeInsert, DiscreteEditPolicy, Interpolation, TimelineBase,
    TimelineKeyframe, TimelineValue, TimelineValueType, edit_curve_value, edit_discrete_value,
};
use shrimply_keyframe_graph_ui::{KeyframeGraph, KeyframePoint, RawSegment};
use shrimply_project::project::Time;
use shrimply_state::player_state::ProjectChange;
use uuid::Uuid;

pub const KEYFRAME_CLIPBOARD_MARKER: &str = "shrimply keyframes";

#[derive(Clone)]
pub struct KeyframeClipboard {
    values: Rc<dyn Any>,
    pub times: Vec<Time>,
    len: usize,
}

#[derive(Clone)]
struct JsonDiscreteClipboard {
    timeline_type: String,
    keyframes: Vec<serde_json::Value>,
}

impl KeyframeClipboard {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub fn copy_keyframes<T: TimelineValueType>(
    value: &TimelineValue<T>,
    selected: &[Time],
) -> Option<KeyframeClipboard> {
    let TimelineBase::Keyframes(keyframes) = &value.base else {
        return None;
    };
    let copied: Vec<T::Keyframe> = keyframes
        .iter()
        .filter(|keyframe| selected.contains(&keyframe.time()))
        .cloned()
        .collect();
    (!copied.is_empty()).then(|| KeyframeClipboard {
        len: copied.len(),
        times: copied.iter().map(TimelineKeyframe::time).collect(),
        values: Rc::new(copied),
    })
}

pub fn paste_keyframes<T: TimelineValueType>(
    value: &mut TimelineValue<T>,
    clipboard: &KeyframeClipboard,
    times: &[Time],
) -> Option<Vec<Time>> {
    let source = clipboard.values.downcast_ref::<Vec<T::Keyframe>>()?;
    if source.len() != times.len() {
        return None;
    }
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return None;
    };
    let mut pasted = source.clone();
    for (keyframe, time) in pasted.iter_mut().zip(times) {
        *keyframe.id_mut() = Uuid::new_v4();
        *keyframe.time_mut() = *time;
    }
    let paste_start = *times.iter().min()?;
    let paste_end = *times.iter().max()?;
    if paste_start == paste_end {
        keyframes.retain(|keyframe| keyframe.time() != paste_start);
    } else {
        keyframes.retain(|keyframe| keyframe.time() < paste_start || keyframe.time() > paste_end);
    }
    let pasted_times = pasted.iter().map(TimelineKeyframe::time).collect();
    keyframes.extend(pasted);
    keyframes.sort_by_key(TimelineKeyframe::time);
    Some(pasted_times)
}

pub fn live_refresh(mut refresh: ProjectChange) -> ProjectChange {
    refresh.inspector = false;
    refresh.live_preview = true;
    refresh
}

pub fn same_frame(left: Time, right: Time, frame_step: Time) -> bool {
    if frame_step > Time::ZERO {
        left.snapped(frame_step) == right.snapped(frame_step)
    } else {
        left.approx_eq(right)
    }
}

pub fn scalar_graph(
    value: &TimelineValue<f32>,
    static_value: f64,
    display: impl Fn(f32) -> f64,
) -> KeyframeGraph {
    let TimelineBase::Keyframes(keyframes) = &value.base else {
        return KeyframeGraph::RawValue {
            points: Vec::new(),
            segments: Vec::new(),
            static_value,
        };
    };
    KeyframeGraph::RawValue {
        points: keyframes
            .iter()
            .map(|keyframe| KeyframePoint {
                time: keyframe.time,
                value: display(keyframe.value),
            })
            .collect(),
        segments: keyframes
            .windows(2)
            .map(|pair| RawSegment {
                owner_id: pair[0].id,
                start: pair[0].time,
                end: pair[1].time,
                start_value: display(pair[0].value),
                end_value: display(pair[1].value),
                interpolation: pair[0].interpolation_to_next,
            })
            .collect(),
        static_value,
    }
}

pub fn step_graph<T: TimelineValueType>(value: &TimelineValue<T>) -> KeyframeGraph {
    step_graph_with(value, |_| 0.5)
}

pub fn step_graph_with<T: TimelineValueType>(
    value: &TimelineValue<T>,
    display: impl Fn(&T) -> f64,
) -> KeyframeGraph {
    let TimelineBase::Keyframes(keyframes) = &value.base else {
        return KeyframeGraph::Step { points: Vec::new() };
    };
    KeyframeGraph::Step {
        points: keyframes
            .iter()
            .map(|keyframe| KeyframePoint {
                time: keyframe.time(),
                value: display(keyframe.value()),
            })
            .collect(),
    }
}

pub fn set_discrete_value<T: TimelineValueType>(
    value: &mut TimelineValue<T>,
    time: Time,
    next: T,
    frame_step: Time,
) -> bool {
    edit_discrete_value(
        value,
        time,
        next,
        |left, right| same_frame(left, right, frame_step),
        DiscreteEditPolicy {
            unchanged_is_noop: false,
            sort_updated_keyframe: false,
        },
    )
}

pub fn add_discrete_keyframe<T: TimelineValueType>(
    value: &mut TimelineValue<T>,
    time: Time,
    frame_step: Time,
) -> bool {
    if !matches!(value.base, TimelineBase::Keyframes(_)) {
        return false;
    }
    let current = value.value_at(time);
    set_discrete_value(value, time, current, frame_step)
}

pub fn delete_discrete_keyframe<T: TimelineValueType>(
    value: &mut TimelineValue<T>,
    time: Time,
    frame_step: Time,
) -> bool {
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    let Some(index) = keyframes
        .iter()
        .position(|keyframe| same_frame(keyframe.time(), time, frame_step))
    else {
        return false;
    };
    let removed = keyframes.remove(index).value().clone();
    if keyframes.is_empty() {
        value.base = TimelineBase::Const(removed);
    }
    true
}

pub fn move_discrete_keyframe<T: TimelineValueType>(
    value: &mut TimelineValue<T>,
    old_time: Time,
    time: Time,
) -> bool {
    move_discrete_keyframes(value, &[(old_time, time)])
}

pub fn move_discrete_keyframes<T: TimelineValueType>(
    value: &mut TimelineValue<T>,
    moves: &[(Time, Time)],
) -> bool {
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    let mut moved = Vec::with_capacity(moves.len());
    for (index, &(old_time, _)) in moves.iter().enumerate() {
        if moves[..index]
            .iter()
            .any(|(previous, _)| previous.approx_eq(old_time))
        {
            return false;
        }
        let Some(keyframe) = keyframes
            .iter()
            .find(|keyframe| keyframe.time().approx_eq(old_time))
            .cloned()
        else {
            return false;
        };
        moved.push(keyframe);
    }
    keyframes.retain(|keyframe| {
        !moves
            .iter()
            .any(|(old_time, _)| keyframe.time().approx_eq(*old_time))
    });
    let mut destinations = Vec::with_capacity(moved.len());
    for (mut keyframe, &(_, time)) in moved.into_iter().zip(moves) {
        keyframes.retain(|other| !other.time().approx_eq(time));
        destinations.retain(|other: &T::Keyframe| !other.time().approx_eq(time));
        *keyframe.time_mut() = time;
        destinations.push(keyframe);
    }
    keyframes.extend(destinations);
    keyframes.sort_by_key(TimelineKeyframe::time);
    true
}

pub fn set_json_discrete_value(
    value: &mut serde_json::Value,
    time: Time,
    next: serde_json::Value,
    frame_step: Time,
) -> Result<bool, String> {
    let keyframes = json_keyframes_mut(value)?;
    for keyframe in keyframes.iter() {
        json_keyframe_time(keyframe)?;
    }
    if let Some(keyframe) = keyframes.iter_mut().find(|keyframe| {
        same_frame(
            json_keyframe_time(keyframe)
                .expect("validated discrete keyframe time must remain valid"),
            time,
            frame_step,
        )
    }) {
        let keyframe = keyframe
            .as_object_mut()
            .ok_or_else(|| "discrete keyframe must be an object".to_string())?;
        if !keyframe.contains_key("value") {
            return Err("discrete keyframe value is missing".to_string());
        }
        keyframe.insert("value".to_string(), next);
        keyframe.insert(
            "time".to_string(),
            serde_json::to_value(time).expect("keyframe time must serialize"),
        );
        sort_json_keyframes(keyframes)?;
        return Ok(true);
    }
    keyframes.push(serde_json::json!({
        "id": Uuid::new_v4(),
        "time": time,
        "value": next,
    }));
    sort_json_keyframes(keyframes)?;
    Ok(true)
}

pub fn add_json_discrete_keyframe(
    value: &mut serde_json::Value,
    time: Time,
    frame_step: Time,
) -> Result<bool, String> {
    let current = {
        let keyframes = json_keyframes(value)?;
        for keyframe in keyframes {
            json_keyframe_time(keyframe)?;
        }
        keyframes
            .iter()
            .rev()
            .find(|keyframe| {
                json_keyframe_time(keyframe)
                    .expect("validated discrete keyframe time must remain valid")
                    <= time
            })
            .or_else(|| keyframes.first())
            .and_then(|keyframe| keyframe.get("value"))
            .cloned()
            .ok_or_else(|| "discrete timeline has no keyframe value".to_string())?
    };
    set_json_discrete_value(value, time, current, frame_step)
}

pub fn delete_json_discrete_keyframe(
    value: &mut serde_json::Value,
    time: Time,
    frame_step: Time,
) -> Result<bool, String> {
    let (removed, empty) = {
        let keyframes = json_keyframes_mut(value)?;
        for keyframe in keyframes.iter() {
            json_keyframe_time(keyframe)?;
        }
        let Some(index) = keyframes.iter().position(|keyframe| {
            same_frame(
                json_keyframe_time(keyframe)
                    .expect("validated discrete keyframe time must remain valid"),
                time,
                frame_step,
            )
        }) else {
            return Ok(false);
        };
        let removed = keyframes
            .remove(index)
            .get("value")
            .cloned()
            .ok_or_else(|| "discrete keyframe value is missing".to_string())?;
        (removed, keyframes.is_empty())
    };
    if empty {
        *value
            .get_mut("base")
            .ok_or_else(|| "discrete timeline base is missing".to_string())? =
            serde_json::json!({ "const": removed });
    }
    Ok(true)
}

pub fn move_json_discrete_keyframe(
    value: &mut serde_json::Value,
    old_time: Time,
    time: Time,
) -> Result<bool, String> {
    move_json_discrete_keyframes(value, &[(old_time, time)])
}

pub fn move_json_discrete_keyframes(
    value: &mut serde_json::Value,
    moves: &[(Time, Time)],
) -> Result<bool, String> {
    let keyframes = json_keyframes_mut(value)?;
    for keyframe in keyframes.iter() {
        json_keyframe_time(keyframe)?;
    }
    let mut moved = Vec::with_capacity(moves.len());
    for (index, &(old_time, _)) in moves.iter().enumerate() {
        if moves[..index]
            .iter()
            .any(|(previous, _)| previous.approx_eq(old_time))
        {
            return Ok(false);
        }
        let Some(keyframe) = keyframes.iter().find(|keyframe| {
            json_keyframe_time(keyframe)
                .expect("validated discrete keyframe time must remain valid")
                .approx_eq(old_time)
        }) else {
            return Ok(false);
        };
        moved.push(keyframe.clone());
    }
    keyframes.retain(|keyframe| {
        !moves.iter().any(|(old_time, _)| {
            json_keyframe_time(keyframe)
                .expect("validated discrete keyframe time must remain valid")
                .approx_eq(*old_time)
        })
    });
    let mut destinations = Vec::with_capacity(moved.len());
    for (mut keyframe, &(_, time)) in moved.into_iter().zip(moves) {
        keyframes.retain(|keyframe| {
            !json_keyframe_time(keyframe)
                .expect("validated discrete keyframe time must remain valid")
                .approx_eq(time)
        });
        destinations.retain(|keyframe| {
            !json_keyframe_time(keyframe)
                .expect("validated discrete keyframe time must remain valid")
                .approx_eq(time)
        });
        keyframe
            .as_object_mut()
            .expect("validated discrete keyframe must remain an object")
            .insert(
                "time".to_string(),
                serde_json::to_value(time).expect("keyframe time must serialize"),
            );
        destinations.push(keyframe);
    }
    keyframes.extend(destinations);
    sort_json_keyframes(keyframes)?;
    Ok(true)
}

pub fn copy_json_discrete_keyframes(
    value: &serde_json::Value,
    selected: &[Time],
    timeline_type: &str,
) -> Result<Option<KeyframeClipboard>, String> {
    let mut copied = Vec::new();
    for keyframe in json_keyframes(value)? {
        if selected.contains(&json_keyframe_time(keyframe)?) {
            copied.push(keyframe.clone());
        }
    }
    if copied.is_empty() {
        return Ok(None);
    }
    let times = copied
        .iter()
        .map(json_keyframe_time)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(KeyframeClipboard {
        len: copied.len(),
        times,
        values: Rc::new(JsonDiscreteClipboard {
            timeline_type: timeline_type.to_string(),
            keyframes: copied,
        }),
    }))
}

pub fn paste_json_discrete_keyframes(
    value: &mut serde_json::Value,
    clipboard: &KeyframeClipboard,
    times: &[Time],
    timeline_type: &str,
) -> Result<Option<Vec<Time>>, String> {
    let Some(source) = clipboard.values.downcast_ref::<JsonDiscreteClipboard>() else {
        return Ok(None);
    };
    if source.timeline_type != timeline_type || source.keyframes.len() != times.len() {
        return Ok(None);
    }
    let mut pasted = source.keyframes.clone();
    for (keyframe, time) in pasted.iter_mut().zip(times) {
        let keyframe = keyframe
            .as_object_mut()
            .ok_or_else(|| "discrete keyframe must be an object".to_string())?;
        keyframe.insert(
            "id".to_string(),
            serde_json::to_value(Uuid::new_v4()).expect("keyframe ID must serialize"),
        );
        keyframe.insert(
            "time".to_string(),
            serde_json::to_value(time).expect("keyframe time must serialize"),
        );
    }
    let paste_start = *times
        .iter()
        .min()
        .ok_or_else(|| "discrete keyframe paste has no times".to_string())?;
    let paste_end = *times
        .iter()
        .max()
        .ok_or_else(|| "discrete keyframe paste has no times".to_string())?;
    let keyframes = json_keyframes_mut(value)?;
    for keyframe in keyframes.iter() {
        json_keyframe_time(keyframe)?;
    }
    keyframes.retain(|keyframe| {
        let time = json_keyframe_time(keyframe)
            .expect("validated discrete keyframe time must remain valid");
        if paste_start == paste_end {
            time != paste_start
        } else {
            time < paste_start || time > paste_end
        }
    });
    keyframes.extend(pasted);
    sort_json_keyframes(keyframes)?;
    Ok(Some(times.to_vec()))
}

fn json_keyframes(value: &serde_json::Value) -> Result<&Vec<serde_json::Value>, String> {
    value
        .get("base")
        .and_then(|base| base.get("keyframes"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "discrete timeline is not keyframed".to_string())
}

fn json_keyframes_mut(
    value: &mut serde_json::Value,
) -> Result<&mut Vec<serde_json::Value>, String> {
    value
        .get_mut("base")
        .and_then(|base| base.get_mut("keyframes"))
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| "discrete timeline is not keyframed".to_string())
}

fn json_keyframe_time(value: &serde_json::Value) -> Result<Time, String> {
    value
        .get("time")
        .cloned()
        .ok_or_else(|| "discrete keyframe time is missing".to_string())
        .and_then(|time| {
            serde_json::from_value(time)
                .map_err(|error| format!("discrete keyframe time is invalid: {error}"))
        })
}

pub fn sort_json_keyframes(keyframes: &mut [serde_json::Value]) -> Result<(), String> {
    for keyframe in keyframes.iter() {
        json_keyframe_time(keyframe)?;
    }
    keyframes.sort_by_key(|keyframe| {
        json_keyframe_time(keyframe).expect("validated discrete keyframe time must remain valid")
    });
    Ok(())
}

pub fn add_scalar_keyframe(value: &mut TimelineValue<f32>, time: Time) -> bool {
    let current = value.value_at(time);
    if !matches!(value.base, TimelineBase::Keyframes(_)) {
        return false;
    }
    edit_curve_value(
        value,
        time,
        current,
        |_, _| false,
        CurveEditPolicy {
            unchanged_keyframe_is_noop: false,
            insert: CurveKeyframeInsert::InheritPreviousInterpolation,
        },
    );
    true
}

pub fn set_scalar_value(value: &mut TimelineValue<f32>, time: Time, next: f32) -> bool {
    if !next.is_finite() {
        return false;
    }
    edit_curve_value(
        value,
        time,
        next,
        |current, next| (*current - *next).abs() <= 0.000_001,
        CurveEditPolicy {
            unchanged_keyframe_is_noop: true,
            insert: CurveKeyframeInsert::InheritPreviousInterpolation,
        },
    )
}

pub fn delete_scalar_keyframe(value: &mut TimelineValue<f32>, time: Time) -> bool {
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    let Some(index) = keyframes
        .iter()
        .position(|keyframe| keyframe.time.approx_eq(time))
    else {
        return false;
    };
    keyframes.remove(index);
    if keyframes.is_empty() {
        value.base = TimelineBase::Const(0.0);
    }
    true
}

pub fn update_scalar_keyframe(
    value: &mut TimelineValue<f32>,
    old_time: Time,
    time: Time,
    next: f32,
) -> bool {
    if !next.is_finite() {
        return false;
    }
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    let Some(index) = keyframes
        .iter()
        .position(|keyframe| keyframe.time.approx_eq(old_time))
    else {
        return false;
    };
    let mut keyframe = keyframes.remove(index);
    keyframes.retain(|other| !other.time.approx_eq(time));
    keyframe.time = time;
    keyframe.value = next;
    keyframes.push(keyframe);
    keyframes.sort_by_key(|keyframe| keyframe.time);
    true
}

pub fn set_scalar_interpolation(
    value: &mut TimelineValue<f32>,
    owner_id: Uuid,
    interpolation: Interpolation,
) -> bool {
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    let Some(keyframe) = keyframes
        .iter_mut()
        .find(|keyframe| keyframe.id == owner_id)
    else {
        return false;
    };
    if keyframe.interpolation_to_next == interpolation {
        return false;
    }
    keyframe.interpolation_to_next = interpolation;
    true
}
