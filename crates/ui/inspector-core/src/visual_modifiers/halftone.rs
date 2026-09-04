use shrimply_project::project::Time;
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect,
    halftone::{HalftoneMode, HalftoneModifier},
};

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &HalftoneModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mode = value
        .mode
        .value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let mut section = InspectorSection::default();
    section.add(scalar(
        &base,
        "size",
        "Size",
        &value.size,
        runtime,
        NumberSpec {
            minimum: 1.0,
            maximum: 100.0,
            unit: "px",
            drag_step: 0.01,
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(scalar(
        &base,
        "angle_degrees",
        if mode == HalftoneMode::Monochrome {
            "Angle"
        } else {
            "Base angle"
        },
        &value.angle_degrees,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            unit: "deg",
            ..NumberSpec::default()
        },
        true,
    ));
    section.add(scalar(
        &base,
        "contrast",
        "Contrast",
        &value.contrast,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 10.0,
            drag_step: 0.01,
            ..NumberSpec::default()
        },
        false,
    ));
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/mode"),
            "Mode",
            &value.mode,
            runtime,
        )
        .live_commit("edit-halftone-mode"),
    );
    if mode != HalftoneMode::Monochrome {
        section.add(scalar(
            &base,
            "rgb_distance",
            "Channel offset",
            &value.rgb_distance,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 100.0,
                drag_step: 0.01,
                unit: "px",
                ..NumberSpec::default()
            },
            false,
        ));
        section.add(scalar(
            &base,
            "channel_angle_offset",
            "Channel angle offset",
            &value.channel_angle_offset,
            runtime,
            NumberSpec {
                drag_step: 1.0,
                unit: "deg",
                ..NumberSpec::default()
            },
            true,
        ));
    }
    section.set_target(modifier_id);
    section
}

fn scalar(
    base: &str,
    field: &str,
    label: &'static str,
    value: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    number: NumberSpec,
    rotating: bool,
) -> crate::InspectorControl {
    super::modifier_scalar_control(
        format!("{base}/{field}"),
        label,
        value,
        runtime,
        number,
        rotating,
    )
}

pub(super) fn mode<'a>(
    item: &'a shrimply_project::project::VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<HalftoneMode>> {
    let (modifier, field) = super::visual_modifier_at_path(item, path)?;
    if field != "effect/effect/config/mode" {
        return None;
    }
    let ModifierEffect::Raster(effect) = &modifier.effect else {
        return None;
    };
    let RasterModifierEffect::Halftone(value) = &**effect else {
        return None;
    };
    (value.mode.id == timeline_id).then_some(&value.mode)
}
