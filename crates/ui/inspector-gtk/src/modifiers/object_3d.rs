use adw::prelude::AdwDialogExt;
use gtk::prelude::*;
use shrimply_core::modifier_model::ModifierModel;
use shrimply_gtk_components::{
    tr,
    ui::{I18nFileFilterExt, StringChoice, labeled_string_selector},
};
use shrimply_inspector_core::{
    ControlKind, InspectorControl, InspectorControlAction, InspectorSection, InspectorTarget,
    NumberSpec, VisualModifierBodyPresentation, visual_modifier_presentations,
};
use shrimply_video_modifiers::scene_3d::Object3dModifier;
use uuid::Uuid;

use crate::InspectorContext;

use super::{ScalarOptions, color_row, scalar_row, vec3_row, vec3_scale_row};

pub fn add_rows(value: &Object3dModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    let key = context
        .selected_item
        .clone()
        .expect("3D object inspector must have a selected item");
    let target = InspectorTarget::Item(key.clone());
    let snapshot = context.inspector_core.snapshot();
    assert_eq!(
        snapshot.target, target,
        "3D object inspector target changed"
    );
    let project = context.project.borrow();
    let item = project
        .video_item(&key)
        .expect("3D object inspector item must still be available");
    let section = visual_modifier_presentations(&project, &key, item, snapshot.runtime)
        .into_iter()
        .find(|modifier| modifier.id == id)
        .and_then(|modifier| match modifier.body {
            Some(VisualModifierBodyPresentation::Object3d(section)) => Some(section),
            _ => None,
        })
        .expect("3D object modifier must still be available");
    drop(project);

    add_shared_rows(value, out, id, &target, section, context);
}

fn add_shared_rows(
    value: &Object3dModifier,
    out: &gtk::Box,
    id: Uuid,
    target: &InspectorTarget,
    section: InspectorSection,
    context: &InspectorContext,
) {
    let mut controls = section.controls.into_iter();
    let model = controls.next().expect("3D object model control is missing");
    let select = controls
        .next()
        .expect("3D object select-model action is missing");
    let clear = controls
        .next()
        .expect("3D object clear-model action is missing");
    out.append(&file_row(model, select, clear, id, target, context));

    for control in controls.filter(|control| control.visible) {
        assert_eq!(
            control.target_id,
            Some(id),
            "3D object control modifier changed",
        );
        let widget = match control.kind {
            ControlKind::LayeredVector3 => vector_row(value, id, &control, context),
            ControlKind::LayeredColor => color_control(value, id, &control, context),
            ControlKind::LayeredNumber => number_row(value, id, &control, context),
            ControlKind::Selector => selector_row(id, target, control, context),
            kind => panic!("unsupported shared 3D object control: {kind:?}"),
        };
        out.append(&widget);
    }
}

fn vector_row(
    value: &Object3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = control
        .timeline_id
        .expect("3D object vector timeline ID is missing");
    let timeline = value
        .number3(timeline_id)
        .expect("3D object vector timeline changed");
    let defaults = NumberSpec::default();
    assert_eq!(control.number.digits, 2);
    assert_eq!(control.width_characters, 5);
    assert_eq!(control.prefixes, ["X", "Y", "Z"]);
    assert_eq!(control.number.maximum, defaults.maximum);
    assert_eq!(control.commit_name, "edit-scene-3d-vec3");
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
    value: &Object3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(
        control.timeline_id,
        Some(value.material.base_color.id),
        "3D object color timeline changed",
    );
    assert_eq!(control.components.len(), 4);
    assert_eq!(control.commit_name, "visual-modifier-color");
    let widget = color_row(&control.label, &value.material.base_color, id, context);
    widget.set_sensitive(control.sensitive);
    widget
}

fn number_row(
    value: &Object3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = control
        .timeline_id
        .expect("3D object number timeline ID is missing");
    let timeline = value
        .number(timeline_id)
        .expect("3D object number timeline changed");
    let defaults = NumberSpec::default();
    assert_eq!(control.number.drag_step, 0.01);
    assert_eq!(control.number.digits, 2);
    assert_eq!(control.width_characters, 8);
    assert_eq!(control.commit_name, "visual-modifier-value");
    assert!(!control.integer);
    assert!(!control.lock);
    let widget = scalar_row(
        &control.label,
        timeline,
        id,
        ScalarOptions {
            minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
            maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
            unit: (!control.number.unit.is_empty()).then_some(control.number.unit),
            rotating: control.prefix_icon_rotates,
        },
        context,
    );
    widget.set_sensitive(control.sensitive);
    widget
}

fn selector_row(
    id: Uuid,
    target: &InspectorTarget,
    control: InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.values.len(), control.labels.len());
    assert_eq!(control.commit_name, "edit-3d-object-normals");
    assert!(control.commit_immediately);
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
            tracing::error!(%error, "Could not update GTK 3D object normals");
        }
    });
    selector.widget().set_sensitive(control.sensitive);
    selector.widget().clone()
}

fn file_row(
    model: InspectorControl,
    select: InspectorControl,
    clear: InspectorControl,
    id: Uuid,
    target: &InspectorTarget,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(model.kind, ControlKind::ReadOnly);
    assert_eq!(select.kind, ControlKind::Action);
    assert_eq!(clear.kind, ControlKind::Action);
    assert_eq!(model.target_id, Some(id));
    assert_eq!(select.target_id, Some(id));
    assert_eq!(clear.target_id, Some(id));
    assert_eq!(
        select.action,
        Some(InspectorControlAction::SelectObject3dModel { modifier_id: id }),
    );
    assert_eq!(
        clear.action,
        Some(InspectorControlAction::ClearObject3dModel { modifier_id: id }),
    );

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let filename = gtk::Label::builder()
        .label(&model.value)
        .halign(gtk::Align::Start)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .css_classes(["dim-label"])
        .build();
    filename.set_tooltip_text((!model.tooltip.is_empty()).then_some(model.tooltip.as_str()));
    row.append(&filename);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    actions.set_halign(gtk::Align::End);
    actions.add_css_class("linked");
    let choose_content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    choose_content.append(&gtk::Image::from_icon_name(&select.prefix_icon));
    choose_content.append(&gtk::Label::new(Some(&select.value)));
    let choose = gtk::Button::builder()
        .child(&choose_content)
        .sensitive(select.sensitive)
        .build();
    choose.set_tooltip_text((!select.tooltip.is_empty()).then_some(select.tooltip.as_str()));
    let clear_button = gtk::Button::builder()
        .icon_name(&clear.prefix_icon)
        .tooltip_text(tr!(&clear.tooltip).as_ref())
        .sensitive(clear.sensitive)
        .build();
    actions.append(&choose);
    actions.append(&clear_button);
    row.append(&actions);

    let choose_target = target.clone();
    let controller = context.inspector_core.clone();
    choose.connect_clicked(move |_| {
        let label = "Select 3D model";
        let filter = gtk::FileFilter::new();
        filter.set_name_i18n("3D models");
        filter.add_pattern("*.obj");
        filter.add_pattern("*.glb");
        let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder()
            .title(tr!(label).as_ref())
            .filters(&filters)
            .build();
        let target = choose_target.clone();
        let controller = controller.clone();
        shrimply_gtk_components::file_picker::open(
            label,
            &dialog,
            None::<&gtk::Window>,
            move |result| {
                let Ok(file) = result else { return };
                let Some(path) = file.path() else { return };
                if let Err(error) = controller.set_object_3d_model(&target, id, &path) {
                    adw::AlertDialog::new(Some("Could not use 3D model"), Some(&error))
                        .present(None::<&gtk::Widget>);
                }
            },
        );
    });
    let controller = context.inspector_core.clone();
    let clear_target = target.clone();
    let action = clear.action.expect("3D object clear action is missing");
    clear_button.connect_clicked(move |_| {
        if let Err(error) = controller.trigger_video_control_action(&clear_target, action) {
            tracing::error!(%error, "Could not clear GTK 3D object model");
        }
    });
    crate::ui::control_row(&model.label, &row)
}
