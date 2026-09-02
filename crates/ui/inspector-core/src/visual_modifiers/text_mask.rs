use shrimply_video_modifiers::text_mask::TextMaskModifier;

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &TextMaskModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(super::modifier_scalar_control(
        format!("{base}/amount"),
        "Amount",
        &value.amount,
        runtime,
        NumberSpec {
            minimum: 0.0,
            maximum: 1.0,
            drag_step: 0.01,
            digits: 2,
            unit: "",
        },
        false,
    ));
    section.add(selector(
        format!("{base}/partial_mode"),
        "Partial mode",
        enum_text(value.partial_mode),
        &[(&"clip", &"Clip"), (&"fade", &"Fade"), (&"snap", &"Snap")],
    ));
    section.add(selector(
        format!("{base}/direction"),
        "Direction",
        enum_text(value.direction),
        &[
            (&"left_to_right", &"Left to right"),
            (&"right_to_left", &"Right to left"),
            (&"top_to_bottom", &"Top to bottom"),
            (&"bottom_to_top", &"Bottom to top"),
        ],
    ));
    section
}

fn selector(
    path: String,
    label: &'static str,
    value: String,
    choices: &[(&str, &str)],
) -> InspectorControl {
    InspectorControl::new(ControlKind::Selector, path, label)
        .value(value)
        .choices(
            choices.iter().map(|choice| choice.0.to_string()).collect(),
            choices.iter().map(|choice| choice.1.to_string()).collect(),
        )
        .immediate_commit("edit-text-mask")
}

fn enum_text(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .expect("text mask enum must serialize")
        .as_str()
        .expect("text mask enum must be text")
        .to_string()
}
