use core::pin::Pin;

use cxx_qt_lib::{QString, QStringList};

use crate::backend::qobject::InspectorBackend;
use crate::section::InspectorControl;

pub(crate) fn has_transform_controls(section: &crate::section::InspectorSection) -> bool {
    shrimply_inspector_core::keyframe_graph::has_transform_controls(section)
}

pub(crate) fn is_transform_path(path: &str) -> bool {
    shrimply_inspector_core::keyframe_graph::is_transform_path(path)
}

pub(crate) fn update_transform_graphs(
    document: &mut crate::list::InspectorDocument,
    live: &shrimply_inspector_core::transform::TransformLivePresentation,
) {
    for category in &mut document.categories {
        for item in &mut category.items {
            let section = match item {
                crate::item::InspectorListItem::Item(item) => &mut item.section,
                crate::item::InspectorListItem::Flat(section) => section,
            };
            shrimply_inspector_core::keyframe_graph::update_transform_graphs(section, live);
        }
    }
}

pub(crate) fn update_control_graphs(
    document: &mut crate::list::InspectorDocument,
    target: &shrimply_inspector_core::InspectorTarget,
) {
    let mut vector_source = None;
    for category in &mut document.categories {
        for item in &mut category.items {
            let section = match item {
                crate::item::InspectorListItem::Item(item) => &mut item.section,
                crate::item::InspectorListItem::Flat(section) => section,
            };
            for control in &mut section.controls {
                if control.scalar_graph.is_none() {
                    continue;
                }
                let graph = if matches!(
                    control.kind,
                    crate::section::ControlKind::LayeredVector2
                        | crate::section::ControlKind::LayeredVector3
                ) {
                    let Ok((value, runtime)) = vector_source
                        .get_or_insert_with(|| super::control_graph_source(target))
                        .as_ref()
                    else {
                        continue;
                    };
                    shrimply_inspector_core::keyframe_graph::vector_control_graph(
                        value, *runtime, control,
                    )
                    .expect("layered vector control must have a vector graph result")
                } else {
                    super::control_graph(target, control)
                };
                if let Ok(graph) = graph {
                    control.scalar_graph = graph;
                }
            }
        }
    }
}

impl InspectorBackend {
    pub fn control_graph_point_times(&self, category: i32, item: i32, control: i32) -> QStringList {
        self.control(category, item, control)
            .and_then(|control| control.scalar_graph.as_ref())
            .map(|graph| {
                graph
                    .points
                    .iter()
                    .map(|point| QString::from(crate::backend::time_text(point.time)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn control_graph_point_values(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> QStringList {
        self.control(category, item, control)
            .and_then(|control| control.scalar_graph.as_ref())
            .map(|graph| {
                graph
                    .points
                    .iter()
                    .map(|point| QString::from(point.value.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn control_graph_segments(&self, category: i32, item: i32, control: i32) -> QStringList {
        let speed = self.control(category, item, control).and_then(|control| {
            shrimply_inspector_core::InspectorGraphKind::for_control(control.kind)
        }) == Some(shrimply_inspector_core::InspectorGraphKind::Speed);
        self.control(category, item, control)
            .and_then(|control| control.scalar_graph.as_ref())
            .map(|graph| {
                graph
                    .segments
                    .iter()
                    .map(|segment| {
                        QString::from(if speed {
                            format!(
                                "{}\t{}\t{}\t{}\t{}",
                                segment.owner_id,
                                crate::backend::time_text(segment.start),
                                crate::backend::time_text(segment.end),
                                segment.start_value,
                                segment.interpolation,
                            )
                        } else {
                            format!(
                                "{}\t{}\t{}\t{}\t{}\t{}",
                                segment.owner_id,
                                crate::backend::time_text(segment.start),
                                crate::backend::time_text(segment.end),
                                segment.start_value,
                                segment.end_value,
                                segment.interpolation,
                            )
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn control_graph_timing(&self, category: i32, item: i32, control: i32) -> QStringList {
        self.control(category, item, control)
            .and_then(|control| control.scalar_graph.as_ref())
            .map(|graph| {
                [graph.range.0, graph.range.1, graph.frame_step]
                    .into_iter()
                    .flat_map(time_parts)
                    .map(|part| QString::from(part.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn control_graph_playhead(&self, category: i32, item: i32, control: i32) -> QStringList {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return QStringList::default();
        };
        let fallback = control.scalar_graph.as_ref().map(|graph| graph.playhead);
        super::current_keyframe_time(&target)
            .ok()
            .or(fallback)
            .map(|time| {
                time_parts(time)
                    .into_iter()
                    .map(|part| QString::from(part.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

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
        let result = shrimply_inspector_core::keyframe_model::exact_time(numerator, denominator)
            .and_then(|time| {
                if shrimply_inspector_core::InspectorGraphKind::uses_discrete_seek(control.kind) {
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return QStringList::default();
        }
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
                        shrimply_inspector_core::keyframe_model::parse_time(&old_time.to_string())?,
                        shrimply_inspector_core::keyframe_model::parse_time(&time.to_string())?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let canonical_times = if control.kind == crate::section::ControlKind::LayeredDrawing {
                super::move_paint_drawing_keyframes(&target, timeline_id(&control)?, &moves)?
            } else if control.kind == crate::section::ControlKind::LayeredBoolean {
                super::move_bool_keyframes(&target, path, &moves)?
            } else if control.kind == crate::section::ControlKind::LayeredSelector {
                super::move_step_keyframes(&target, path, &moves, &control.keyframe_commit_name)?
            } else if control.kind == crate::section::ControlKind::LayeredVector2 {
                super::move_vector2_keyframes(&target, path, &moves, &control.keyframe_commit_name)?
            } else if control.kind == crate::section::ControlKind::LayeredVector3 {
                super::move_vector3_keyframes(&target, path, &moves, &control.keyframe_commit_name)?
            } else if control.kind == crate::section::ControlKind::LayeredColor {
                super::move_color_keyframes(
                    &target,
                    path,
                    timeline_id(&control)?,
                    &moves,
                    &control.keyframe_commit_name,
                )?
            } else if control.kind == crate::section::ControlKind::LayeredText {
                super::move_text_keyframes(
                    &target,
                    path,
                    timeline_id(&control)?,
                    &moves,
                    text_keyframe_commits(&control)?,
                )?
            } else {
                let modifier = control
                    .audio_modifier
                    .then(|| modifier_ids(&control))
                    .transpose()?;
                let background_integer = background_integer(&control);
                for (&(old_time, time), value) in moves.iter().zip(values.iter()) {
                    let displayed_value =
                        shrimply_inspector_core::keyframe_model::parse_graph_value(
                            &value.to_string(),
                        )?;
                    let stored_value = if background_integer || modifier.is_some() {
                        control.store_number(if control.integer {
                            displayed_value.round()
                        } else {
                            displayed_value
                        })
                    } else {
                        control.map_number_for_storage(displayed_value)
                    };
                    let change = shrimply_inspector_core::AudioModifierKeyframeMove {
                        old_time,
                        time,
                        displayed_value: stored_value,
                        store_multiplier: 1.0,
                    };
                    if background_integer {
                        super::move_background_integer_keyframe(
                            &target,
                            path,
                            timeline_id(&control)?,
                            change,
                        )?;
                    } else if let Some((modifier_id, timeline_id)) = modifier {
                        super::move_audio_modifier_keyframe(
                            &target,
                            modifier_id,
                            timeline_id,
                            change,
                        )?;
                    } else {
                        super::move_scalar_keyframe(
                            &target,
                            path,
                            change,
                            shrimply_inspector_core::NumberConstraint {
                                integer: control.integer || control.number_constraint.integer,
                                ..control.number_constraint
                            },
                            &control.keyframe_commit_name,
                        )?;
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
            Ok(times) => {
                if timeline_path(&control).starts_with("/content/") {
                    super::mark_dirty();
                }
                times
            }
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
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
                let time = shrimply_inspector_core::keyframe_model::parse_time(&time.to_string())?;
                if control.kind == crate::section::ControlKind::LayeredDrawing {
                    super::delete_paint_drawing_keyframe(&target, timeline_id(&control)?, time)?;
                } else if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::delete_bool_keyframe(&target, path, time)?;
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::delete_step_keyframe(
                        &target,
                        path,
                        time,
                        &control.keyframe_commit_name,
                    )?;
                } else if control.kind == crate::section::ControlKind::LayeredVector2 {
                    super::delete_vector2_keyframe(
                        &target,
                        path,
                        time,
                        &control.keyframe_commit_name,
                    )?;
                } else if control.kind == crate::section::ControlKind::LayeredVector3 {
                    super::delete_vector3_keyframe(
                        &target,
                        path,
                        time,
                        &control.keyframe_commit_name,
                    )?;
                } else if control.kind == crate::section::ControlKind::LayeredColor {
                    super::delete_color_keyframe(
                        &target,
                        path,
                        timeline_id(&control)?,
                        time,
                        &control.keyframe_commit_name,
                    )?;
                } else if control.kind == crate::section::ControlKind::LayeredText {
                    super::delete_text_keyframe(
                        &target,
                        path,
                        timeline_id(&control)?,
                        time,
                        text_keyframe_commits(&control)?,
                    )?;
                } else if let Some((modifier_id, timeline_id)) = modifier {
                    super::delete_audio_modifier_keyframe(&target, modifier_id, timeline_id, time)?;
                } else if background_integer(&control) {
                    super::delete_background_integer_keyframe(
                        &target,
                        path,
                        timeline_id(&control)?,
                        time,
                    )?;
                } else if path == "/transform/rotation_degrees" {
                    super::delete_transform_scalar_keyframe(
                        &target,
                        path,
                        time,
                        &control.keyframe_commit_name,
                    )?;
                } else {
                    super::delete_scalar_keyframe(
                        &target,
                        path,
                        time,
                        &control.keyframe_commit_name,
                    )?;
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let result = shrimply_inspector_core::keyframe_model::exact_time(numerator, denominator)
            .and_then(|time| {
                if control.scalar_graph.is_none() {
                    return Err("control has no keyframe graph".to_string());
                }
                if control.kind == crate::section::ControlKind::LayeredDrawing {
                    super::add_paint_drawing_keyframe(&target, timeline_id(&control)?, time)
                } else if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::add_bool_keyframe(&target, timeline_path(&control), time)
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::add_step_keyframe(
                        &target,
                        timeline_path(&control),
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredVector2 {
                    super::add_vector2_keyframe(
                        &target,
                        timeline_path(&control),
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredVector3 {
                    super::add_vector3_keyframe(
                        &target,
                        timeline_path(&control),
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredColor {
                    super::add_color_keyframe(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredText {
                    super::add_text_keyframe(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        time,
                        text_keyframe_commits(&control)?,
                    )
                } else if background_integer(&control) {
                    super::add_background_integer_keyframe(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        time,
                    )
                } else if control.audio_modifier {
                    let (modifier_id, timeline_id) = modifier_ids(&control)?;
                    super::add_audio_modifier_keyframe(&target, modifier_id, timeline_id, time)
                } else {
                    super::add_scalar_keyframe(
                        &target,
                        timeline_path(&control),
                        time,
                        control.number_constraint,
                        &control.keyframe_commit_name,
                    )
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return false;
        }
        let result = parse_times(times)
            .and_then(|times| {
                if control.scalar_graph.is_none() {
                    return Err("control has no keyframe graph".to_string());
                }
                if control.kind == crate::section::ControlKind::LayeredDrawing {
                    super::copy_paint_drawing_keyframes(&target, timeline_id(&control)?, &times)
                } else if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::copy_bool_keyframes(&target, timeline_path(&control), &times)
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::copy_step_keyframes(&target, timeline_path(&control), &times)
                } else if control.kind == crate::section::ControlKind::LayeredVector2 {
                    super::copy_vector2_keyframes(&target, timeline_path(&control), &times)
                } else if control.kind == crate::section::ControlKind::LayeredVector3 {
                    super::copy_vector3_keyframes(&target, timeline_path(&control), &times)
                } else if control.kind == crate::section::ControlKind::LayeredColor {
                    super::copy_color_keyframes(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        &times,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredText {
                    super::copy_text_keyframes(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        &times,
                    )
                } else if background_integer(&control) {
                    super::copy_background_integer_keyframes(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        &times,
                    )
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let result = shrimply_inspector_core::keyframe_model::exact_time(numerator, denominator)
            .and_then(|time| {
                if control.scalar_graph.is_none() {
                    return Err("control has no keyframe graph".to_string());
                }
                if control.kind == crate::section::ControlKind::LayeredDrawing {
                    super::paste_paint_drawing_keyframes(&target, timeline_id(&control)?, time)
                } else if control.kind == crate::section::ControlKind::LayeredBoolean {
                    super::paste_bool_keyframes(&target, timeline_path(&control), time)
                } else if control.kind == crate::section::ControlKind::LayeredSelector {
                    super::paste_step_keyframes(
                        &target,
                        timeline_path(&control),
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredVector2 {
                    super::paste_vector2_keyframes(
                        &target,
                        timeline_path(&control),
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredVector3 {
                    super::paste_vector3_keyframes(
                        &target,
                        timeline_path(&control),
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredColor {
                    super::paste_color_keyframes(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        time,
                        &control.keyframe_commit_name,
                    )
                } else if control.kind == crate::section::ControlKind::LayeredText {
                    super::paste_text_keyframes(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        time,
                        text_keyframe_commits(&control)?,
                    )
                } else if background_integer(&control) {
                    super::paste_background_integer_keyframes(
                        &target,
                        timeline_path(&control),
                        timeline_id(&control)?,
                        time,
                    )
                } else if control.audio_modifier {
                    let (modifier_id, timeline_id) = modifier_ids(&control)?;
                    super::paste_audio_modifier_keyframes(&target, modifier_id, timeline_id, time)
                } else {
                    super::paste_scalar_keyframes(
                        &target,
                        timeline_path(&control),
                        time,
                        control.number_constraint,
                        &control.keyframe_commit_name,
                    )
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let result = (|| {
            if control.scalar_graph.is_none() {
                return Err("control has no keyframe graph".to_string());
            }
            let owner_id =
                shrimply_inspector_core::keyframe_model::parse_owner_id(&owner_id.to_string())?;
            let interpolation = usize::try_from(interpolation)
                .map_err(|_| "keyframe interpolation is invalid".to_string())?;
            if control.kind == crate::section::ControlKind::LayeredDrawing {
                super::set_paint_drawing_interpolation(
                    &target,
                    timeline_id(&control)?,
                    owner_id,
                    interpolation,
                )
            } else if control.kind == crate::section::ControlKind::LayeredVector2 {
                super::set_vector2_interpolation(
                    &target,
                    timeline_path(&control),
                    owner_id,
                    interpolation,
                    &control.keyframe_commit_name,
                )
            } else if control.kind == crate::section::ControlKind::LayeredVector3 {
                super::set_vector3_interpolation(
                    &target,
                    timeline_path(&control),
                    owner_id,
                    interpolation,
                    &control.keyframe_commit_name,
                )
            } else if control.kind == crate::section::ControlKind::LayeredColor {
                super::set_color_interpolation(
                    &target,
                    timeline_path(&control),
                    timeline_id(&control)?,
                    owner_id,
                    interpolation,
                    &control.keyframe_commit_name,
                )
            } else if control.kind == crate::section::ControlKind::LayeredText {
                super::set_text_keyframe_interpolation(
                    &target,
                    timeline_path(&control),
                    timeline_id(&control)?,
                    owner_id,
                    interpolation,
                    text_keyframe_commits(&control)?.interpolation,
                )
            } else if background_integer(&control) {
                super::set_background_integer_interpolation(
                    &target,
                    timeline_path(&control),
                    timeline_id(&control)?,
                    owner_id,
                    interpolation,
                )
            } else if control.audio_modifier {
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
                    &control.keyframe_commit_name,
                )
            }
        })();
        self.as_mut().finish(result);
    }

    pub fn text_interpolation_labels(&self) -> QStringList {
        shrimply_core::timeline_value::TextInterpolation::ALL
            .into_iter()
            .map(|interpolation| shrimply_i18n_qt::text(interpolation.label()))
            .collect()
    }

    pub fn text_interpolation_tooltips(&self) -> QStringList {
        shrimply_core::timeline_value::TextInterpolation::ALL
            .into_iter()
            .map(|interpolation| shrimply_i18n_qt::text(interpolation.tooltip()))
            .collect()
    }

    pub fn control_graph_text_interpolation(
        &self,
        category: i32,
        item: i32,
        control: i32,
        owner_id: &QString,
    ) -> i32 {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return -1;
        };
        if control.kind != crate::section::ControlKind::LayeredText {
            return -1;
        }
        if super::ensure_control_timeline(&target, &control).is_err() {
            return -1;
        }
        let Some(timeline_id) = control.timeline_id else {
            return -1;
        };
        let Ok(owner_id) =
            shrimply_inspector_core::keyframe_model::parse_owner_id(&owner_id.to_string())
        else {
            return -1;
        };
        super::text_keyframe_text_interpolation(
            &target,
            timeline_path(&control),
            timeline_id,
            owner_id,
        )
        .ok()
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
    }

    pub fn set_control_graph_text_interpolation(
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
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let result = (|| {
            if control.kind != crate::section::ControlKind::LayeredText {
                return Err("control is not a text keyframe graph".to_string());
            }
            let timeline_id = timeline_id(&control)?;
            let owner_id =
                shrimply_inspector_core::keyframe_model::parse_owner_id(&owner_id.to_string())?;
            let interpolation = usize::try_from(interpolation)
                .map_err(|_| "text interpolation is invalid".to_string())?;
            super::set_text_keyframe_text_interpolation(
                &target,
                timeline_path(&control),
                timeline_id,
                owner_id,
                interpolation,
                text_keyframe_commits(&control)?.text_interpolation,
            )
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

fn text_keyframe_commits(
    control: &InspectorControl,
) -> Result<shrimply_inspector_core::TextKeyframeCommits, String> {
    control
        .text_keyframe_commits
        .ok_or_else(|| "text control has no keyframe commit policy".to_string())
}

pub(crate) fn background_integer(control: &InspectorControl) -> bool {
    let path = timeline_path(control);
    control.integer && (path == "/content/star_points" || path.starts_with("/content/generator/"))
}

fn time_parts(time: shrimply_project::project::Time) -> [i64; 2] {
    [
        shrimply_core::timeline_value::fraction_numerator(time.seconds),
        shrimply_core::timeline_value::fraction_denominator(time.seconds),
    ]
}

fn modifier_ids(control: &InspectorControl) -> Result<(uuid::Uuid, uuid::Uuid), String> {
    control
        .target_id
        .zip(control.timeline_id)
        .ok_or_else(|| "audio modifier keyframe target is unavailable".to_string())
}

fn timeline_id(control: &InspectorControl) -> Result<uuid::Uuid, String> {
    control
        .timeline_id
        .ok_or_else(|| "keyframe timeline ID is unavailable".to_string())
}

fn parse_times(values: &QStringList) -> Result<Vec<shrimply_project::project::Time>, String> {
    values
        .iter()
        .map(|value| shrimply_inspector_core::keyframe_model::parse_time(&value.to_string()))
        .collect()
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
