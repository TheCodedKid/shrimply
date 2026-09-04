use glam::Vec3;
use shrimply_core::timeline_value::{
    CurveEditPolicy, CurveKeyframeInsert, Interpolation, TimelineBase, TimelineValue,
    TimelineValueType, edit_curve_value, insert_curve_keyframe,
};
use shrimply_project::project::Time;
use uuid::Uuid;

pub type Vec3Timeline = TimelineValue<Vec3>;
pub const COMPONENT_COUNT: usize = 3;

pub fn value_at(value: &Vec3Timeline, time: Time) -> Vec3 {
    value.value_at(time)
}

pub fn set_component(
    value: &mut Vec3Timeline,
    evaluation_time: Time,
    keyframe_time: Time,
    component: usize,
    next: f64,
    minimum: Option<f64>,
) -> bool {
    if !next.is_finite() {
        return false;
    }
    let mut components = value_at(value, evaluation_time).to_array();
    let Some(component) = components.get_mut(component) else {
        return false;
    };
    *component = next as f32;
    let mut current = Vec3::from_array(components);
    if let Some(minimum) = minimum {
        current = current.max(Vec3::splat(minimum as f32));
    }
    set_value(value, keyframe_time, current)
}

pub fn set_value(value: &mut Vec3Timeline, time: Time, next: Vec3) -> bool {
    if !next.is_finite() {
        return false;
    }
    edit_curve_value(
        value,
        time,
        next,
        |current, next| current.abs_diff_eq(*next, 0.000_001),
        CurveEditPolicy {
            unchanged_keyframe_is_noop: false,
            insert: CurveKeyframeInsert::InheritPreviousInterpolation,
        },
    )
}

pub fn set_keyframes_enabled(
    value: &mut Vec3Timeline,
    evaluation_time: Time,
    keyframe_time: Time,
    enabled: bool,
) -> bool {
    let current = value_at(value, evaluation_time);
    crate::keyframe_model::set_keyframes_enabled(value, keyframe_time, current, enabled)
}

pub fn set_expression_enabled(value: &mut Vec3Timeline, enabled: bool) -> bool {
    crate::keyframe_model::set_expression_enabled(
        value,
        enabled,
        crate::timeline_value::VECTOR3_EXPRESSION_DEFAULT,
    )
}

pub fn set_expression_source(value: &mut Vec3Timeline, source: String) -> bool {
    let Some(expression) = value.expression.as_mut() else {
        return false;
    };
    if expression.source == source {
        return false;
    }
    expression.source = source;
    true
}

pub fn add_keyframe(value: &mut Vec3Timeline, time: Time) -> bool {
    let current = value_at(value, time);
    let TimelineBase::Keyframes(keyframes) = &mut value.base else {
        return false;
    };
    if let Some(keyframe) = keyframes
        .iter_mut()
        .find(|keyframe| keyframe.time.approx_eq(time))
    {
        keyframe.time = time;
        keyframes.sort_by_key(|keyframe| keyframe.time);
    } else {
        insert_curve_keyframe(
            keyframes,
            Vec3::keyframe(time, current),
            CurveKeyframeInsert::InheritPreviousInterpolation,
        );
    }
    true
}

pub fn delete_keyframe(value: &mut Vec3Timeline, time: Time, frame_step: Time) -> bool {
    crate::keyframe_model::delete_discrete_keyframe(value, time, frame_step)
}

pub fn move_keyframes(value: &mut Vec3Timeline, moves: &[(Time, Time)]) -> bool {
    crate::keyframe_model::move_discrete_keyframes(value, moves)
}

pub fn copy_keyframes(
    value: &Vec3Timeline,
    selected: &[Time],
) -> Option<crate::keyframe_model::KeyframeClipboard> {
    crate::keyframe_model::copy_keyframes(value, selected)
}

pub fn paste_keyframes(
    value: &mut Vec3Timeline,
    clipboard: &crate::keyframe_model::KeyframeClipboard,
    times: &[Time],
) -> Option<Vec<Time>> {
    crate::keyframe_model::paste_keyframes(value, clipboard, times)
}

pub fn set_interpolation(
    value: &mut Vec3Timeline,
    owner_id: Uuid,
    interpolation: Interpolation,
) -> bool {
    crate::keyframe_model::set_interpolation(value, owner_id, interpolation)
}

pub fn format_value(
    value: Vec3,
    prefixes: [&str; COMPONENT_COUNT],
    digits: usize,
    unit: &str,
) -> String {
    format!(
        "{} {:.*}{}  {} {:.*}{}  {} {:.*}{}",
        prefixes[0],
        digits,
        value.x,
        unit,
        prefixes[1],
        digits,
        value.y,
        unit,
        prefixes[2],
        digits,
        value.z,
        unit,
    )
}
