use shrimply_video_modifiers::film_grain::FilmGrainModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &FilmGrainModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline, minimum, maximum) in [
        ("amount", "Amount", &value.amount, 0.0, 1.0),
        ("size", "Size", &value.size, 0.1, 20.0),
        ("colored", "Color", &value.colored, 0.0, 1.0),
    ] {
        section.add(super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                maximum,
                drag_step: 0.01,
                digits: 2,
                unit: "",
            },
            false,
        ));
    }
    section.add(
        super::modifier_scalar_control(
            format!("{base}/seed"),
            "Seed",
            &value.seed,
            runtime,
            NumberSpec {
                minimum: 0.0,
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
