use shrimply_core::{VideoSampleMethod, timeline_value::TimelineValue};
use shrimply_video_modifiers::{ModifierEffect, rasterize::RasterizeModifier};

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub const SAMPLE_METHOD_COMMIT: &str = "edit-rasterize-upsampling";

pub(super) fn presentation(
    value: &RasterizeModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector2_control(
        format!("{base}/size"),
        "Size",
        value.size(),
        runtime,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/sample_method"),
            "Upsampling",
            &value.sample_method,
            runtime,
        )
        .live_commit(SAMPLE_METHOD_COMMIT),
    );
    section.set_target(modifier_id);
    section
}

pub fn sample_method(effect: &ModifierEffect) -> Option<&TimelineValue<VideoSampleMethod>> {
    let ModifierEffect::Rasterize(value) = effect else {
        return None;
    };
    Some(&value.sample_method)
}

pub fn sample_method_mut(
    effect: &mut ModifierEffect,
) -> Option<&mut TimelineValue<VideoSampleMethod>> {
    let ModifierEffect::Rasterize(value) = effect else {
        return None;
    };
    Some(&mut value.sample_method)
}

pub(super) fn sample_method_at_path<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<VideoSampleMethod>> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != "effect/effect/sample_method" {
        return None;
    }
    sample_method(&modifier.effect).filter(|timeline| timeline.id == timeline_id)
}
