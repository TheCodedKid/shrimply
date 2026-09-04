use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect,
    sam2::{Sam2Model, Sam2Modifier, Sam2PointLabel},
    sam2_analysis,
};

use crate::{
    AnalysisControlPresentation, AnalysisTooltip, ControlKind, ControlRowRole, InspectorControl,
    InspectorControlAction, InspectorController, InspectorRuntime, InspectorSection,
    InspectorTarget, NumberSpec,
};

pub const EDIT_COMMIT: &str = "edit-sam2-prompts";
pub const ANALYZE_TOOLTIP: &str =
    "Precompute compact CPU mask frames so normal playback does not run SAM2";

pub(super) fn presentation(
    value: &Sam2Modifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            format!("{base}/model"),
            "Model",
            super::enum_text(value.model),
            [
                ("tiny".to_string(), "Tiny".to_string()),
                ("small".to_string(), "Small".to_string()),
                ("base_plus".to_string(), "Base+".to_string()),
                ("large".to_string(), "Large".to_string()),
            ],
        )
        .immediate_commit(EDIT_COMMIT)
        .action(InspectorControlAction::SetSam2Model { modifier_id }),
    );
    for (field, label, timeline, minimum) in [
        ("threshold", "Threshold", &value.threshold, -8.0),
        ("softness", "Edge softness", &value.softness, 0.0),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                maximum: 8.0,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    for (point_index, point) in value.points.iter().enumerate() {
        section.add(
            super::modifier_vector2_control(
                format!("{base}/points/{point_index}/position"),
                shrimply_i18n_core::text_args(
                    "Point %{number}",
                    &[("number", (point_index + 1).to_string())],
                ),
                &point.position,
                runtime,
                NumberSpec {
                    minimum: 0.0,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "x",
                },
                false,
            )
            .action(InspectorControlAction::SetSam2PointPosition {
                modifier_id,
                point_id: point.id,
            })
            .row_group(point.id, ControlRowRole::Primary),
        );
        section.add(
            crate::selector::selector(
                format!("{base}/points/{point_index}/label"),
                "Point type",
                super::enum_text(point.label),
                [
                    ("foreground".to_string(), "Foreground".to_string()),
                    ("background".to_string(), "Background".to_string()),
                ],
            )
            .immediate_commit(EDIT_COMMIT)
            .action(InspectorControlAction::SetSam2PointLabel {
                modifier_id,
                point_id: point.id,
            })
            .row_group(point.id, ControlRowRole::Auxiliary),
        );
        let mut remove = InspectorControl::new(
            ControlKind::Action,
            format!("{base}/points/{point_index}/remove"),
            "",
        )
        .value("Remove point")
        .tooltip("Remove point")
        .action(InspectorControlAction::RemoveSam2Point {
            modifier_id,
            point_id: point.id,
        })
        .row_group(point.id, ControlRowRole::TrailingAction);
        remove.prefix_icon = "user-trash-symbolic".to_string();
        section.add(remove);
    }
    if let Some(box_prompt) = value.box_prompt {
        let mut remove = InspectorControl::new(
            ControlKind::Action,
            format!("{base}/box_prompt/remove"),
            "Box",
        )
        .value("Remove box")
        .tooltip("Remove box")
        .action(InspectorControlAction::RemoveSam2Box {
            modifier_id,
            box_id: box_prompt.id,
        });
        remove.prefix_icon = "user-trash-symbolic".to_string();
        section.add(remove);
    }
    let can_analyze = !value.points.is_empty() || value.box_prompt.is_some();
    let prompt_signature = value.prompt_signature();
    let analysis = sam2_analysis_control(
        modifier_id,
        value.analysis_generation,
        prompt_signature,
        can_analyze,
    );
    section.add(super::modifier_analysis_control(
        format!("{base}/analyze"),
        analysis,
        InspectorControlAction::ToggleSam2Analysis {
            modifier_id,
            generation: value.analysis_generation,
            prompt_signature,
            can_analyze,
        },
    ));
    section.add(super::modifier_boolean_control(
        format!("{base}/invert"),
        "Invert",
        value.invert,
        EDIT_COMMIT,
    ));
    section.set_target(modifier_id);
    section
}

pub fn sam2_analysis_control(
    modifier_id: uuid::Uuid,
    generation: u64,
    prompt_signature: u64,
    can_analyze: bool,
) -> AnalysisControlPresentation {
    match sam2_analysis::get_for_prompt(modifier_id, generation, prompt_signature) {
        Some(sam2_analysis::Status::Running {
            message,
            completed_frames,
            total_frames,
            ..
        }) => AnalysisControlPresentation {
            label: message,
            progress: if total_frames == 0 {
                -1.0
            } else {
                completed_frames as f64 / total_frames as f64
            },
            tooltip: AnalysisTooltip::MessageKey(ANALYZE_TOOLTIP),
            sensitive: true,
            running: true,
            cancelling: false,
            terminal: false,
            suggested: false,
        },
        Some(sam2_analysis::Status::Complete { .. }) => AnalysisControlPresentation {
            label: "Reanalyze".to_string(),
            progress: -1.0,
            tooltip: AnalysisTooltip::MessageKey(ANALYZE_TOOLTIP),
            sensitive: can_analyze,
            running: false,
            cancelling: false,
            terminal: true,
            suggested: false,
        },
        Some(sam2_analysis::Status::Failed(error)) => AnalysisControlPresentation {
            label: if error == "Compute server connection failed" {
                error.clone()
            } else {
                "Analyze".to_string()
            },
            progress: -1.0,
            tooltip: AnalysisTooltip::RawError(error),
            sensitive: can_analyze,
            running: false,
            cancelling: false,
            terminal: true,
            suggested: true,
        },
        Some(sam2_analysis::Status::Cancelling) => AnalysisControlPresentation {
            label: "Cancelling…".to_string(),
            progress: -1.0,
            tooltip: AnalysisTooltip::MessageKey(ANALYZE_TOOLTIP),
            sensitive: false,
            running: false,
            cancelling: true,
            terminal: false,
            suggested: false,
        },
        Some(sam2_analysis::Status::Cancelled) => AnalysisControlPresentation {
            label: "Cancelled".to_string(),
            progress: -1.0,
            tooltip: AnalysisTooltip::MessageKey(ANALYZE_TOOLTIP),
            sensitive: can_analyze,
            running: false,
            cancelling: false,
            terminal: true,
            suggested: true,
        },
        None => AnalysisControlPresentation {
            label: "Analyze".to_string(),
            progress: -1.0,
            tooltip: AnalysisTooltip::MessageKey(ANALYZE_TOOLTIP),
            sensitive: can_analyze,
            running: false,
            cancelling: false,
            terminal: false,
            suggested: true,
        },
    }
}

impl InspectorController {
    pub fn sam2_presentation(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
    ) -> Result<InspectorSection, String> {
        let project = self.project.borrow();
        let item = project
            .video_item(super::video_address(target)?)
            .ok_or_else(|| "SAM2 item is no longer available".to_string())?;
        let (index, modifier) = item
            .modifiers
            .iter()
            .enumerate()
            .find(|(_, modifier)| modifier.id == modifier_id)
            .ok_or_else(|| "SAM2 modifier is no longer available".to_string())?;
        let ModifierEffect::Raster(effect) = &modifier.effect else {
            return Err("SAM2 modifier is no longer available".to_string());
        };
        let RasterModifierEffect::Sam2(value) = &**effect else {
            return Err("SAM2 modifier is no longer available".to_string());
        };
        Ok(presentation(
            value,
            index,
            modifier_id,
            crate::model::target_runtime(&project, &self.player_state, target),
        ))
    }

    pub fn set_sam2_model(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        model: Sam2Model,
    ) -> Result<(), String> {
        self.edit_sam2(target, modifier_id, |sam2| {
            if sam2.model == model {
                return false;
            }
            sam2.model = model;
            true
        })
    }

    pub fn set_sam2_point_position(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        point_id: uuid::Uuid,
        first: f64,
        second: f64,
    ) -> Result<(), String> {
        if !first.is_finite()
            || !second.is_finite()
            || !(0.0..=1.0).contains(&first)
            || !(0.0..=1.0).contains(&second)
        {
            return Err("SAM2 point coordinates must be between 0 and 1".to_string());
        }
        let path = {
            let project = self.project.borrow();
            let item = project
                .video_item(super::video_address(target)?)
                .ok_or_else(|| "SAM2 item is no longer available".to_string())?;
            let (modifier_index, modifier) = item
                .modifiers
                .iter()
                .enumerate()
                .find(|(_, modifier)| modifier.id == modifier_id)
                .ok_or_else(|| "SAM2 modifier is no longer available".to_string())?;
            let ModifierEffect::Raster(effect) = &modifier.effect else {
                return Err("SAM2 modifier is no longer available".to_string());
            };
            let RasterModifierEffect::Sam2(sam2) = &**effect else {
                return Err("SAM2 modifier is no longer available".to_string());
            };
            let point_index = sam2
                .points
                .iter()
                .position(|point| point.id == point_id)
                .ok_or_else(|| "SAM2 point is no longer available".to_string())?;
            format!(
                "/modifiers/{modifier_index}/effect/effect/config/points/{point_index}/position"
            )
        };
        self.set_vector2_value(
            target,
            &path,
            first,
            second,
            crate::InspectorCommit::Coalesced(EDIT_COMMIT),
        )
    }

    pub fn set_sam2_point_label(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        point_id: uuid::Uuid,
        label: Sam2PointLabel,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let sam2 = sam2_modifier_mut(&mut project, target, modifier_id)?;
        let point = sam2
            .points
            .iter_mut()
            .find(|point| point.id == point_id)
            .ok_or_else(|| "SAM2 point is no longer available".to_string())?;
        if point.label == label {
            return Ok(());
        }
        point.label = label;
        shrimply_project::project::commit_edit(&project, EDIT_COMMIT);
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }

    pub fn remove_sam2_point(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        point_id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let sam2 = sam2_modifier_mut(&mut project, target, modifier_id)?;
        let index = sam2
            .points
            .iter()
            .position(|point| point.id == point_id)
            .ok_or_else(|| "SAM2 point is no longer available".to_string())?;
        sam2.points.remove(index);
        if sam2.points.is_empty() && sam2.box_prompt.is_none() {
            sam2.seed_position = None;
        }
        shrimply_project::project::commit_edit(&project, EDIT_COMMIT);
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }

    pub fn remove_sam2_box(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        box_id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let sam2 = sam2_modifier_mut(&mut project, target, modifier_id)?;
        if !sam2
            .box_prompt
            .is_some_and(|box_prompt| box_prompt.id == box_id)
        {
            return Err("SAM2 box is no longer available".to_string());
        }
        sam2.box_prompt = None;
        if sam2.points.is_empty() {
            sam2.seed_position = None;
        }
        shrimply_project::project::commit_edit(&project, EDIT_COMMIT);
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }

    pub fn set_sam2_inverted(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        inverted: bool,
    ) -> Result<(), String> {
        self.edit_sam2(target, modifier_id, |sam2| {
            if sam2.invert == inverted {
                return false;
            }
            sam2.invert = inverted;
            true
        })
    }

    pub fn toggle_sam2_analysis(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        server_url: String,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let sam2 = sam2_modifier_mut(&mut project, target, modifier_id)?;
        let current_generation = sam2.analysis_generation;
        let prompt_signature = sam2.prompt_signature();
        if let Some((run_id, _)) =
            sam2_analysis::active_run(modifier_id, current_generation, prompt_signature)
            && sam2_analysis::cancel(modifier_id, run_id)
        {
            drop(project);
            super::refresh(&self.player_state);
            return Ok(());
        }
        if sam2.points.is_empty() && sam2.box_prompt.is_none() {
            return Err("SAM2 analysis requires a point or box prompt".to_string());
        }
        let next_generation = current_generation.wrapping_add(1).max(1);
        sam2.analysis_generation = next_generation;
        shrimply_project::project::commit_edit(&project, EDIT_COMMIT);
        drop(project);
        sam2_analysis::start(
            modifier_id,
            next_generation,
            sam2_analysis::Status::Running {
                message: "Sending request…".to_string(),
                completed_frames: 0,
                total_frames: 0,
                prompt_signature,
                server_url,
            },
        );
        super::refresh(&self.player_state);
        Ok(())
    }

    fn edit_sam2(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        edit: impl FnOnce(&mut Sam2Modifier) -> bool,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let sam2 = sam2_modifier_mut(&mut project, target, modifier_id)?;
        if !edit(sam2) {
            return Ok(());
        }
        shrimply_project::project::commit_edit(&project, EDIT_COMMIT);
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }
}

fn sam2_modifier_mut<'a>(
    project: &'a mut shrimply_project::project::Project,
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
) -> Result<&'a mut Sam2Modifier, String> {
    project
        .video_item_mut(super::video_address(target)?)
        .and_then(|item| {
            item.modifiers
                .iter_mut()
                .find(|modifier| modifier.id == modifier_id)
        })
        .and_then(|modifier| match &mut modifier.effect {
            ModifierEffect::Raster(effect) => match &mut **effect {
                RasterModifierEffect::Sam2(sam2) => Some(sam2),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| "SAM2 modifier is no longer available".to_string())
}
