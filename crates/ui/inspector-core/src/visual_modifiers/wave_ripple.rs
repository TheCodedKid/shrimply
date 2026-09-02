use crate::{InspectorRuntime, InspectorSection, NumberSpec};
use shrimply_video_modifiers::wave_ripple::WaveRippleModifier;

pub(super) fn presentation(
    value: &WaveRippleModifier,
    index: usize,
    runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let scalar = |field, label, timeline, minimum, unit, rotating| {
        super::modifier_scalar_control(
            format!("{base}/{field}"),
            label,
            timeline,
            runtime,
            NumberSpec {
                minimum,
                maximum: 1_000_000.0,
                drag_step: if rotating { 1.0 } else { 0.01 },
                digits: 2,
                unit,
            },
            rotating,
        )
    };
    let mut section = InspectorSection::default();
    section.add(scalar(
        "amplitude",
        "Amplitude",
        &value.amplitude,
        -1_000_000.0,
        "px",
        false,
    ));
    section.add(scalar(
        "wavelength",
        "Wavelength",
        &value.wavelength,
        0.0,
        "px",
        false,
    ));
    section.add(scalar(
        "angle_degrees",
        "Angle",
        &value.angle_degrees,
        -1_000_000.0,
        "deg",
        true,
    ));
    section.add(scalar(
        "phase",
        "Phase",
        &value.phase,
        -1_000_000.0,
        "deg",
        true,
    ));
    section
}
