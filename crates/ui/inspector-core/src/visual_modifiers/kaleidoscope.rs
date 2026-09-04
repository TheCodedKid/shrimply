use shrimply_video_modifiers::kaleidoscope::KaleidoscopeModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &KaleidoscopeModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_vector2_control(
        format!("{base}/center"),
        "Center",
        &value.center,
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
        super::modifier_scalar_control(
            format!("{base}/segments"),
            "Segments",
            &value.segments,
            runtime,
            NumberSpec {
                minimum: 2.0,
                maximum: 64.0,
                drag_step: 1.0,
                digits: 0,
                ..NumberSpec::default()
            },
            false,
        )
        .integer(),
    );
    section.add(super::modifier_scalar_control(
        format!("{base}/rotation_degrees"),
        "Rotation",
        &value.rotation_degrees,
        runtime,
        NumberSpec {
            drag_step: 1.0,
            unit: "deg",
            ..NumberSpec::default()
        },
        true,
    ));
    section.set_target(modifier_id);
    section
}
