use shrimply_video_modifiers::channel_mixer::ChannelMixerModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &ChannelMixerModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline) in [
        ("rr", "Red ← red", &value.rr),
        ("rg", "Red ← green", &value.rg),
        ("rb", "Red ← blue", &value.rb),
        ("gr", "Green ← red", &value.gr),
        ("gg", "Green ← green", &value.gg),
        ("gb", "Green ← blue", &value.gb),
        ("br", "Blue ← red", &value.br),
        ("bg", "Blue ← green", &value.bg),
        ("bb", "Blue ← blue", &value.bb),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum: -2.0,
                maximum: 2.0,
                drag_step: 0.01,
                digits: 2,
                unit: "",
            },
            false,
        ));
    }
    section
}
