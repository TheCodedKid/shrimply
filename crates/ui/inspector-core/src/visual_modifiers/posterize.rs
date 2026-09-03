use shrimply_video_modifiers::posterize::PosterizeModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &PosterizeModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(
        super::modifier_scalar_control(
            format!("/modifiers/{index}/effect/effect/config/levels"),
            "Levels",
            &value.levels,
            runtime,
            NumberSpec {
                minimum: 2.0,
                maximum: 256.0,
                drag_step: 1.0,
                digits: 0,
                ..NumberSpec::default()
            },
            false,
        )
        .integer(),
    );
    section
}
