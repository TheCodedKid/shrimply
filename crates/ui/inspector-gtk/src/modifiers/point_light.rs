use gtk::prelude::*;
use shrimply_core::modifier_model::ModifierModel;
use shrimply_inspector_core::{
    ControlKind, InspectorControl, InspectorTarget, NumberSpec, VisualModifierBodyPresentation,
    visual_modifier_presentations,
};
use shrimply_video_modifiers::scene_3d::PointLightModifier;
use uuid::Uuid;

use super::{InspectorContext, ScalarOptions, color_row, scalar_row, vec3_row};

pub fn add_rows(value: &PointLightModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    let key = context
        .selected_item
        .clone()
        .expect("point light inspector must have a selected item");
    let target = InspectorTarget::Item(key.clone());
    let snapshot = context.inspector_core.snapshot();
    assert_eq!(
        snapshot.target, target,
        "point light inspector target changed"
    );
    let project = context.project.borrow();
    let item = project
        .video_item(&key)
        .expect("point light inspector item must still be available");
    let section = visual_modifier_presentations(&project, &key, item, snapshot.runtime)
        .into_iter()
        .find(|modifier| modifier.id == id)
        .and_then(|modifier| match modifier.body {
            Some(VisualModifierBodyPresentation::PointLight(section)) => Some(section),
            _ => None,
        })
        .expect("point light modifier must still be available");
    drop(project);

    for control in section
        .controls
        .into_iter()
        .filter(|control| control.visible)
    {
        assert_eq!(
            control.target_id,
            Some(id),
            "point light control modifier changed"
        );
        let widget = match control.kind {
            ControlKind::LayeredVector3 => vector_row(value, id, &control, context),
            ControlKind::LayeredColor => color_control(value, id, &control, context),
            ControlKind::LayeredNumber => number_row(value, id, &control, context),
            kind => panic!("unsupported shared point light control: {kind:?}"),
        };
        out.append(&widget);
    }
}

fn vector_row(
    value: &PointLightModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.timeline_id, Some(value.position.id));
    assert_eq!(control.number.drag_step, 0.1);
    assert_eq!(control.number.digits, 2);
    assert_eq!(
        control.number,
        NumberSpec {
            drag_step: 0.1,
            digits: 2,
            ..NumberSpec::default()
        },
    );
    assert_eq!(control.width_characters, 5);
    assert_eq!(control.prefixes, ["X", "Y", "Z"]);
    assert_eq!(control.commit_name, "visual-modifier-vector");
    assert!(!control.lock);
    assert!(!control.prefix_icon_rotates);
    let widget = vec3_row(&control.label, &value.position, id, false, context);
    widget.set_sensitive(control.sensitive);
    widget
}

fn color_control(
    value: &PointLightModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.timeline_id, Some(value.color.id));
    assert_eq!(control.components.len(), 4);
    assert_eq!(control.commit_name, "visual-modifier-color");
    let widget = color_row(&control.label, &value.color, id, context);
    widget.set_sensitive(control.sensitive);
    widget
}

fn number_row(
    value: &PointLightModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = control
        .timeline_id
        .expect("point light number timeline ID is missing");
    let timeline = value
        .number(timeline_id)
        .expect("point light number timeline changed");
    let defaults = NumberSpec::default();
    assert_eq!(control.number.drag_step, 0.01);
    assert_eq!(control.number.digits, 2);
    assert_eq!(control.number.maximum, defaults.maximum);
    assert_eq!(control.number.unit, "");
    assert_eq!(control.width_characters, 8);
    assert_eq!(control.commit_name, "visual-modifier-value");
    assert!(!control.integer);
    assert!(!control.lock);
    assert!(!control.prefix_icon_rotates);
    let widget = scalar_row(
        &control.label,
        timeline,
        id,
        ScalarOptions {
            minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
            maximum: None,
            unit: None,
            rotating: false,
        },
        context,
    );
    widget.set_sensitive(control.sensitive);
    widget
}
