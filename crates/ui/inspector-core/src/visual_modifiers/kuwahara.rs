use shrimply_core::timeline_value::TimelineValue;
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect,
    kuwahara::{KuwaharaModifier, KuwaharaVersion},
};

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub const VERSION_COMMIT: &str = "edit-kuwahara-version";

pub(super) fn presentation(
    value: &KuwaharaModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/version"),
            "Version",
            &value.version,
            runtime,
        )
        .live_commit(VERSION_COMMIT),
    );
    section.add(super::modifier_scalar_control(
        format!("{base}/radius"),
        "Radius",
        &value.radius,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 32.0,
            drag_step: 0.01,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.set_target(modifier_id);
    section
}

pub fn version(effect: &ModifierEffect) -> Option<&TimelineValue<KuwaharaVersion>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::Kuwahara(value) = &**effect else {
        return None;
    };
    Some(&value.version)
}

pub fn version_mut(effect: &mut ModifierEffect) -> Option<&mut TimelineValue<KuwaharaVersion>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::Kuwahara(value) = &mut **effect else {
        return None;
    };
    Some(&mut value.version)
}

pub(super) fn version_at_path<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<KuwaharaVersion>> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != "effect/effect/config/version" {
        return None;
    }
    version(&modifier.effect).filter(|timeline| timeline.id == timeline_id)
}
