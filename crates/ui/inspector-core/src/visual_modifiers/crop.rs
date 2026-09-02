use shrimply_video_modifiers::crop::CropModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &CropModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let (mode, edges, maximum, unit) = match value {
        CropModifier::Percentage(edges) => ("percentage", edges, 100.0, "%"),
        CropModifier::Pixels(edges) => ("pixels", edges, NumberSpec::default().maximum, "px"),
    };
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            format!("{base}/mode"),
            "Mode",
            mode,
            [
                ("percentage".into(), "Percentage".into()),
                ("pixels".into(), "Pixels".into()),
            ],
        )
        .immediate_commit("edit-crop-mode"),
    );
    for (field, label, timeline) in [
        ("top", "Top", &edges.top),
        ("right", "Right", &edges.right),
        ("bottom", "Bottom", &edges.bottom),
        ("left", "Left", &edges.left),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/values/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum,
                drag_step: 0.01,
                digits: 2,
                unit,
            },
            false,
        ));
    }
    section
}
