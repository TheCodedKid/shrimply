use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use shrimply_core::timeline_value::{
    CurveEditPolicy, CurveKeyframeInsert, TextInterpolation, TimelineBase, TimelineBool,
    TimelineExpressionValue, TimelineStep, TimelineValue, TimelineValueType, edit_curve_value,
};
use shrimply_project::project::{ItemAddress, Time};

use crate::audio_modifiers::{audio_item_address, audio_modifier_evaluation_time};
use crate::{
    AudioModifierKeyframeMove, InspectorCommit, InspectorController, InspectorExpressionOutput,
    InspectorTarget, TextKeyframeCommits,
};

impl InspectorController {
    pub fn current_keyframe_time(&self, target: &InspectorTarget) -> Result<Time, String> {
        let address = match target {
            InspectorTarget::Item(address) | InspectorTarget::Transition { item: address, .. } => {
                address
            }
            InspectorTarget::Project | InspectorTarget::Track(_) => {
                return Err("inspector target has no keyframe time".to_string());
            }
        };
        self.project
            .borrow()
            .keyframe_time(
                address,
                shrimply_state::player_state::current_time(&self.player_state),
            )
            .ok_or_else(|| "the current item keyframe time is no longer available".to_string())
    }

    pub fn set_bool_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        next: bool,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.bool_timeline(target, path)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "boolean keyframe time is no longer available".to_string())?;
        if !crate::keyframe_model::set_discrete_value(
            &mut value,
            time,
            next.into(),
            runtime.frame_step,
        ) {
            return Ok(());
        }
        self.set_live_value(target, path, serialize_bool_timeline(value))
    }

    pub fn set_bool_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        self.set_video_step_keyframes_enabled::<TimelineBool>(target, path, enabled, commit)
    }

    pub fn set_text_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        next: String,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "text keyframe time is no longer available".to_string())?;
        if !crate::timeline_text::set_value(&mut value, time, next, runtime.frame_step) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Live,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Live),
            commit,
        )
    }

    pub fn set_text_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        let evaluation_time = runtime
            .local_time
            .ok_or_else(|| "text evaluation time is no longer available".to_string())?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "text keyframe time is no longer available".to_string())?;
        if !crate::timeline_text::set_keyframes_enabled(&mut value, evaluation_time, time, enabled)
        {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn text_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<InspectorExpressionOutput<String>, String> {
        self.video_modifier_expression_output(
            target,
            path,
            timeline_id,
            crate::timeline_text::video_value,
        )
    }

    pub fn set_text_expression_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_expression_enabled(&mut value, enabled) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn set_text_expression_source(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        source: String,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_expression_source(&mut value, source) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Live,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Live),
            commit,
        )
    }

    pub fn set_step_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.set_step_keyframes_enabled_with_commit(
            target,
            path,
            enabled,
            InspectorCommit::Immediate(crate::model::INSPECTOR_EDIT_COMMIT),
        )
    }

    pub fn set_step_keyframes_enabled_with_commit(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        if path.ends_with("/effect/effect/config/method")
            && sampling_method_target(&self.project.borrow(), target, path)?
        {
            return self.set_video_step_keyframes_enabled::<shrimply_core::VideoSampleMethod>(
                target, path, enabled, commit,
            );
        }
        if path.ends_with("/effect/effect/config/mode")
            && mask_mode_target(&self.project.borrow(), target, path)?
        {
            return self
                .set_video_step_keyframes_enabled::<shrimply_video_modifiers::mask::MaskMode>(
                    target, path, enabled, commit,
                );
        }
        if let Some(timeline) = crate::generated::step_timeline(path) {
            return match timeline {
                crate::generated::StepTimeline::Shape => self
                    .set_video_step_keyframes_enabled::<shrimply_project::project::ShapeKind>(
                        target, path, enabled, commit,
                    ),
                crate::generated::StepTimeline::Rounding => self
                    .set_video_step_keyframes_enabled::<
                        shrimply_project::project::ShapeRoundingStrategy,
                    >(target, path, enabled, commit),
                crate::generated::StepTimeline::HorizontalAlign => self
                    .set_video_step_keyframes_enabled::<
                        shrimply_project::project::TextHorizontalAlign,
                    >(target, path, enabled, commit),
                crate::generated::StepTimeline::VerticalAlign => self
                    .set_video_step_keyframes_enabled::<
                        shrimply_project::project::VerticalAlign,
                    >(target, path, enabled, commit),
                crate::generated::StepTimeline::Direction => self
                    .set_video_step_keyframes_enabled::<
                        shrimply_project::project::TextDirection,
                    >(target, path, enabled, commit),
                crate::generated::StepTimeline::FontStyle => self
                    .set_video_step_keyframes_enabled::<
                        shrimply_project::project::TextFontStyle,
                    >(target, path, enabled, commit),
            };
        }
        match path {
            crate::gaussian_3d::MODEL_ROTATION_ORDER_PATH => self
                .set_video_step_keyframes_enabled::<shrimply_3dgs::RotationOrder>(
                    target, path, enabled, commit,
                ),
            "/content/generator/mode" => {
                let project = self.project.borrow();
                let item = project
                    .video_item(video_item_address(target)?)
                    .ok_or_else(|| "background item is no longer available".to_string())?;
                let kind = crate::background::generator(item)
                    .map(shrimply_project::project::BackgroundGenerator::kind);
                drop(project);
                match kind {
                    Some(shrimply_project::project::BackgroundKind::ColorGradient) => self
                        .set_video_step_keyframes_enabled::<
                            shrimply_project::project::GradientMode,
                        >(target, path, enabled, commit),
                    Some(shrimply_project::project::BackgroundKind::PerlinNoise) => self
                        .set_video_step_keyframes_enabled::<shrimply_project::project::PerlinMode>(
                            target, path, enabled, commit,
                        ),
                    _ => Err("background mode is no longer available".to_string()),
                }
            }
            "/content/generator/curve" => self
                .set_video_step_keyframes_enabled::<shrimply_project::project::Curve>(
                    target, path, enabled, commit,
                ),
            "/content/generator/line_style" => self
                .set_video_step_keyframes_enabled::<shrimply_project::project::GridLineStyle>(
                    target, path, enabled, commit,
                ),
            "/content/generator/distribution" => self
                .set_video_step_keyframes_enabled::<shrimply_project::project::NoiseDistribution>(
                    target, path, enabled, commit,
                ),
            "/content/generator/color_mode" => self
                .set_video_step_keyframes_enabled::<shrimply_project::project::NoiseColorMode>(
                    target, path, enabled, commit,
                ),
            "/content/generator/fill" => {
                let project = self.project.borrow();
                let item = project
                    .video_item(video_item_address(target)?)
                    .ok_or_else(|| "background item is no longer available".to_string())?;
                let kind = crate::background::generator(item)
                    .map(shrimply_project::project::BackgroundGenerator::kind);
                drop(project);
                match kind {
                    Some(shrimply_project::project::BackgroundKind::Rainbow) => self
                        .set_video_step_keyframes_enabled::<shrimply_project::project::RainbowFill>(
                            target, path, enabled, commit,
                        ),
                    Some(shrimply_project::project::BackgroundKind::Voronoi) => self
                        .set_video_step_keyframes_enabled::<shrimply_project::project::VoronoiFill>(
                            target, path, enabled, commit,
                        ),
                    _ => Err("background fill is no longer available".to_string()),
                }
            }
            "/content/generator/bands" => self
                .set_video_step_keyframes_enabled::<shrimply_project::project::RainbowBands>(
                    target, path, enabled, commit,
                ),
            "/content/generator/metric" => self
                .set_video_step_keyframes_enabled::<shrimply_project::project::VoronoiMetric>(
                    target, path, enabled, commit,
                ),
            "/sample_method" => self
                .set_video_step_keyframes_enabled::<shrimply_core::VideoSampleMethod>(
                    target, path, enabled, commit,
                ),
            path if path.ends_with("/effect/effect/sample_method") => self
                .set_video_step_keyframes_enabled::<shrimply_core::VideoSampleMethod>(
                    target, path, enabled, commit,
                ),
            "/compositing/blend_mode" => self
                .set_video_step_keyframes_enabled::<shrimply_core::LayerBlendMode>(
                    target, path, enabled, commit,
                ),
            path if path.ends_with("/effect/effect/config/operation") => self
                .set_video_step_keyframes_enabled::<
                    shrimply_video_modifiers::erode_dilate::ErodeDilateOperation,
                >(target, path, enabled, commit),
            path if path.ends_with("/effect/effect/config/pattern") => self
                .set_video_step_keyframes_enabled::<
                    shrimply_video_modifiers::dithering::DitheringPattern,
                >(target, path, enabled, commit),
            path if path.ends_with("/effect/effect/config/color_mode") => self
                .set_video_step_keyframes_enabled::<
                    shrimply_video_modifiers::dithering::DitheringColorMode,
                >(target, path, enabled, commit),
            path if path.ends_with("/effect/effect/config/version") => self
                .set_video_step_keyframes_enabled::<
                    shrimply_video_modifiers::kuwahara::KuwaharaVersion,
                >(target, path, enabled, commit),
            path if path.ends_with("/effect/effect/config/row_offset_axis") => self
                .set_video_step_keyframes_enabled::<
                    shrimply_video_modifiers::repeat::RepeatOffsetAxis,
                >(target, path, enabled, commit),
            path if path.ends_with("/effect/effect/config/address_mode") => self
                .set_video_step_keyframes_enabled::<shrimply_core::TextureAddressMode>(
                    target, path, enabled, commit,
                ),
            path if path.ends_with("/effect/effect/config/mode") => self
                .set_video_step_keyframes_enabled::<
                    shrimply_video_modifiers::halftone::HalftoneMode,
                >(target, path, enabled, commit),
            _ => Err(format!("unknown step timeline: {path}")),
        }
    }

    fn set_video_step_keyframes_enabled<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String>
    where
        T: TimelineStep + DeserializeOwned + Serialize,
    {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let mut value: TimelineValue<T> = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("step timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid step timeline: {error}"))?;
        let evaluation_time = snapshot
            .runtime
            .local_time
            .ok_or_else(|| "step evaluation time is no longer available".to_string())?;
        let current = value.value_at(evaluation_time);
        let time = snapshot
            .runtime
            .keyframe_playhead
            .ok_or_else(|| "step keyframe time is no longer available".to_string())?;
        if !crate::keyframe_model::set_keyframes_enabled(&mut value, time, current, enabled) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serde_json::to_value(value).expect("step timeline must serialize"),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn bool_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<bool>, String> {
        let outcome = self.video_expression_output::<TimelineBool>(target, path)?;
        Ok(InspectorExpressionOutput {
            value: outcome.value.get(),
            error: outcome.error,
        })
    }

    pub fn step_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: Option<uuid::Uuid>,
    ) -> Result<InspectorExpressionOutput<String>, String> {
        if path.ends_with("/effect/effect/config/method")
            && sampling_method_target(&self.project.borrow(), target, path)?
        {
            let timeline_id = timeline_id
                .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
            return self.visual_modifier_step_expression_output(
                target,
                path,
                timeline_id,
                crate::visual_modifiers::sampling_method_timeline,
            );
        }
        if path.ends_with("/effect/effect/config/mode")
            && mask_mode_target(&self.project.borrow(), target, path)?
        {
            let timeline_id = timeline_id
                .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
            return self.visual_modifier_step_expression_output(
                target,
                path,
                timeline_id,
                crate::visual_modifiers::mask_mode,
            );
        }
        match path {
            path if crate::paint::paint_taper_path(path) => self
                .step_expression_output_as::<shrimply_project::project::PaintTaper>(
                target, path,
            ),
            crate::gaussian_3d::MODEL_ROTATION_ORDER_PATH => timeline_id
                .ok_or_else(|| "Gaussian timeline ID is unavailable".to_string())
                .and_then(|timeline_id| self.ensure_timeline(target, path, timeline_id))
                .and_then(|()| {
                    self.step_expression_output_as::<shrimply_3dgs::RotationOrder>(target, path)
                }),
            "/content/shape" => timeline_id
                .ok_or_else(|| "shape timeline ID is unavailable".to_string())
                .and_then(|timeline_id| self.ensure_timeline(target, path, timeline_id))
                .and_then(|()| {
                    self.step_expression_output_as::<shrimply_project::project::ShapeKind>(
                        target, path,
                    )
                }),
            "/content/rounding_strategy" => timeline_id
                .ok_or_else(|| "shape timeline ID is unavailable".to_string())
                .and_then(|timeline_id| self.ensure_timeline(target, path, timeline_id))
                .and_then(|()| {
                    self.step_expression_output_as::<
                        shrimply_project::project::ShapeRoundingStrategy,
                    >(target, path)
                }),
            "/sample_method" => {
                self.step_expression_output_as::<shrimply_core::VideoSampleMethod>(target, path)
            }
            path if path.ends_with("/effect/effect/sample_method") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::rasterize_sample_method_timeline,
                )
            }
            "/compositing/blend_mode" => {
                self.step_expression_output_as::<shrimply_core::LayerBlendMode>(target, path)
            }
            path if path.ends_with("/effect/effect/config/operation") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::erode_dilate_operation,
                )
            }
            path if path.ends_with("/effect/effect/config/pattern") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::dithering_pattern,
                )
            }
            path if path.ends_with("/effect/effect/config/color_mode") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::dithering_color_mode,
                )
            }
            path if path.ends_with("/effect/effect/config/version") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::kuwahara_version_timeline,
                )
            }
            path if path.ends_with("/effect/effect/config/row_offset_axis") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::repeat_offset_axis_timeline,
                )
            }
            path if path.ends_with("/effect/effect/config/address_mode") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::texture_bounds_address_mode_timeline,
                )
            }
            path if path.ends_with("/effect/effect/config/mode") => {
                let timeline_id = timeline_id
                    .ok_or_else(|| "visual modifier step timeline ID is unavailable".to_string())?;
                self.visual_modifier_step_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::halftone_mode,
                )
            }
            _ => Err(format!("unknown step timeline: {path}")),
        }
    }

    fn visual_modifier_step_expression_output<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        timeline: for<'a> fn(
            &'a shrimply_project::project::VideoItem,
            &str,
            uuid::Uuid,
        ) -> Option<&'a TimelineValue<T>>,
    ) -> Result<InspectorExpressionOutput<String>, String>
    where
        T: TimelineExpressionValue + TimelineStep,
    {
        let outcome = self.video_modifier_expression_output(target, path, timeline_id, timeline)?;
        let value = T::variants()
            .iter()
            .find(|variant| variant.value == outcome.value)
            .expect("evaluated modifier step must be a declared variant")
            .key
            .to_string();
        Ok(InspectorExpressionOutput {
            value,
            error: outcome.error,
        })
    }

    fn step_expression_output_as<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<String>, String>
    where
        T: TimelineExpressionValue + TimelineStep + DeserializeOwned,
    {
        let outcome = self.video_expression_output::<T>(target, path)?;
        let value = T::variants()
            .iter()
            .find(|variant| variant.value == outcome.value)
            .expect("evaluated timeline step must be one of its declared variants")
            .key
            .to_string();
        Ok(InspectorExpressionOutput {
            value,
            error: outcome.error,
        })
    }

    pub(crate) fn video_expression_output<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<InspectorExpressionOutput<T>, String>
    where
        T: TimelineExpressionValue + DeserializeOwned,
    {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let player = shrimply_state::player_state::snapshot(&self.player_state);
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let position = project
            .timeline_time_to_sequence(&address.track(), player.position)
            .ok_or_else(|| "expression time is no longer available".to_string())?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "expression item is no longer available".to_string())?;
        let serialized = transform_expression_value(item, path).unwrap_or_else(|| {
            serde_json::to_value(item)
                .expect("video item must serialize")
                .pointer(path)
                .cloned()
                .unwrap_or(Value::Null)
        });
        let value: TimelineValue<T> = serde_json::from_value(serialized)
            .map_err(|error| format!("invalid timeline expression value at {path}: {error}"))?;
        let audio =
            self.audio_sampler
                .borrow_mut()
                .sample(&project, player.position, player.revision);
        let evaluation = shrimply_evaluation::VisualEvaluation::for_item_with_audio(
            &project, item, position, &audio,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            &value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub(crate) fn video_modifier_expression_output<T>(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        timeline: for<'a> fn(
            &'a shrimply_project::project::VideoItem,
            &str,
            uuid::Uuid,
        ) -> Option<&'a TimelineValue<T>>,
    ) -> Result<InspectorExpressionOutput<T>, String>
    where
        T: TimelineExpressionValue,
    {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let player = shrimply_state::player_state::snapshot(&self.player_state);
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let position = project
            .timeline_time_to_sequence(&address.track(), player.position)
            .ok_or_else(|| "modifier expression time is no longer available".to_string())?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "modifier expression item is no longer available".to_string())?;
        let value = timeline(item, path, timeline_id)
            .ok_or_else(|| format!("modifier expression timeline is unavailable: {path}"))?;
        let audio =
            self.audio_sampler
                .borrow_mut()
                .sample(&project, player.position, player.revision);
        let evaluation = shrimply_evaluation::VisualEvaluation::for_item_with_audio(
            &project, item, position, &audio,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub fn set_color_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        next: shrimply_core::Color<u8>,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.color_timeline(target, path, timeline_id)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "color keyframe time is no longer available".to_string())?;
        if !crate::timeline_color::set_value(
            &mut value,
            runtime.local_time.unwrap_or(Time::ZERO),
            time,
            next,
        ) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Live,
            path,
            serialize_color_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Live),
            commit,
        )
    }

    pub fn set_color_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.color_timeline(target, path, timeline_id)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "color keyframe time is no longer available".to_string())?;
        if !crate::timeline_color::set_keyframes_enabled(
            &mut value,
            runtime.local_time.unwrap_or(Time::ZERO),
            time,
            enabled,
        ) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_color_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn color_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<InspectorExpressionOutput<shrimply_core::Color<u8>>, String> {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let player = shrimply_state::player_state::snapshot(&self.player_state);
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let position = project
            .timeline_time_to_sequence(&address.track(), player.position)
            .ok_or_else(|| "color expression time is no longer available".to_string())?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "color expression item is no longer available".to_string())?;
        let value = video_color_value(item, path, timeline_id)
            .ok_or_else(|| format!("color timeline is no longer available: {path}"))?;
        let audio =
            self.audio_sampler
                .borrow_mut()
                .sample(&project, player.position, player.revision);
        let evaluation = shrimply_evaluation::VisualEvaluation::for_item_with_audio(
            &project, item, position, &audio,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub fn color_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<shrimply_core::Color<u8>, String> {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let player = shrimply_state::player_state::snapshot(&self.player_state);
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let sequence_time = project
            .timeline_time_to_sequence(&address.track(), player.position)
            .ok_or_else(|| "color time is no longer available".to_string())?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "color item is no longer available".to_string())?;
        let local_time = shrimply_project::project::generated_item_time(item, sequence_time)
            .ok_or_else(|| "color time is outside the item".to_string())?;
        let value = video_color_value(item, path, timeline_id)
            .ok_or_else(|| format!("color timeline is no longer available: {path}"))?;
        Ok(crate::timeline_color::value_at(value, local_time))
    }

    pub fn move_color_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        moves: &[(Time, Time)],
        commit: InspectorCommit<'_>,
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.color_timeline(target, path, timeline_id)?;
        if !crate::timeline_color::move_keyframes(&mut value, &moves) {
            return Err("color keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value_with_commit(
            target,
            path,
            serialize_color_timeline(value),
            commit,
        )?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_color_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, _) = self.color_timeline(target, path, timeline_id)?;
        if !crate::timeline_color::delete_keyframe(&mut value, time) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_color_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn add_color_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, _) = self.color_timeline(target, path, timeline_id)?;
        if !crate::timeline_color::add_keyframe(&mut value, time) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_color_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn copy_color_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.color_timeline(target, path, timeline_id)?;
        let Some(mut clipboard) = crate::timeline_color::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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

    pub fn paste_color_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("color keyframes cannot be pasted at this time".to_string());
        };
        let (mut value, _) = self.color_timeline(target, path, timeline_id)?;
        let Some(pasted) = crate::timeline_color::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_color_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )?;
        Ok(pasted.len())
    }

    pub fn set_color_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let interpolation = crate::keyframe_model::interpolation(interpolation_index)?;
        let (mut value, _) = self.color_timeline(target, path, timeline_id)?;
        if !crate::timeline_color::set_interpolation(&mut value, owner_id, interpolation)? {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_color_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    fn color_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<
        (
            TimelineValue<shrimply_core::Color<u8>>,
            crate::InspectorRuntime,
        ),
        String,
    > {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "color item is no longer available".to_string())?;
        let value = video_color_value(item, path, timeline_id)
            .cloned()
            .ok_or_else(|| format!("color timeline is no longer available: {path}"))?;
        crate::timeline_color::validate_timeline(&value, timeline_id)?;
        let runtime = crate::model::target_runtime(&project, &self.player_state, target);
        Ok((value, runtime))
    }

    pub fn move_bool_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        moves: &[(Time, Time)],
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.bool_timeline(target, path)?;
        if !crate::keyframe_model::move_discrete_keyframes(&mut value, &moves) {
            return Err("boolean keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value(target, path, serialize_bool_timeline(value))?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_bool_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.bool_timeline(target, path)?;
        if !crate::keyframe_model::delete_discrete_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.set_value(target, path, serialize_bool_timeline(value))
    }

    pub fn add_bool_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, runtime) = self.bool_timeline(target, path)?;
        if !crate::keyframe_model::add_discrete_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.set_value(target, path, serialize_bool_timeline(value))
    }

    pub fn copy_bool_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.bool_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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

    pub fn paste_bool_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("boolean keyframes cannot be pasted at this time".to_string());
        };
        let (mut value, _) = self.bool_timeline(target, path)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.set_value(target, path, serialize_bool_timeline(value))?;
        Ok(pasted.len())
    }

    pub fn move_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        moves: &[(Time, Time)],
        commits: TextKeyframeCommits,
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::move_keyframes(&mut value, &moves) {
            return Err("text keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value_with_commit(
            target,
            path,
            serialize_text_timeline(value),
            InspectorCommit::Coalesced(commits.move_keyframe),
        )?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_text_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commits: TextKeyframeCommits,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::delete_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commits.delete),
        )
    }

    pub fn add_text_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commits: TextKeyframeCommits,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, runtime) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::add_keyframe(&mut value, time, runtime.frame_step) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commits.add),
        )
    }

    pub fn copy_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.text_timeline(target, path, timeline_id)?;
        let Some(mut clipboard) = crate::timeline_text::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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

    pub fn paste_text_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
        commits: TextKeyframeCommits,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("text keyframes cannot be pasted at this time".to_string());
        };
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        let Some(pasted) = crate::timeline_text::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commits.paste),
        )?;
        Ok(pasted.len())
    }

    pub fn set_text_keyframe_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
        commit_name: &str,
    ) -> Result<(), String> {
        let interpolation = crate::keyframe_model::interpolation(interpolation_index)?;
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_interpolation(&mut value, owner_id, interpolation)
            .unwrap_or(false)
        {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commit_name),
        )
    }

    pub fn text_keyframe_text_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
    ) -> Result<usize, String> {
        let (value, _) = self.text_timeline(target, path, timeline_id)?;
        let interpolation = crate::timeline_text::text_interpolation(&value, owner_id)
            .ok_or_else(|| "text keyframe is no longer available".to_string())?;
        TextInterpolation::ALL
            .iter()
            .position(|candidate| *candidate == interpolation)
            .ok_or_else(|| "text interpolation is unavailable".to_string())
    }

    pub fn set_text_keyframe_text_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
        commit_name: &str,
    ) -> Result<(), String> {
        let interpolation = TextInterpolation::ALL
            .get(interpolation_index)
            .copied()
            .ok_or_else(|| "text interpolation is invalid".to_string())?;
        let (mut value, _) = self.text_timeline(target, path, timeline_id)?;
        if !crate::timeline_text::set_text_interpolation(&mut value, owner_id, interpolation)? {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_text_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            InspectorCommit::Immediate(commit_name),
        )
    }

    pub fn seek_discrete_keyframe(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<(), String> {
        let address = video_item_address(target)?;
        let project = self.project.borrow();
        let position = project
            .keyframe_timeline_time(address, time)
            .ok_or_else(|| "boolean keyframe time is no longer available".to_string())?;
        drop(project);
        shrimply_state::player_state::seek_time(&self.player_state, position);
        Ok(())
    }

    pub fn move_step_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        moves: &[(Time, Time)],
        commit: InspectorCommit<'_>,
    ) -> Result<Vec<Time>, String> {
        let moves = self.canonical_video_keyframe_moves(target, moves)?;
        let (mut value, _) = self.json_step_timeline(target, path)?;
        if !crate::keyframe_model::move_json_discrete_keyframes(&mut value, &moves)? {
            return Err("selector keyframe move targets are no longer available".to_string());
        }
        self.set_live_keyframe_graph_value_with_commit(target, path, value, commit)?;
        Ok(moves.into_iter().map(|(_, time)| time).collect())
    }

    pub fn delete_step_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.json_step_timeline(target, path)?;
        if !crate::keyframe_model::delete_json_discrete_keyframe(
            &mut value,
            time,
            runtime.frame_step,
        )? {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            value,
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn add_step_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let time = self.canonical_video_keyframe_time(target, time)?;
        let (mut value, runtime) = self.json_step_timeline(target, path)?;
        if !crate::keyframe_model::add_json_discrete_keyframe(&mut value, time, runtime.frame_step)?
        {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            value,
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn copy_step_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.json_step_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_json_discrete_keyframes(
            &value,
            selected,
            step_timeline_type(&self.project.borrow(), target, path)?,
        )?
        else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
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

    pub fn paste_step_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("step keyframes cannot be pasted at this time".to_string());
        };
        let (mut value, _) = self.json_step_timeline(target, path)?;
        let Some(pasted) = crate::keyframe_model::paste_json_discrete_keyframes(
            &mut value,
            &clipboard,
            &times,
            step_timeline_type(&self.project.borrow(), target, path)?,
        )?
        else {
            return Ok(0);
        };
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            value,
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )?;
        Ok(pasted.len())
    }

    fn json_step_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<(Value, crate::InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let value = snapshot
            .value
            .pointer(path)
            .cloned()
            .ok_or_else(|| format!("step timeline is no longer available: {path}"))?;
        Ok((value, snapshot.runtime))
    }

    fn canonical_video_keyframe_time(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<Time, String> {
        let address = video_item_address(target)?;
        let project = self.project.borrow();
        project
            .keyframe_timeline_time(address, time)
            .and_then(|time| project.keyframe_time(address, time))
            .ok_or_else(|| "keyframe time is no longer available".to_string())
    }

    fn canonical_video_keyframe_moves(
        &self,
        target: &InspectorTarget,
        moves: &[(Time, Time)],
    ) -> Result<Vec<(Time, Time)>, String> {
        moves
            .iter()
            .map(|&(old_time, time)| {
                Ok((old_time, self.canonical_video_keyframe_time(target, time)?))
            })
            .collect()
    }

    fn bool_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<(TimelineValue<TimelineBool>, crate::InspectorRuntime), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let value: TimelineValue<TimelineBool> = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("boolean timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid boolean timeline: {error}"))?;
        Ok((value, snapshot.runtime))
    }

    fn text_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<(TimelineValue<String>, crate::InspectorRuntime), String> {
        if &self.target() != target {
            return Err("inspector target changed".to_string());
        }
        let project = self.project.borrow();
        let address = video_item_address(target)?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "text item is no longer available".to_string())?;
        let value = crate::timeline_text::video_value(item, path, timeline_id)
            .cloned()
            .ok_or_else(|| format!("text timeline is no longer available: {path}"))?;
        let runtime = crate::model::target_runtime(&project, &self.player_state, target);
        Ok((value, runtime))
    }

    pub fn set_scalar_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        enabled: bool,
        constraint: crate::NumberConstraint,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let mut value: TimelineValue<f32> = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("inspector timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid inspector timeline: {error}"))?;
        let evaluation_time = snapshot.runtime.local_time.unwrap_or(Time::ZERO);
        let keyframe_time = snapshot
            .runtime
            .keyframe_playhead
            .ok_or_else(|| "scalar keyframe time is no longer available".to_string())?;
        if !crate::timeline_value::scalar::set_keyframes_enabled(
            &mut value,
            evaluation_time,
            keyframe_time,
            enabled,
            constraint.into(),
        ) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn set_background_integer_keyframes_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.background_integer_timeline(target, path, timeline_id)?;
        let current = value.value_at(
            runtime
                .local_time
                .ok_or_else(|| "background integer evaluation time is unavailable".to_string())?,
        );
        let keyframe_time = runtime
            .keyframe_playhead
            .ok_or_else(|| "background integer keyframe time is unavailable".to_string())?;
        if !crate::keyframe_model::set_keyframes_enabled(
            &mut value,
            keyframe_time,
            current,
            enabled,
        ) {
            return Ok(());
        }
        let (_, keyframe_commit, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Structural,
            InspectorCommit::Immediate(keyframe_commit),
            true,
        )
    }

    pub fn set_background_integer_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        next: u32,
    ) -> Result<(), String> {
        let (mut value, runtime) = self.background_integer_timeline(target, path, timeline_id)?;
        let time = runtime
            .keyframe_playhead
            .ok_or_else(|| "background integer keyframe time is unavailable".to_string())?;
        let changed = match &mut value.base {
            TimelineBase::Const(current) if *current == next => false,
            TimelineBase::Const(current) => {
                *current = next;
                true
            }
            TimelineBase::Keyframes(keyframes) => {
                if let Some(keyframe) = keyframes.iter_mut().find(|keyframe| {
                    crate::keyframe_model::same_frame(keyframe.time, time, runtime.frame_step)
                }) {
                    let changed = keyframe.time != time || keyframe.value != next;
                    keyframe.time = time;
                    keyframe.value = next;
                    keyframes.sort_by_key(|keyframe| keyframe.time);
                    changed
                } else {
                    keyframes.push(u32::keyframe(time, next));
                    keyframes.sort_by_key(|keyframe| keyframe.time);
                    true
                }
            }
        };
        if !changed {
            return Ok(());
        }
        let (edit_commit, _, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Live,
            InspectorCommit::Coalesced(edit_commit),
            false,
        )
    }

    pub fn commit_background_integer_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<(), String> {
        self.background_integer_timeline(target, path, timeline_id)?;
        let (edit_commit, _, _) = integer_commits(path)?;
        let project = self.project.borrow();
        shrimply_project::project::commit_edit(&project, edit_commit);
        Ok(())
    }

    pub fn set_background_integer_expression_enabled(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        if !crate::keyframe_model::set_expression_enabled(&mut value, enabled, "x") {
            return Ok(());
        }
        let (_, _, expression_commit) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Structural,
            InspectorCommit::Immediate(expression_commit),
            true,
        )
    }

    pub fn set_background_integer_expression_source(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        source: String,
    ) -> Result<(), String> {
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        let Some(expression) = &mut value.expression else {
            return Err("background integer expression is not enabled".to_string());
        };
        if expression.source == source {
            return Ok(());
        }
        expression.source = source;
        let (_, _, expression_commit) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Live,
            InspectorCommit::Coalesced(expression_commit),
            false,
        )
    }

    pub fn background_integer_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<InspectorExpressionOutput<u32>, String> {
        self.background_integer_timeline(target, path, timeline_id)?;
        self.video_modifier_expression_output(
            target,
            path,
            timeline_id,
            |item, path, timeline_id| video_integer_value(item, path, timeline_id),
        )
    }

    pub fn background_integer_value(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<u32, String> {
        let (value, runtime) = self.background_integer_timeline(target, path, timeline_id)?;
        let time = runtime
            .local_time
            .ok_or_else(|| "background integer evaluation time is unavailable".to_string())?;
        Ok(value.value_at(time))
    }

    pub fn background_integer_graph(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<Option<crate::ScalarGraph>, String> {
        let (value, runtime) = self.background_integer_timeline(target, path, timeline_id)?;
        Ok(crate::background::integer_graph(&value, runtime))
    }

    pub fn move_background_integer_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        change: AudioModifierKeyframeMove,
    ) -> Result<(), String> {
        let time = self.canonical_scalar_keyframe_time(target, change.time)?;
        let next = background_integer(change.displayed_value * change.store_multiplier)?;
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        let TimelineBase::Keyframes(keyframes) = &mut value.base else {
            return Err("background integer keyframes are disabled".to_string());
        };
        let Some(index) = keyframes
            .iter()
            .position(|keyframe| keyframe.time.approx_eq(change.old_time))
        else {
            return Err("background integer keyframe is no longer available".to_string());
        };
        let mut keyframe = keyframes.remove(index);
        keyframes.retain(|other| !other.time.approx_eq(time));
        keyframe.time = time;
        keyframe.value = next;
        keyframes.push(keyframe);
        keyframes.sort_by_key(|keyframe| keyframe.time);
        let (_, keyframe_commit, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Live,
            InspectorCommit::Coalesced(keyframe_commit),
            false,
        )
    }

    pub fn delete_background_integer_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
    ) -> Result<(), String> {
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        let TimelineBase::Keyframes(keyframes) = &mut value.base else {
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
            value.base = TimelineBase::Const(removed.value);
        }
        let (_, keyframe_commit, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Structural,
            InspectorCommit::Immediate(keyframe_commit),
            true,
        )
    }

    pub fn add_background_integer_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
    ) -> Result<(), String> {
        let time = self.canonical_scalar_keyframe_time(target, time)?;
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        if !matches!(value.base, TimelineBase::Keyframes(_)) {
            return Ok(());
        }
        let current = value.value_at(time);
        edit_curve_value(
            &mut value,
            time,
            current,
            |_, _| false,
            CurveEditPolicy {
                unchanged_keyframe_is_noop: false,
                insert: CurveKeyframeInsert::InheritPreviousInterpolation,
            },
        );
        let (_, keyframe_commit, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Structural,
            InspectorCommit::Immediate(keyframe_commit),
            true,
        )
    }

    pub fn set_background_integer_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
    ) -> Result<(), String> {
        let interpolation = crate::keyframe_model::interpolation(interpolation_index)?;
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        if !crate::keyframe_model::set_interpolation(&mut value, owner_id, interpolation) {
            return Ok(());
        }
        let (_, keyframe_commit, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Structural,
            InspectorCommit::Immediate(keyframe_commit),
            true,
        )
    }

    pub fn copy_background_integer_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        selected: &[Time],
    ) -> Result<usize, String> {
        let (value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
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

    pub fn paste_background_integer_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        time: Time,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("keyframes cannot be pasted at this time".to_string());
        };
        let (mut value, _) = self.background_integer_timeline(target, path, timeline_id)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        let (_, keyframe_commit, _) = integer_commits(path)?;
        self.replace_background_integer(
            target,
            path,
            timeline_id,
            value,
            crate::model::EditKind::Structural,
            InspectorCommit::Immediate(keyframe_commit),
            true,
        )?;
        Ok(pasted.len())
    }

    fn background_integer_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
    ) -> Result<(TimelineValue<u32>, crate::InspectorRuntime), String> {
        if integer_commits(path).is_err() {
            return Err("background integer path is invalid".to_string());
        }
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        let value: TimelineValue<u32> = serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("background integer is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid background integer: {error}"))?;
        if value.id != timeline_id {
            return Err(format!("background integer is no longer available: {path}"));
        }
        Ok((value, snapshot.runtime))
    }

    fn replace_background_integer(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: uuid::Uuid,
        value: TimelineValue<u32>,
        kind: crate::model::EditKind,
        commit: InspectorCommit<'_>,
        inspector: bool,
    ) -> Result<(), String> {
        self.background_integer_timeline(target, path, timeline_id)?;
        self.replace_value_with_commit(
            target,
            kind,
            path,
            serde_json::to_value(value).expect("background integer timeline must serialize"),
            Some(shrimply_state::player_state::ProjectChange {
                video: true,
                inspector,
                ..Default::default()
            }),
            commit,
        )
    }

    pub fn scalar_timeline(
        &self,
        target: &InspectorTarget,
        path: &str,
    ) -> Result<TimelineValue<f32>, String> {
        let snapshot = self.snapshot();
        if &snapshot.target != target {
            return Err("inspector target changed".to_string());
        }
        serde_json::from_value(
            snapshot
                .value
                .pointer(path)
                .cloned()
                .ok_or_else(|| format!("inspector timeline is no longer available: {path}"))?,
        )
        .map_err(|error| format!("invalid inspector timeline: {error}"))
    }

    pub fn scalar_expression_output(
        &self,
        target: &InspectorTarget,
        path: &str,
        timeline_id: Option<uuid::Uuid>,
    ) -> Result<InspectorExpressionOutput, String> {
        if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. })) {
            if path.starts_with("/modifiers/")
                && let Some(timeline_id) = timeline_id
            {
                return self.video_modifier_expression_output(
                    target,
                    path,
                    timeline_id,
                    crate::visual_modifiers::visual_modifier_number,
                );
            }
            if let Some(timeline_id) = timeline_id {
                self.ensure_timeline(target, path, timeline_id)?;
            }
            return self.video_expression_output(target, path);
        }
        let value = self.scalar_timeline(target, path)?;
        let project = self.project.borrow();
        let address = audio_item_address(target)?;
        let local_time = audio_modifier_evaluation_time(&project, &self.player_state, target)?;
        let item = project
            .audio_item(address)
            .ok_or_else(|| "audio item is no longer available".to_string())?;
        let evaluation = shrimply_evaluation::VisualEvaluation::for_audio_item_local_time(
            &project, item, local_time,
        );
        let outcome = shrimply_evaluation::resolve_with_error(
            &value,
            &evaluation,
            &mut self.expression_cache.borrow_mut(),
        );
        Ok(InspectorExpressionOutput {
            value: outcome.value,
            error: outcome.error,
        })
    }

    pub fn move_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        change: AudioModifierKeyframeMove,
        constraint: crate::NumberConstraint,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let time = self.canonical_scalar_keyframe_time(target, change.time)?;
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::timeline_value::scalar::move_stored_keyframe(
            &mut value,
            change.old_time,
            time,
            (change.displayed_value * change.store_multiplier) as f32,
            constraint,
        ) {
            return Ok(());
        }
        self.set_live_keyframe_graph_value_with_commit(
            target,
            path,
            serialize_timeline(value),
            commit,
        )
    }

    pub fn delete_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::keyframe_model::delete_scalar_keyframe(&mut value, time) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn add_scalar_keyframe(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
        constraint: crate::NumberConstraint,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let time = self.canonical_scalar_keyframe_time(target, time)?;
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::timeline_value::scalar::add_keyframe(&mut value, time, constraint.into()) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn set_scalar_keyframe_interpolation(
        &self,
        target: &InspectorTarget,
        path: &str,
        owner_id: uuid::Uuid,
        interpolation_index: usize,
        commit: InspectorCommit<'_>,
    ) -> Result<(), String> {
        let interpolation = crate::keyframe_model::interpolation(interpolation_index)?;
        let mut value = self.scalar_timeline(target, path)?;
        if !crate::keyframe_model::set_scalar_interpolation(&mut value, owner_id, interpolation) {
            return Ok(());
        }
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )
    }

    pub fn copy_scalar_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        selected: &[Time],
    ) -> Result<usize, String> {
        let value = self.scalar_timeline(target, path)?;
        let Some(mut clipboard) = crate::keyframe_model::copy_keyframes(&value, selected) else {
            self.keyframe_clipboard.replace(None);
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
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

    pub fn paste_scalar_keyframes(
        &self,
        target: &InspectorTarget,
        path: &str,
        time: Time,
        constraint: crate::NumberConstraint,
        commit: InspectorCommit<'_>,
    ) -> Result<usize, String> {
        let Some(clipboard) = self.keyframe_clipboard.borrow().clone() else {
            return Ok(0);
        };
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
        let times =
            crate::keyframe_model::clipboard_paste_times(&project, Some(address), &clipboard, time);
        drop(project);
        let Some(times) = times else {
            return Err("keyframes cannot be pasted at this time".to_string());
        };
        let mut value = self.scalar_timeline(target, path)?;
        let Some(pasted) = crate::keyframe_model::paste_keyframes(&mut value, &clipboard, &times)
        else {
            return Ok(0);
        };
        crate::timeline_value::scalar::constrain_keyframes(&mut value, &pasted, constraint.into());
        self.replace_value_with_commit(
            target,
            crate::model::EditKind::Structural,
            path,
            serialize_timeline(value),
            crate::refresh::audio_path_change(target, path, crate::model::EditKind::Structural),
            commit,
        )?;
        Ok(pasted.len())
    }

    pub fn seek_scalar_keyframe(&self, target: &InspectorTarget, time: Time) -> Result<(), String> {
        let address = scalar_item_address(target)?;
        let project = self.project.borrow();
        let position = project
            .keyframe_timeline_time(address, time)
            .ok_or_else(|| "keyframe time is no longer available".to_string())?;
        drop(project);
        shrimply_state::player_state::seek_time(&self.player_state, position);
        Ok(())
    }

    fn canonical_scalar_keyframe_time(
        &self,
        target: &InspectorTarget,
        time: Time,
    ) -> Result<Time, String> {
        let project = self.project.borrow();
        let address = scalar_item_address(target)?;
        project
            .keyframe_timeline_time(address, time)
            .and_then(|timeline_time| project.keyframe_time(address, timeline_time))
            .ok_or_else(|| "scalar keyframe time is no longer available".to_string())
    }
}

fn transform_expression_value(
    item: &shrimply_project::project::VideoItem,
    path: &str,
) -> Option<Value> {
    let value = match path {
        "/transform/position" => serde_json::to_value(&item.transform.position),
        "/transform/anchor" => serde_json::to_value(&item.transform.anchor),
        "/transform/scale" => serde_json::to_value(&item.transform.scale),
        "/transform/shear" => serde_json::to_value(&item.transform.shear),
        "/transform/rotation_degrees" => serde_json::to_value(&item.transform.rotation_degrees),
        _ => return None,
    };
    Some(value.expect("transform timeline must serialize"))
}

fn serialize_timeline(value: TimelineValue<f32>) -> Value {
    serde_json::to_value(value).expect("scalar timeline must serialize")
}

fn background_integer(value: f64) -> Result<u32, String> {
    if value.is_finite() && value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value) {
        Ok(value as u32)
    } else {
        Err(format!("invalid background integer: {value}"))
    }
}

fn integer_commits(path: &str) -> Result<(&'static str, &'static str, &'static str), String> {
    if path.starts_with("/content/generator/") {
        Ok((
            crate::background::INTEGER_EDIT_COMMIT,
            crate::background::INTEGER_KEYFRAME_COMMIT,
            crate::background::INTEGER_EXPRESSION_COMMIT,
        ))
    } else if let Some(commits) = crate::generated::integer_commits(path) {
        Ok(commits)
    } else {
        Err(format!("unknown integer timeline: {path}"))
    }
}

fn video_integer_value<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<u32>> {
    if path.starts_with("/content/generator/") {
        crate::background::integer_value(item, timeline_id)
    } else {
        crate::generated::integer_value(item, path, timeline_id)
    }
}

fn video_color_value<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<shrimply_core::Color<u8>>> {
    if path.starts_with("/content/generator/") {
        crate::background::color_value(item, timeline_id)
    } else if path.starts_with("/content/") {
        crate::paint::color_value(item, path, timeline_id)
            .or_else(|| crate::generated::color_value(item, path, timeline_id))
            .or_else(|| {
                let shrimply_project::project::VideoItemContent::Obj(scene) = &item.content else {
                    return None;
                };
                crate::scene_3d::color(scene, timeline_id)
            })
    } else {
        crate::visual_modifiers::visual_modifier_color(item, path, timeline_id)
    }
}

fn serialize_bool_timeline(value: TimelineValue<TimelineBool>) -> Value {
    serde_json::to_value(value).expect("boolean timeline must serialize")
}

fn serialize_text_timeline(value: TimelineValue<String>) -> Value {
    serde_json::to_value(value).expect("text timeline must serialize")
}

fn serialize_color_timeline(value: TimelineValue<shrimply_core::Color<u8>>) -> Value {
    serde_json::to_value(value).expect("color timeline must serialize")
}

fn video_item_address(target: &InspectorTarget) -> Result<&ItemAddress, String> {
    let InspectorTarget::Item(address @ ItemAddress::Video { .. }) = target else {
        return Err("boolean keyframe target is not a video item".to_string());
    };
    Ok(address)
}

fn scalar_item_address(target: &InspectorTarget) -> Result<&ItemAddress, String> {
    match target {
        InspectorTarget::Item(
            address @ (ItemAddress::Audio { .. } | ItemAddress::Video { .. }),
        ) => Ok(address),
        _ => Err("scalar keyframe target is not an audio or video item".to_string()),
    }
}

fn step_timeline_type(
    project: &shrimply_project::project::Project,
    target: &InspectorTarget,
    path: &str,
) -> Result<&'static str, String> {
    if path.ends_with("/effect/effect/config/method")
        && sampling_method_target(project, target, path)?
    {
        return Ok(std::any::type_name::<shrimply_core::VideoSampleMethod>());
    }
    if path.ends_with("/effect/effect/config/mode") && mask_mode_target(project, target, path)? {
        return Ok(std::any::type_name::<
            shrimply_video_modifiers::mask::MaskMode,
        >());
    }
    if let Some(timeline_type) = crate::generated::step_timeline_type(path) {
        return Ok(timeline_type);
    }
    if crate::paint::paint_taper_path(path) {
        return Ok(std::any::type_name::<shrimply_project::project::PaintTaper>());
    }
    match path {
        crate::gaussian_3d::MODEL_ROTATION_ORDER_PATH => {
            Ok(std::any::type_name::<shrimply_3dgs::RotationOrder>())
        }
        "/sample_method" => Ok(std::any::type_name::<shrimply_core::VideoSampleMethod>()),
        path if path.ends_with("/effect/effect/sample_method") => {
            Ok(std::any::type_name::<shrimply_core::VideoSampleMethod>())
        }
        "/compositing/blend_mode" => Ok(std::any::type_name::<shrimply_core::LayerBlendMode>()),
        path if path.ends_with("/effect/effect/config/operation") => Ok(std::any::type_name::<
            shrimply_video_modifiers::erode_dilate::ErodeDilateOperation,
        >()),
        path if path.ends_with("/effect/effect/config/pattern") => Ok(std::any::type_name::<
            shrimply_video_modifiers::dithering::DitheringPattern,
        >()),
        path if path.ends_with("/effect/effect/config/color_mode") => Ok(std::any::type_name::<
            shrimply_video_modifiers::dithering::DitheringColorMode,
        >()),
        path if path.ends_with("/effect/effect/config/version") => Ok(std::any::type_name::<
            shrimply_video_modifiers::kuwahara::KuwaharaVersion,
        >()),
        path if path.ends_with("/effect/effect/config/row_offset_axis") => {
            Ok(std::any::type_name::<
                shrimply_video_modifiers::repeat::RepeatOffsetAxis,
            >())
        }
        path if path.ends_with("/effect/effect/config/address_mode") => {
            Ok(std::any::type_name::<shrimply_core::TextureAddressMode>())
        }
        path if path.ends_with("/effect/effect/config/mode") => Ok(std::any::type_name::<
            shrimply_video_modifiers::halftone::HalftoneMode,
        >()),
        _ => Err(format!("unknown step timeline: {path}")),
    }
}

fn sampling_method_target(
    project: &shrimply_project::project::Project,
    target: &InspectorTarget,
    path: &str,
) -> Result<bool, String> {
    let item = project
        .video_item(video_item_address(target)?)
        .ok_or_else(|| "video item is no longer available".to_string())?;
    Ok(crate::visual_modifiers::is_sampling_method(item, path))
}

fn mask_mode_target(
    project: &shrimply_project::project::Project,
    target: &InspectorTarget,
    path: &str,
) -> Result<bool, String> {
    let item = project
        .video_item(video_item_address(target)?)
        .ok_or_else(|| "video item is no longer available".to_string())?;
    Ok(crate::visual_modifiers::is_mask_mode(item, path))
}
