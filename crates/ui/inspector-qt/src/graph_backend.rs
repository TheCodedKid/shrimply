use core::pin::Pin;

use cxx_qt_lib::{QString, QStringList};

use crate::backend::qobject::InspectorBackend;
use crate::section::InspectorControl;

impl InspectorBackend {
    pub fn seek_control_graph(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        numerator: i64,
        denominator: i64,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if control.scalar_graph.is_none() {
            return;
        }
        let result = exact_time(numerator, denominator).and_then(|time| {
            if matches!(
                control.kind,
                crate::section::ControlKind::LayeredBoolean
                    | crate::section::ControlKind::LayeredSelector
            ) {
                super::seek_discrete_keyframe(&target, time)
            } else if control.audio_modifier {
                super::seek_audio_modifier_keyframe(&target, time)
            } else {
                super::seek_scalar_keyframe(&target, time)
            }
        });
        self.as_mut().finish(result);
    }

    pub fn move_control_graph_keys(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        old_times: &QStringList,
        times: &QStringList,
        values: &QStringList,
    ) -> QStringList {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return QStringList::default();
        };
        let result = (|| {
            if control.scalar_graph.is_none() {
                return Err("control has no keyframe graph".to_string());
            }
            if old_times.len() != times.len() || times.len() != values.len() {
                return Err("keyframe graph columns have different lengths".to_string());
            }
            let path = control.timeline_path.as_deref().unwrap_or(&control.path);
            let moves = old_times
                .iter()
                .zip(times.iter())
                .map(|(old_time, time)| {
                    Ok((
                        parse_time(&old_time.to_string())?,
                        parse_time(&time.to_string())?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let canonical_times = if control.kind == crate::section::ControlKind::LayeredBoolean {
                super::move_bool_keyframes(&target, path, &moves)?
            } else if control.kind == crate::section::ControlKind::LayeredSelector {
                super::move_step_keyframes(&target, path, &moves)?
            } else {
                let modifier = control
                    .audio_modifier
                    .then(|| modifier_ids(&control))
                    .transpose()?;
                for (&(old_time, time), value) in moves.iter().zip(values.iter()) {
                    let change = shrimply_inspector_core::AudioModifierKeyframeMove {
                        old_time,
                        time,
                        displayed_value: parse_graph_value(&value.to_string())?,
                        store_multiplier: control.store_multiplier,
                    };
                    if let Some((modifier_id, timeline_id)) = modifier {
                        super::move_audio_modifier_keyframe(
                            &target,
                            modifier_id,
                            timeline_id,
                            change,
                        )?;
                    } else {
                        super::move_scalar_keyframe(&target, path, change)?;
                    }
                }
                moves.iter().map(|&(_, time)| time).collect()
            };
            Ok(canonical_times
                .into_iter()
                .map(|time| QString::from(crate::backend::time_text(time)))
                .collect())
        })();
        match result {
            Ok(times) => times,
            Err(error) => {
                self.as_mut().finish(Err(error));
                QStringList::default()
            }
        }
    }

    pub fn delete_control_graph_keys(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        times: &QStringList,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = (|| {
            if control.scalar_graph.is_none() {
                return Err("control has no keyframe graph".to_string());
            }
            let path = control.timeline_path.as_deref().unwrap_or(&control.path);
            let modifier = control
                .audio_modifier
                .then(|| modifier_ids(&control))
                .transpose()?;
            for time in times.iter() {
                let time = parse_time(&time.to_string())?;
                if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::delete_bool_keyframe(&target, path, time)?;
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::delete_step_keyframe(&target, path, time)?;
                } else if let Some((modifier_id, timeline_id)) = modifier {
                    super::delete_audio_modifier_keyframe(&target, modifier_id, timeline_id, time)?;
                } else {
                    super::delete_scalar_keyframe(&target, path, time)?;
                }
            }
            Ok(())
        })();
        self.as_mut().finish(result);
    }

    pub fn add_control_graph_key(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        numerator: i64,
        denominator: i64,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = exact_time(numerator, denominator).and_then(|time| {
            if control.scalar_graph.is_none() {
                return Err("control has no keyframe graph".to_string());
            }
            if control.kind == crate::section::ControlKind::LayeredBoolean {
                super::add_bool_keyframe(&target, timeline_path(&control), time)
            } else if control.kind == crate::section::ControlKind::LayeredSelector {
                super::add_step_keyframe(&target, timeline_path(&control), time)
            } else if control.audio_modifier {
                let (modifier_id, timeline_id) = modifier_ids(&control)?;
                super::add_audio_modifier_keyframe(&target, modifier_id, timeline_id, time)
            } else {
                super::add_scalar_keyframe(&target, timeline_path(&control), time)
            }
        });
        self.as_mut().finish(result);
    }

    pub fn copy_control_graph_keys(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        times: &QStringList,
    ) -> bool {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return false;
        };
        let result = parse_times(times)
            .and_then(|times| {
                if control.scalar_graph.is_none() {
                    return Err("control has no keyframe graph".to_string());
                }
                if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::copy_bool_keyframes(&target, timeline_path(&control), &times)
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::copy_step_keyframes(&target, timeline_path(&control), &times)
                } else if control.audio_modifier {
                    let (modifier_id, timeline_id) = modifier_ids(&control)?;
                    super::copy_audio_modifier_keyframes(&target, modifier_id, timeline_id, &times)
                } else {
                    super::copy_scalar_keyframes(&target, timeline_path(&control), &times)
                }
            })
            .map(|count| confirmation(count, "copied"));
        let copied = matches!(&result, Ok(Some(_)));
        self.as_mut().finish_confirmation(result);
        copied
    }

    pub fn paste_control_graph_keys(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        numerator: i64,
        denominator: i64,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = exact_time(numerator, denominator)
            .and_then(|time| {
                if control.scalar_graph.is_none() {
                    return Err("control has no keyframe graph".to_string());
                }
                if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::paste_bool_keyframes(&target, timeline_path(&control), time)
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::paste_step_keyframes(&target, timeline_path(&control), time)
                } else if control.audio_modifier {
                    let (modifier_id, timeline_id) = modifier_ids(&control)?;
                    super::paste_audio_modifier_keyframes(&target, modifier_id, timeline_id, time)
                } else {
                    super::paste_scalar_keyframes(&target, timeline_path(&control), time)
                }
            })
            .map(|count| confirmation(count, "pasted"));
        self.as_mut().finish_confirmation(result);
    }

    pub fn set_control_graph_interpolation(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        owner_id: &QString,
        interpolation: i32,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = (|| {
            if control.scalar_graph.is_none() {
                return Err("control has no keyframe graph".to_string());
            }
            let owner_id = uuid::Uuid::parse_str(&owner_id.to_string())
                .map_err(|_| "keyframe owner ID is invalid".to_string())?;
            let interpolation = usize::try_from(interpolation)
                .map_err(|_| "keyframe interpolation is invalid".to_string())?;
            if control.audio_modifier {
                let (modifier_id, timeline_id) = modifier_ids(&control)?;
                super::set_audio_modifier_keyframe_interpolation(
                    &target,
                    modifier_id,
                    timeline_id,
                    owner_id,
                    interpolation,
                )
            } else {
                super::set_scalar_keyframe_interpolation(
                    &target,
                    timeline_path(&control),
                    owner_id,
                    interpolation,
                )
            }
        })();
        self.as_mut().finish(result);
    }

    pub fn toggle_control_graph_playback(self: Pin<&mut Self>) {
        super::toggle_keyframe_playback();
    }
}

fn timeline_path(control: &InspectorControl) -> &str {
    control.timeline_path.as_deref().unwrap_or(&control.path)
}

fn modifier_ids(control: &InspectorControl) -> Result<(uuid::Uuid, uuid::Uuid), String> {
    control
        .target_id
        .zip(control.timeline_id)
        .ok_or_else(|| "audio modifier keyframe target is unavailable".to_string())
}

fn exact_time(numerator: i64, denominator: i64) -> Result<shrimply_project::project::Time, String> {
    if denominator <= 0 {
        return Err("keyframe graph time denominator must be positive".to_string());
    }
    Ok(shrimply_project::project::Time::from_fraction(
        numerator,
        denominator,
    ))
}

fn parse_time(value: &str) -> Result<shrimply_project::project::Time, String> {
    let (numerator, denominator) = value
        .split_once('/')
        .ok_or_else(|| format!("keyframe graph time is not an exact fraction: {value}"))?;
    exact_time(
        numerator
            .parse()
            .map_err(|_| format!("keyframe graph time numerator is invalid: {value}"))?,
        denominator
            .parse()
            .map_err(|_| format!("keyframe graph time denominator is invalid: {value}"))?,
    )
}

fn parse_times(values: &QStringList) -> Result<Vec<shrimply_project::project::Time>, String> {
    values
        .iter()
        .map(|value| parse_time(&value.to_string()))
        .collect()
}

fn parse_graph_value(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|_| format!("keyframe graph value is invalid: {value}"))
        .and_then(|value| {
            value
                .is_finite()
                .then_some(value)
                .ok_or_else(|| "keyframe graph value must be finite".to_string())
        })
}

fn confirmation(count: usize, action: &str) -> Option<String> {
    (count > 0).then(|| {
        if count == 1 {
            format!("1 keyframe {action}")
        } else {
            format!("{count} keyframes {action}")
        }
    })
}
