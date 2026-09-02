use shrimply_video_modifiers::sam2::Sam2Modifier;

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &Sam2Modifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            format!("{base}/model"),
            "Model",
            super::enum_text(value.model),
            [
                ("tiny".to_string(), "Tiny".to_string()),
                ("small".to_string(), "Small".to_string()),
                ("base_plus".to_string(), "Base+".to_string()),
                ("large".to_string(), "Large".to_string()),
            ],
        )
        .immediate_commit("edit-sam2-model"),
    );
    for (field, label, timeline, minimum) in [
        ("threshold", "Threshold", &value.threshold, -8.0),
        ("softness", "Edge softness", &value.softness, 0.0),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                maximum: 8.0,
                drag_step: 0.01,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    for (point_index, point) in value.points.iter().enumerate() {
        section.add(super::modifier_vector2_control(
            format!("{base}/points/{point_index}/position"),
            format!("Point {}", point_index + 1),
            &point.position,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                digits: 2,
                unit: "x",
            },
            false,
        ));
        section.add(
            crate::selector::selector(
                format!("{base}/points/{point_index}/label"),
                "Point type",
                super::enum_text(point.label),
                [
                    ("foreground".to_string(), "Foreground".to_string()),
                    ("background".to_string(), "Background".to_string()),
                ],
            )
            .immediate_commit("edit-sam2-point-type"),
        );
        section.add(
            InspectorControl::new(
                ControlKind::Action,
                format!("{base}/points/{point_index}/remove"),
                "",
            )
            .value("Remove point")
            .tooltip("Remove point")
            .target(point.id),
        );
    }
    if let Some(box_prompt) = value.box_prompt {
        section.add(
            InspectorControl::new(ControlKind::Action, format!("{base}/box_prompt/remove"), "Box")
                .value("Remove box")
                .tooltip("Remove box")
                .target(box_prompt.id),
        );
    }
    section.add(
        InspectorControl::new(ControlKind::Action, format!("{base}/analyze"), "")
            .value("Analyze")
            .sensitive(!value.points.is_empty() || value.box_prompt.is_some())
            .tooltip("Precompute compact CPU mask frames so normal playback does not run SAM2"),
    );
    section.add(super::modifier_boolean_control(
        format!("{base}/invert"),
        "Invert",
        value.invert,
        "edit-sam2-invert",
    ));
    section
}
