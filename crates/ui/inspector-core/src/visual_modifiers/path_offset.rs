use shrimply_core::timeline_value::TimelineValue;
use shrimply_video_modifiers::path_offset::PathOffsetModifier;

use crate::{InspectorRuntime, InspectorSection, NumberSpec};

pub(super) fn presentation(
    value: &PathOffsetModifier,
    index: usize,
    modifier_id: uuid::Uuid,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let mut section = InspectorSection::default();
    for (field, label, timeline, minimum, integer) in [
        ("amplitude", "Amplitude", &value.amplitude, Some(0.0), false),
        ("spacing", "Spacing", &value.spacing, Some(0.1), false),
        ("seed", "Seed", &value.seed, Some(0.0), true),
        ("evolution", "Evolution", &value.evolution, None, true),
    ] {
        let mut number = NumberSpec {
            drag_step: if integer { 1.0 } else { 0.01 },
            digits: if integer { 0 } else { 2 },
            unit: if matches!(field, "amplitude" | "spacing") {
                "px"
            } else {
                ""
            },
            ..NumberSpec::default()
        };
        if let Some(minimum) = minimum {
            number.minimum = minimum;
        }
        let control = super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            number,
            false,
        );
        section.add(if integer { control.integer() } else { control });
    }
    section.set_target(modifier_id);
    section
}

pub(super) fn number<'a>(
    value: &'a PathOffsetModifier,
    field: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a TimelineValue<f32>> {
    let timeline = match field {
        "effect/effect/config/amplitude" => &value.amplitude,
        "effect/effect/config/spacing" => &value.spacing,
        "effect/effect/config/seed" => &value.seed,
        "effect/effect/config/evolution" => &value.evolution,
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}
