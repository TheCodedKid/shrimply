use shrimply_core::timeline_value::TimelineValue;
use shrimply_project::project::{ItemAddress, Project};
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect,
    mask::{MaskMode, MaskModifier},
};

use crate::{
    ControlKind, InspectorControl, InspectorControlAction, InspectorController, InspectorRuntime,
    InspectorSection, InspectorTarget,
};

pub const MODE_COMMIT: &str = "edit-mask-mode";

pub(super) fn presentation(
    project: &Project,
    address: &ItemAddress,
    value: &MaskModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    let mut source =
        InspectorControl::new(ControlKind::Action, format!("{base}/item_id"), "Source")
            .value(source_label(project, Some(address), value.item_id))
            .tooltip("Drag onto a visual clip in the timeline")
            .drag_payload(modifier_id.to_string())
            .action_icon("edit-clear-symbolic", "Clear mask source");
    if value.item_id.is_some() {
        source = source.action(InspectorControlAction::ClearMaskSource { modifier_id });
    }
    section.add(source);
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/mode"),
            "Mode",
            &value.mode,
            runtime,
        )
        .live_commit(MODE_COMMIT),
    );
    section.add(super::modifier_boolean_control(
        format!("{base}/invert"),
        "Invert",
        value.invert,
        "edit-mask",
    ));
    section.set_target(modifier_id);
    section
}

pub fn source_label(
    project: &Project,
    address: Option<&ItemAddress>,
    id: Option<uuid::Uuid>,
) -> String {
    let Some(id) = id else {
        return shrimply_i18n_core::text("Drag onto a visual clip…").into_owned();
    };
    let Some(address) = address else {
        return shrimply_i18n_core::text("Missing item").into_owned();
    };
    let Some(tracks) = project.video_tracks_for_path(address.sequence_path()) else {
        return shrimply_i18n_core::text("Missing item").into_owned();
    };
    tracks
        .iter()
        .enumerate()
        .find_map(|(track_index, track)| {
            track
                .items
                .iter()
                .position(|item| item.id == id)
                .map(|item_index| {
                    shrimply_i18n_core::text_args(
                        "Track %{track} · Item %{item}",
                        &[
                            ("track", (track_index + 1).to_string()),
                            ("item", (item_index + 1).to_string()),
                        ],
                    )
                })
        })
        .unwrap_or_else(|| shrimply_i18n_core::text("Missing item").into_owned())
}

pub(super) fn mode<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<MaskMode>> {
    let value = modifier(item, path, "effect/effect/config/mode")?;
    (value.mode.id == timeline_id).then_some(&value.mode)
}

pub fn mode_value(effect: &ModifierEffect) -> Option<&TimelineValue<MaskMode>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::Mask(value) = &**effect else {
        return None;
    };
    Some(&value.mode)
}

pub fn mode_value_mut(effect: &mut ModifierEffect) -> Option<&mut TimelineValue<MaskMode>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::Mask(value) = &mut **effect else {
        return None;
    };
    Some(&mut value.mode)
}

pub(super) fn is_mode(item: &shrimply_project::project::VideoItem, path: &str) -> bool {
    modifier(item, path, "effect/effect/config/mode").is_some()
}

impl InspectorController {
    pub fn mask_presentation(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
    ) -> Result<InspectorSection, String> {
        let project = self.project.borrow();
        let address = super::video_address(target)?;
        let item = project
            .video_item(address)
            .ok_or_else(|| "mask item is no longer available".to_string())?;
        let (index, modifier) = item
            .modifiers
            .iter()
            .enumerate()
            .find(|(_, modifier)| modifier.id == modifier_id)
            .ok_or_else(|| "mask modifier is no longer available".to_string())?;
        let ModifierEffect::Raster(effect) = &modifier.effect else {
            return Err("mask modifier is no longer available".to_string());
        };
        let RasterModifierEffect::Mask(value) = &**effect else {
            return Err("mask modifier is no longer available".to_string());
        };
        Ok(presentation(
            &project,
            address,
            value,
            index,
            modifier_id,
            crate::model::target_runtime(&project, &self.player_state, target),
        ))
    }

    pub fn clear_mask_source(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let mask = mask_modifier_mut(&mut project, target, modifier_id)?;
        if mask.item_id.is_none() {
            return Ok(());
        }
        mask.item_id = None;
        shrimply_project::project::commit_edit(&project, "edit-mask");
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }

    pub fn set_mask_inverted(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        inverted: bool,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let mask = mask_modifier_mut(&mut project, target, modifier_id)?;
        if mask.invert == inverted {
            return Ok(());
        }
        mask.invert = inverted;
        shrimply_project::project::commit_edit(&project, "edit-mask");
        drop(project);
        shrimply_state::player_state::refresh_project(
            &self.player_state,
            shrimply_state::player_state::ProjectChange {
                video: true,
                ..Default::default()
            },
        );
        Ok(())
    }
}

pub fn set_mask_source(
    project: &mut Project,
    source: &ItemAddress,
    modifier_id: uuid::Uuid,
) -> Result<bool, String> {
    let ItemAddress::Video { .. } = source else {
        return Err("mask source is not a video item".to_string());
    };
    let tracks = project
        .video_tracks_for_path(source.sequence_path())
        .ok_or_else(|| "mask source sequence is no longer available".to_string())?;
    if project.video_item(source).is_none() {
        return Err("mask source item is no longer available".to_string());
    }
    let owner = tracks
        .iter()
        .find_map(|track| {
            track.items.iter().find_map(|item| {
                item.modifiers
                    .iter()
                    .any(|modifier| {
                        modifier.id == modifier_id
                            && matches!(
                                modifier.effect,
                                ModifierEffect::Raster(ref effect)
                                    if matches!(&**effect, RasterModifierEffect::Mask(_))
                            )
                    })
                    .then(|| ItemAddress::Video {
                        sequence_path: source.sequence_path().to_vec(),
                        track_id: track.id,
                        item_id: item.id,
                    })
            })
        })
        .ok_or_else(|| "mask modifier is no longer available".to_string())?;
    if owner.item_id() == source.item_id() {
        return Err("a mask source cannot be its own item".to_string());
    }
    let mask = mask_modifier_mut(project, &InspectorTarget::Item(owner), modifier_id)?;
    if mask.item_id == Some(source.item_id()) {
        return Ok(false);
    }
    mask.item_id = Some(source.item_id());
    shrimply_project::project::commit_edit(project, "edit-mask-source");
    Ok(true)
}

fn mask_modifier_mut<'a>(
    project: &'a mut Project,
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
) -> Result<&'a mut MaskModifier, String> {
    project
        .video_item_mut(super::video_address(target)?)
        .and_then(|item| {
            item.modifiers
                .iter_mut()
                .find(|modifier| modifier.id == modifier_id)
        })
        .and_then(|modifier| match &mut modifier.effect {
            ModifierEffect::Raster(effect) => match &mut **effect {
                RasterModifierEffect::Mask(value) => Some(value),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| "mask modifier is no longer available".to_string())
}

fn modifier<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    expected_field: &str,
) -> Option<&'a MaskModifier> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != expected_field {
        return None;
    }
    let ModifierEffect::Raster(effect) = &modifier.effect else {
        return None;
    };
    let RasterModifierEffect::Mask(value) = &**effect else {
        return None;
    };
    Some(value)
}
