use super::*;

pub(super) fn current_keyframe_time(
    target: &InspectorTarget,
) -> Result<shrimply_project::project::Time, String> {
    with_controller(|controller| controller.current_keyframe_time(target))
}

pub(super) fn audio_modifier_expression_output(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput, String> {
    with_controller(|controller| {
        controller.audio_modifier_expression_output(target, modifier_id, timeline_id)
    })
}

pub(super) fn move_audio_modifier_keyframe(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    change: shrimply_inspector_core::AudioModifierKeyframeMove,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.move_audio_modifier_keyframe(target, modifier_id, timeline_id, change)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

pub(super) fn delete_audio_modifier_keyframe(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_audio_modifier_keyframe(target, modifier_id, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

pub(super) fn add_audio_modifier_keyframe(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_audio_modifier_keyframe(target, modifier_id, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

pub(super) fn set_audio_modifier_keyframe_interpolation(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_audio_modifier_keyframe_interpolation(
            target,
            modifier_id,
            timeline_id,
            owner_id,
            interpolation,
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

pub(super) fn seek_audio_modifier_keyframe(
    target: &InspectorTarget,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.seek_audio_modifier_keyframe(target, time))
}

pub(super) fn toggle_keyframe_playback() {
    with_controller(|controller| {
        controller.toggle_keyframe_playback();
        Ok(())
    })
    .expect("Qt inspector controller must be installed before graph playback");
}

pub(super) fn copy_audio_modifier_keyframes(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.copy_audio_modifier_keyframes(target, modifier_id, timeline_id, times)
    })
}

pub(super) fn paste_audio_modifier_keyframes(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_audio_modifier_keyframes(target, modifier_id, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

pub(super) fn set_audio_modifier_timeline_mode(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    path: &str,
    change: shrimply_inspector_core::TimelineModeChange<'_>,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_audio_modifier_timeline_mode(target, modifier_id, timeline_id, path, change)
    })
}

pub(super) fn set_audio_modifier_expression_source(
    target: &InspectorTarget,
    id: uuid::Uuid,
    path: &str,
    source: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_audio_modifier_expression_source(target, id, path, source)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

pub(super) fn apply_project_settings(
    width: i32,
    height: i32,
    fps_numerator: &str,
    fps_denominator: &str,
) -> Result<(), String> {
    let dimensions = shrimply_project::project::CanvasSize {
        width: u32::try_from(width).map_err(|_| "project width must be positive".to_string())?,
        height: u32::try_from(height).map_err(|_| "project height must be positive".to_string())?,
    };
    let numerator = fps_numerator
        .parse::<i64>()
        .map_err(|_| format!("invalid project frame-rate numerator: {fps_numerator}"))?;
    let denominator = fps_denominator
        .parse::<i64>()
        .map_err(|_| format!("invalid project frame-rate denominator: {fps_denominator}"))?;
    if numerator <= 0 || denominator <= 0 {
        return Err("project frame rate must be positive".to_string());
    }
    let frame_rate = shrimply_math_core::fraction_new(numerator, denominator);
    with_controller(|controller| controller.apply_project_settings(dimensions, frame_rate))
}

pub(super) fn can_paste_audio_modifiers(target: &InspectorTarget) -> bool {
    CONTROLLER.with_borrow(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller
                .as_ref()
                .expect("Qt inspector paste check requested before installation")
                .can_paste_audio_modifiers(
                    target,
                    clipboard
                        .as_ref()
                        .expect("Qt inspector paste check requested before installation"),
                )
        })
    })
}

pub(super) fn can_paste_visual_modifiers(target: &InspectorTarget) -> bool {
    CONTROLLER.with_borrow(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller
                .as_ref()
                .expect("Qt inspector paste check requested before installation")
                .can_paste_visual_modifiers(
                    target,
                    clipboard
                        .as_ref()
                        .expect("Qt inspector paste check requested before installation"),
                )
        })
    })
}

pub(super) fn add_visual_modifier(
    target: &InspectorTarget,
    kind: &str,
) -> Result<uuid::Uuid, String> {
    with_controller(|controller| controller.add_visual_modifier(target, kind))
}

pub(super) fn paste_visual_modifiers(target: &InspectorTarget) -> Result<usize, String> {
    with_controller(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller.paste_visual_modifiers(
                target,
                clipboard
                    .as_ref()
                    .expect("Qt inspector paste requested before installation"),
            )
        })
    })
}

pub(super) fn add_audio_modifier(target: &InspectorTarget, kind: &str) -> Result<(), String> {
    with_controller(|controller| controller.add_audio_modifier(target, kind))
}

pub(super) fn paste_audio_modifiers(target: &InspectorTarget) -> Result<usize, String> {
    with_controller(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller.paste_audio_modifiers(
                target,
                clipboard
                    .as_ref()
                    .expect("Qt inspector paste requested before installation"),
            )
        })
    })
}

pub(super) fn set_audio_cache_preset(
    target: &InspectorTarget,
    id: uuid::Uuid,
    preset: &str,
) -> Result<(), String> {
    with_controller(|controller| controller.set_audio_cache_preset(target, id, preset))
}

pub(super) fn toggle_audio_cache(target: &InspectorTarget, id: uuid::Uuid) -> Result<(), String> {
    with_controller(|controller| controller.toggle_audio_cache(target, id))
}

pub(super) fn set_visual_cache_quality(
    target: &InspectorTarget,
    id: uuid::Uuid,
    quality: &str,
) -> Result<(), String> {
    with_controller(|controller| controller.set_visual_cache_quality(target, id, quality))
}

pub(super) fn toggle_visual_cache(target: &InspectorTarget, id: uuid::Uuid) -> Result<(), String> {
    with_controller(|controller| controller.toggle_visual_cache(target, id))
}

pub(super) fn toggle_sam2_analysis(target: &InspectorTarget, id: uuid::Uuid) -> Result<(), String> {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt inspector preferences requested before installation"),
        )
        .compute_server_url
    });
    with_controller(|controller| controller.toggle_sam2_analysis(target, id, server_url))
}

pub(super) fn transparent_fill_analysis_control(
    target: &InspectorTarget,
    id: uuid::Uuid,
) -> Result<shrimply_inspector_core::AnalysisControlPresentation, String> {
    with_controller(|controller| controller.transparent_fill_analysis_control(target, id))
}

pub(super) fn camera_analysis_control(
    target: &InspectorTarget,
) -> Result<shrimply_inspector_core::AnalysisControlPresentation, String> {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt camera inspector used before preferences were installed"),
        )
        .compute_server_url
    });
    request_camera_models(&server_url);
    with_controller(|controller| controller.camera_analysis_control(target, &server_url))
}

pub(super) fn toggle_camera_analysis(target: &InspectorTarget) -> Result<(), String> {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt camera inspector used before preferences were installed"),
        )
        .compute_server_url
    });
    with_controller(|controller| controller.toggle_camera_analysis(target, server_url))
}

pub(super) fn toggle_transparent_fill_analysis(
    target: &InspectorTarget,
    id: uuid::Uuid,
) -> Result<(), String> {
    with_controller(|controller| controller.toggle_transparent_fill_analysis(target, id))
}

pub(super) fn set_sam2_point_label(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    point_id: uuid::Uuid,
    label: &str,
) -> Result<(), String> {
    let label = serde_json::from_value(serde_json::Value::String(label.to_string()))
        .map_err(|_| format!("unknown SAM2 point type: {label}"))?;
    with_controller(|controller| {
        controller.set_sam2_point_label(target, modifier_id, point_id, label)
    })
}

pub(super) fn set_sam2_model(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    model: &str,
) -> Result<(), String> {
    let model = serde_json::from_value(serde_json::Value::String(model.to_string()))
        .map_err(|_| format!("unknown SAM2 model: {model}"))?;
    with_controller(|controller| controller.set_sam2_model(target, modifier_id, model))
}

pub(super) fn set_sam2_point_position(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    point_id: uuid::Uuid,
    first: f64,
    second: f64,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_sam2_point_position(target, modifier_id, point_id, first, second)
    })
}

pub(super) fn perform_action(
    target: &InspectorTarget,
    action: InspectorAction,
) -> Result<Option<String>, String> {
    let mut confirmation = None;
    let result = match action {
        InspectorAction::Reset { path, value } => {
            with_controller(|controller| controller.set_value(target, &path, value))
        }
        InspectorAction::ResetFields { values } => {
            with_controller(|controller| controller.set_values(target, &values))
        }
        InspectorAction::ResetVideo { reset } => {
            with_controller(|controller| controller.reset_video(target, &reset))
        }
        InspectorAction::ResetManim { reset } => {
            with_controller(|controller| controller.reset_manim(target, &reset))
        }
        InspectorAction::ResetManimParameters { reset } => {
            with_controller(|controller| controller.reset_manim_parameters(target, &reset))
        }
        InspectorAction::SetBoolean { path, value } => {
            with_controller(|controller| controller.set_value(target, &path, Value::Bool(value)))
        }
        InspectorAction::ResetAudioModifier { id, effect } => {
            with_controller(|controller| controller.reset_audio_modifier(target, id, effect))
        }
        InspectorAction::SetAudioModifierEnabled { id, enabled } => {
            with_controller(|controller| controller.set_audio_modifier_enabled(target, id, enabled))
        }
        InspectorAction::CopyAudioModifier { id } => with_controller(|controller| {
            PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
                controller
                    .copy_audio_modifier(
                        target,
                        id,
                        clipboard
                            .as_ref()
                            .expect("Qt inspector copy requested before installation"),
                    )
                    .map(|name| confirmation = Some(format!("{name} copied")))
            })
        }),
        InspectorAction::MoveAudioModifier { id, offset } => {
            with_controller(|controller| controller.move_audio_modifier(target, id, offset))
        }
        InspectorAction::RemoveAudioModifier { id } => {
            with_controller(|controller| controller.remove_audio_modifier(target, id))
        }
        InspectorAction::ResetVisualModifier { id, effect } => {
            with_controller(|controller| controller.reset_visual_modifier(target, id, effect))
        }
        InspectorAction::SetVisualModifierEnabled { id, enabled } => {
            with_controller(|controller| {
                controller.set_visual_modifier_enabled(target, id, enabled)
            })
        }
        InspectorAction::CopyVisualModifier { id } => with_controller(|controller| {
            PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
                controller
                    .copy_visual_modifier(
                        target,
                        id,
                        clipboard
                            .as_ref()
                            .expect("Qt inspector copy requested before installation"),
                    )
                    .map(|name| confirmation = Some(format!("{name} copied")))
            })
        }),
        InspectorAction::MoveVisualModifier { id, offset } => {
            with_controller(|controller| controller.move_visual_modifier(target, id, offset))
        }
        InspectorAction::RemoveVisualModifier { id } => {
            with_controller(|controller| controller.remove_visual_modifier(target, id))
        }
        InspectorAction::SetAlphaMask {
            target: mask_target,
            enabled,
        } => {
            let result = with_controller(|controller| {
                controller.set_alpha_mask_enabled(target, mask_target, enabled)
            });
            if result.is_ok() {
                focus_alpha_mask(target, mask_target, enabled);
            }
            result
        }
        InspectorAction::ReloadAsset { asset, kind } => {
            shrimply_inspector_core::video::reload_asset(&asset, kind)
                .inspect(|()| crate::mark_dirty())
        }
    };
    result.map(|()| confirmation)
}
