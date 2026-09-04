use gtk::prelude::*;
use shrimply_inspector_core::{ControlKind, InspectorControl, InspectorTarget, NumberSpec};
use shrimply_video_modifiers::scene_3d::SunLightModifier;
use uuid::Uuid;

use super::{InspectorContext, color_row, shared_scalar_row, vec3_row};

pub fn add_rows(
    value: &SunLightModifier,
    out: &gtk::Box,
    modifier_id: Uuid,
    context: &InspectorContext,
) {
    let key = context
        .selected_item
        .clone()
        .expect("Sun Light inspector must have a selected item");
    let section = context
        .inspector_core
        .sun_light_presentation(&InspectorTarget::Item(key), modifier_id)
        .expect("live Sun Light modifier must have a shared presentation");
    let [rotation, color, intensity, angular_radius]: [_; 4] = section
        .controls
        .try_into()
        .expect("Sun Light presentation must contain exactly four controls");

    let rotation_row = rotation_row(value, modifier_id, &rotation, context);
    rotation_row.set_sensitive(rotation.sensitive);
    if rotation.visible {
        out.append(&rotation_row);
    }

    let color_row = shared_color_row(value, modifier_id, &color, context);
    color_row.set_sensitive(color.sensitive);
    if color.visible {
        out.append(&color_row);
    }

    for (control, field, timeline) in [
        (&intensity, "intensity", &value.intensity),
        (
            &angular_radius,
            "angular_radius_degrees",
            &value.angular_radius_degrees,
        ),
    ] {
        let row = shared_scalar_row(control, field, timeline, modifier_id, context);
        row.set_sensitive(control.sensitive);
        if control.visible {
            out.append(&row);
        }
    }
}

fn rotation_row(
    value: &SunLightModifier,
    modifier_id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::LayeredVector3);
    assert!(
        control
            .path
            .ends_with("/effect/effect/config/rotation_degrees")
    );
    assert_eq!(control.target_id, Some(modifier_id));
    assert_eq!(control.timeline_id, Some(value.rotation_degrees.id));
    assert_eq!(control.commit_name, "visual-modifier-vector");
    assert_eq!(control.width_characters, 5);
    assert_eq!(control.prefixes, ["X", "Y", "Z"]);
    assert_eq!(
        control.number,
        NumberSpec {
            drag_step: 1.0,
            digits: 2,
            unit: "°",
            ..NumberSpec::default()
        }
    );
    assert!(!control.lock);
    assert!(control.prefix_icon_rotates);
    vec3_row(
        &control.label,
        &value.rotation_degrees,
        modifier_id,
        true,
        context,
    )
}

fn shared_color_row(
    value: &SunLightModifier,
    modifier_id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::LayeredColor);
    assert!(control.path.ends_with("/effect/effect/config/color"));
    assert_eq!(control.target_id, Some(modifier_id));
    assert_eq!(control.timeline_id, Some(value.color.id));
    assert_eq!(control.commit_name, "visual-modifier-color");
    assert_eq!(control.components.len(), 4);
    color_row(&control.label, &value.color, modifier_id, context)
}
