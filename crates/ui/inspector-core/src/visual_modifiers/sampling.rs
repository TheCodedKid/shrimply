use shrimply_video_modifiers::sampling::SamplingModifier;

use crate::{InspectorRuntime, InspectorSection};

pub(super) fn presentation(
    value: &SamplingModifier,
    index: usize,
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
        .live_commit("edit-raster-sampling"),
    );
    section
}
