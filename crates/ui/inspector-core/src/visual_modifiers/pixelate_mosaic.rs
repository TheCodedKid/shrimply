use shrimply_video_modifiers::pixelate_mosaic::PixelateMosaicModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &PixelateMosaicModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("block_width", "Block width", &value.block_width),
        ("block_height", "Block height", &value.block_height),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: 1.0,
                maximum: 512.0,
                drag_step: 1.0,
                digits: 0,
                ..NumberSpec::default()
            },
            false,
        ));
    }
    section
}
