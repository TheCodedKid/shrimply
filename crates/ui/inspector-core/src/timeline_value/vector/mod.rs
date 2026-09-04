use shrimply_core::timeline_value::{TimelineBase, TimelineValue, TimelineVector};
use shrimply_project::project::Time;

use crate::keyframe_graph::{GraphPoint, GraphSegment, KeyframeGraph, SpeedSegment};
use crate::{InspectorRuntime, ScalarGraph};

pub mod vec2;
pub mod vec3;

pub fn speed_graph<T: TimelineVector>(value: &TimelineValue<T>) -> KeyframeGraph {
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
                let start = pair[0].time;
                let end = pair[1].time;
                let seconds = end.signed_sub(start).as_secs_f64();
                (seconds > f64::EPSILON).then(|| SpeedSegment {
                    owner_id: pair[0].id,
                    start,
                    end,
                    value: T::distance(&pair[0].value, &pair[1].value) / seconds,
                    interpolation: pair[0].interpolation_to_next,
                })
            })
            .collect(),
        keys: keyframes.iter().map(|keyframe| keyframe.time).collect(),
        static_value: 0.0,
    }
}

pub(crate) fn scalar_speed_graph<T: TimelineVector>(
    value: &TimelineValue<T>,
    runtime: InspectorRuntime,
) -> Option<ScalarGraph> {
    let TimelineBase::Keyframes(_) = &value.base else {
        return None;
    };
    let KeyframeGraph::Speed { segments, keys, .. } = speed_graph(value) else {
        unreachable!("vector timeline must produce a speed graph")
    };
    Some(ScalarGraph {
        points: keys
            .into_iter()
            .map(|time| GraphPoint { time, value: 0.0 })
            .collect(),
        segments: segments
            .into_iter()
            .map(|segment| GraphSegment {
                owner_id: segment.owner_id,
                start: segment.start,
                end: segment.end,
                start_value: segment.value,
                end_value: segment.value,
                interpolation: shrimply_core::timeline_value::Interpolation::KEYFRAME
                    .iter()
                    .position(|candidate| *candidate == segment.interpolation)
                    .expect("vector interpolation must be available"),
            })
            .collect(),
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}
