use shrimply_core::{VideoSampleMethod, timeline_value::TimelineValue};
use shrimply_video_modifiers::{ModifierEffect, RasterModifierEffect, sampling::SamplingModifier};

use crate::{InspectorRuntime, InspectorSection};

pub const METHOD_COMMIT: &str = "edit-raster-sampling";

pub(super) fn presentation(
    value: &SamplingModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::layered_step_selector(
            format!("/modifiers/{index}/effect/effect/config/method"),
            "Method",
            &value.method,
            runtime,
        )
        .live_commit(METHOD_COMMIT),
    );
    section.set_target(modifier_id);
    section
}

pub fn method(effect: &ModifierEffect) -> Option<&TimelineValue<VideoSampleMethod>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::Sampling(value) = &**effect else {
        return None;
    };
    Some(&value.method)
}

pub fn method_mut(effect: &mut ModifierEffect) -> Option<&mut TimelineValue<VideoSampleMethod>> {
    let ModifierEffect::Raster(effect) = effect else {
        return None;
    };
    let RasterModifierEffect::Sampling(value) = &mut **effect else {
        return None;
    };
    Some(&mut value.method)
}

pub(super) fn method_at_path<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<VideoSampleMethod>> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != "effect/effect/config/method" {
        return None;
    }
    method(&modifier.effect).filter(|timeline| timeline.id == timeline_id)
}

pub(super) fn is_method(item: &shrimply_project::project::VideoItem, path: &str) -> bool {
    let Some((modifier, field)) = super::visual_modifier_at_path(item, path) else {
        return false;
    };
    field == "effect/effect/config/method" && method(&modifier.effect).is_some()
}
