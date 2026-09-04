use gtk::prelude::*;
use shrimply_core::modifier_model::ModifierModel;
use shrimply_gtk_components::{
    tr,
    ui::{StringChoice, labeled_string_selector},
};
use shrimply_inspector_core::{ControlKind, InspectorControl, InspectorTarget, NumberSpec};
use shrimply_shape_3d::Shape3dModifier;
use uuid::Uuid;

use crate::InspectorContext;

use super::{ScalarOptions, color_row, integer_scalar_row, scalar_row, vec3_row, vec3_scale_row};

pub fn add_rows(value: &Shape3dModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    let key = context
        .selected_item
        .clone()
        .expect("3D shape inspector must have a selected item");
    let target = InspectorTarget::Item(key);
    let section = context
        .inspector_core
        .shape_3d_presentation(&target, id)
        .expect("3D shape modifier must still be available");

    for control in section
        .controls
        .into_iter()
        .filter(|control| control.visible)
    {
        assert_eq!(control.target_id, Some(id), "3D shape control changed");
        let widget = match control.kind {
            ControlKind::Selector => selector_row(id, &target, control, context),
            ControlKind::LayeredNumber => number_row(value, id, &control, context),
            ControlKind::LayeredVector3 => vector_row(value, id, &control, context),
            ControlKind::LayeredColor => color_control(value, id, &control, context),
            kind => panic!("unsupported shared 3D shape control: {kind:?}"),
        };
        out.append(&widget);
    }
}

fn selector_row(
    id: Uuid,
    target: &InspectorTarget,
    control: InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.values.len(), control.labels.len());
    assert!(control.commit_immediately);
    assert!(!control.commit_name.is_empty());
    let choices = control
        .values
        .iter()
        .cloned()
        .zip(control.labels.iter())
        .map(|(value, label)| StringChoice {
            value,
            label: tr!(label).into_owned(),
        })
        .collect();
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let selector = labeled_string_selector(&control.label, &control.value, choices, move |value| {
        let result = controller
            .ensure_visual_modifier(&target, &path, id)
            .and_then(|()| controller.set_video_field(&target, &path, &value, &commit_name, true));
        if let Err(error) = result {
            tracing::error!(%error, "Could not update GTK 3D shape selector");
        }
    });
    selector.widget().set_sensitive(control.sensitive);
    selector.widget().clone()
}

fn number_row(
    value: &Shape3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = control
        .timeline_id
        .expect("3D shape number timeline ID is missing");
    let timeline = value
        .number(timeline_id)
        .expect("3D shape number timeline changed");
    let defaults = NumberSpec::default();
    assert_eq!(control.width_characters, 8);
    assert_eq!(control.commit_name, "visual-modifier-value");
    assert!(!control.lock);
    assert!(!control.prefix_icon_rotates);
    let options = ScalarOptions {
        minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
        maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
        unit: (!control.number.unit.is_empty()).then_some(control.number.unit),
        rotating: false,
    };
    let widget = if control.integer {
        assert_eq!(control.number.drag_step, 1.0);
        assert_eq!(control.number.digits, 0);
        integer_scalar_row(&control.label, timeline, id, options, context)
    } else {
        assert_eq!(control.number.drag_step, 0.01);
        assert_eq!(control.number.digits, 2);
        scalar_row(&control.label, timeline, id, options, context)
    };
    widget.set_sensitive(control.sensitive);
    widget
}

fn vector_row(
    value: &Shape3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = control
        .timeline_id
        .expect("3D shape vector timeline ID is missing");
    let timeline = value
        .number3(timeline_id)
        .expect("3D shape vector timeline changed");
    let defaults = NumberSpec::default();
    assert_eq!(control.number.digits, 2);
    assert_eq!(control.width_characters, 5);
    assert_eq!(control.prefixes, ["X", "Y", "Z"]);
    assert_eq!(control.number.maximum, defaults.maximum);
    assert_eq!(control.commit_name, "visual-modifier-vector");
    let degrees = control.number.unit == "°";
    assert_eq!(control.number.drag_step, if degrees { 1.0 } else { 0.1 });
    let widget = if control.lock {
        assert!(!degrees);
        assert_eq!(control.number.minimum, 0.0);
        vec3_scale_row(&control.label, timeline, id, context)
    } else {
        assert_eq!(control.number.minimum, defaults.minimum);
        vec3_row(&control.label, timeline, id, degrees, context)
    };
    widget.set_sensitive(control.sensitive);
    widget
}

fn color_control(
    value: &Shape3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.timeline_id, Some(value.material.base_color.id));
    assert_eq!(control.components.len(), 4);
    assert_eq!(control.commit_name, "visual-modifier-color");
    let widget = color_row(&control.label, &value.material.base_color, id, context);
    widget.set_sensitive(control.sensitive);
    widget
}
