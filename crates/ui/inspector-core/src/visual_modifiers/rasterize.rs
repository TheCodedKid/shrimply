use shrimply_video_modifiers::rasterize::RasterizeModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &RasterizeModifier,
    index: usize,
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
        .live_commit("edit-rasterize-upsampling"),
    );
    section
}
