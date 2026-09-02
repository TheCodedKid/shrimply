use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::texture_bounds::TextureBoundsModifier;

pub(super) fn presentation(
    value: &TextureBoundsModifier,
    index: usize,
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
        .immediate_commit("edit-texture-addressing"),
    );
    section
}
