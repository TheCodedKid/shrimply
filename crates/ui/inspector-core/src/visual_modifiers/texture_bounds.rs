use shrimply_core::{TextureAddressMode, timeline_value::TimelineValue};
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect, texture_bounds::TextureBoundsModifier,
};

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub const ADDRESS_MODE_COMMIT: &str = "edit-texture-addressing";

pub(super) fn presentation(
    value: &TextureBoundsModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("top", "Top", &value.edges.top),
        ("right", "Right", &value.edges.right),
        ("bottom", "Bottom", &value.edges.bottom),
        ("left", "Left", &value.edges.left),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/edges/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                drag_step: 0.01,
                digits: 2,
                unit: "px",
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/address_mode"),
            "Addressing",
            &value.address_mode,
            runtime,
        )
        .immediate_commit(ADDRESS_MODE_COMMIT),
    );
    section.set_target(modifier_id);
    section
}

pub fn address_mode(effect: &ModifierEffect) -> Option<&TimelineValue<TextureAddressMode>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::TextureBounds(value) = &**effect else {
        return None;
    };
    Some(&value.address_mode)
}

pub fn address_mode_mut(
    effect: &mut ModifierEffect,
) -> Option<&mut TimelineValue<TextureAddressMode>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::TextureBounds(value) = &mut **effect else {
        return None;
    };
    Some(&mut value.address_mode)
}

pub(super) fn address_mode_at_path<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<TextureAddressMode>> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != "effect/effect/config/address_mode" {
        return None;
    }
    address_mode(&modifier.effect).filter(|timeline| timeline.id == timeline_id)
}

pub(super) fn number<'a>(
    value: &'a TextureBoundsModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<f32>> {
    let timeline = match field {
        "effect/effect/config/edges/top" => &value.edges.top,
        "effect/effect/config/edges/right" => &value.edges.right,
        "effect/effect/config/edges/bottom" => &value.edges.bottom,
        "effect/effect/config/edges/left" => &value.edges.left,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}
