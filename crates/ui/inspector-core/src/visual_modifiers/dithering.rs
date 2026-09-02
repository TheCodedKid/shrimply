use shrimply_video_modifiers::dithering::{DitheringColorMode, DitheringModifier};

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_project::project::Time;

pub(super) fn presentation(
    value: &DitheringModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/pattern"),
            "Pattern",
            &value.pattern,
            runtime,
        )
        .live_commit("edit-dithering-pattern"),
    );
    section.add(
        crate::selector::layered_step_selector(
            format!("{base}/color_mode"),
            "Color mode",
            &value.color_mode,
            runtime,
        )
        .live_commit("edit-dithering-color-mode"),
    );
    section.add(super::modifier_scalar_control(
        format!("{base}/levels"),
        "Levels",
        &value.levels,
        runtime,
        NumberSpec {
            minimum: 2.0,
            maximum: 256.0,
            drag_step: 1.0,
            digits: 0,
            unit: "",
        },
        false,
    ));
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
    if value
        .color_mode
        .value_at(runtime.local_time.unwrap_or(Time::ZERO))
        == DitheringColorMode::Palette
    {
        for (palette_index, color) in value.palette.iter().enumerate() {
            section.add(super::modifier_color_control(
                format!("{base}/palette/{palette_index}"),
                "Color",
                color,
                runtime,
            ));
            section.add(
                InspectorControl::new(
                    ControlKind::Action,
                    format!("{base}/palette/{palette_index}/remove"),
                    "",
                )
                .value("Remove color")
                .tooltip("Remove color"),
            );
        }
        section.add(
            InspectorControl::new(ControlKind::Action, format!("{base}/palette/add"), "")
                .value("Add color"),
        );
    }
    section
}
