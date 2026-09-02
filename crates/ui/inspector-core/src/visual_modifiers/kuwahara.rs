use shrimply_video_modifiers::kuwahara::KuwaharaModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &KuwaharaModifier,
    index: usize,
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
        .live_commit("edit-kuwahara-version"),
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
    section
}
