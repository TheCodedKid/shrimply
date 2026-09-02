use shrimply_video_modifiers::opacity::OpacityModifier;

use crate::{InspectorControl, InspectorRuntime, NumberSpec};

#[derive(Clone, Debug, PartialEq)]
pub struct OpacityModifierPresentation {
    pub opacity: InspectorControl,
}

pub(super) fn presentation(
    value: &OpacityModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> OpacityModifierPresentation {
    OpacityModifierPresentation {
        opacity: super::modifier_scalar_control(
            format!("/modifiers/{index}/effect/effect/config/opacity"),
            "Opacity",
            &value.opacity,
            runtime,
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                digits: 2,
                ..NumberSpec::default()
            },
            false,
        ),
    }
}
