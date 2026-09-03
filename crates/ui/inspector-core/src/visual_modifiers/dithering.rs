use shrimply_core::{Color, timeline_value::TimelineValue};
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect,
    dithering::{DitheringColorMode, DitheringModifier},
};

use shrimply_project::project::Time;

use crate::{
    ControlKind, InspectorControl, InspectorControlAction, InspectorController, InspectorRuntime,
    InspectorSection, InspectorTarget, NumberSpec,
};

pub(super) fn presentation(
    value: &DitheringModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/pattern"),
            "Pattern",
            &value.pattern,
            runtime,
        )
        .live_commit("edit-dithering-pattern"),
    );
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/color_mode"),
            "Color mode",
            &value.color_mode,
            runtime,
        )
        .live_commit("edit-dithering-color-mode"),
    );
    section.add(
        super::modifier_scalar_control(
            format!("{base}/levels"),
            "Levels",
            &value.levels,
            runtime,
            NumberSpec {
                minimum: 2.0,
                maximum: 256.0,
                drag_step: 1.0,
                digits: 0,
                unit: "",
            },
            false,
        )
        .integer(),
    );
    section.add(super::modifier_scalar_control(
        format!("{base}/amount"),
        "Amount",
        &value.amount,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 1.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
        false,
    ));
    if value
        .color_mode
        .value_at(runtime.local_time.unwrap_or(Time::ZERO))
        == DitheringColorMode::Palette
    {
        for (palette_index, color) in value.palette.iter().enumerate() {
            let mut control = super::modifier_color_control(
                format!("{base}/palette/{palette_index}"),
                "Color",
                color,
                runtime,
            );
            control.prefix_icon = "user-trash-symbolic".to_string();
            control.tooltip = "Remove color".to_string();
            control.action = Some(InspectorControlAction::RemoveDitheringPaletteColor {
                modifier_id,
                color_id: color.id,
            });
            section.add(control);
        }
        let mut add =
            InspectorControl::new(ControlKind::Action, format!("{base}/palette/add"), "")
                .value("Add color")
                .action(InspectorControlAction::AddDitheringPaletteColor { modifier_id });
        add.prefix_icon = "list-add-symbolic".to_string();
        section.add(add);
    }
    section
}

pub(super) fn pattern<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<
    &'a TimelineValue<shrimply_video_modifiers::dithering::DitheringPattern>,
> {
    let value = modifier(item, path, "effect/effect/config/pattern")?;
    (value.pattern.id == timeline_id).then_some(&value.pattern)
}

pub(super) fn color_mode<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<DitheringColorMode>> {
    let value = modifier(item, path, "effect/effect/config/color_mode")?;
    (value.color_mode.id == timeline_id).then_some(&value.color_mode)
}

pub(super) fn palette_color<'a>(
    value: &'a DitheringModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<Color<u8>>> {
    let index = field
        .strip_prefix("effect/effect/config/palette/")?
        .parse::<usize>()
        .ok()?;
    value
        .palette
        .get(index)
        .filter(|color| color.id == timeline_id)
}

impl InspectorController {
    pub(crate) fn add_dithering_palette_color(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        dithering_modifier_mut(&mut project, target, modifier_id)?
            .palette
            .push(TimelineValue::new_const(Color::<u8>::WHITE));
        shrimply_project::project::commit_edit(&project, "add-dithering-palette-color");
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }

    pub(crate) fn remove_dithering_palette_color(
        &self,
        target: &InspectorTarget,
        modifier_id: uuid::Uuid,
        color_id: uuid::Uuid,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let modifier = dithering_modifier_mut(&mut project, target, modifier_id)?;
        let index = modifier
            .palette
            .iter()
            .position(|color| color.id == color_id)
            .ok_or_else(|| "dithering palette color is no longer available".to_string())?;
        modifier.palette.remove(index);
        shrimply_project::project::commit_edit(&project, "remove-dithering-palette-color");
        drop(project);
        super::refresh(&self.player_state);
        Ok(())
    }
}

fn dithering_modifier_mut<'a>(
    project: &'a mut shrimply_project::project::Project,
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
) -> Result<&'a mut DitheringModifier, String> {
    project
        .video_item_mut(super::video_address(target)?)
        .and_then(|item| {
            item.modifiers
                .iter_mut()
                .find(|modifier| modifier.id == modifier_id)
        })
        .and_then(|modifier| match &mut modifier.effect {
            ModifierEffect::Raster(effect) => match &mut **effect {
                RasterModifierEffect::Dithering(value) => Some(value),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| "dithering modifier is no longer available".to_string())
}

fn modifier<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    expected_field: &str,
) -> Option<&'a DitheringModifier> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != expected_field {
        return None;
    }
    let ModifierEffect::Raster(effect) = &modifier.effect else {
        return None;
    };
    let RasterModifierEffect::Dithering(value) = &**effect else {
        return None;
    };
    Some(value)
}
