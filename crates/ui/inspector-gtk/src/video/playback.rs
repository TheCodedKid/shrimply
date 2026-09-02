use gtk::prelude::*;
use shrimply_gtk_components::{
    tr,
    ui::{StringChoice, labeled_string_selector, switch_row},
};
use shrimply_inspector_core::{ControlKind, InspectorControl, InspectorTarget, VideoCard};
use shrimply_project::project::VideoItem;

use crate::ui::NumberPicker;

use super::super::{
    InspectorContext,
    item::{DefaultInspectorItem, InspectorListItem},
    section::InspectorSection,
};

pub(super) fn speed_item(item: &VideoItem) -> InspectorListItem {
    shared_item(shrimply_inspector_core::video::playback::speed(item))
}

pub(super) fn frame_rate_item(item: &VideoItem) -> InspectorListItem {
    shared_item(shrimply_inspector_core::video::playback::frame_rate(item))
}

pub(super) fn motion_blur_item(item: &VideoItem) -> InspectorListItem {
    shared_item(shrimply_inspector_core::video::playback::motion_blur(item))
}

pub(super) fn repeat_item(item: &VideoItem) -> InspectorListItem {
    shared_item(shrimply_inspector_core::video::playback::repeat(item))
}

fn shared_item(card: VideoCard) -> InspectorListItem {
    let default = card.clone();
    DefaultInspectorItem::new_with_default(
        card.key,
        card.title,
        card,
        controls,
        move |_| default.clone(),
        |context, card| {
            let Some(reset) = &card.reset else {
                return;
            };
            let Some(item) = context.selected_item.clone() else {
                return;
            };
            if let Err(error) = context
                .inspector_core
                .reset_video(&InspectorTarget::Item(item), reset)
            {
                tracing::error!(%error, "Could not reset GTK video playback card");
            }
        },
    )
    .boxed()
}

fn controls(card: &VideoCard, context: &InspectorContext) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    let Some(item) = context.selected_item.clone() else {
        return vec![section.into_widget()];
    };
    let target = InspectorTarget::Item(item);
    for control in card.section.controls.iter().cloned() {
        match control.kind {
            ControlKind::Fraction => add_fraction(&section, context, &target, control),
            ControlKind::Selector => add_selector(&section, context, &target, control),
            ControlKind::Boolean => add_boolean(&section, context, &target, control),
            ControlKind::Number => add_number(&section, context, &target, control),
            _ => panic!("unsupported GTK video playback control kind"),
        }
    }
    vec![section.into_widget()]
}

fn add_fraction(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: InspectorControl,
) {
    let [numerator, denominator] = control.components.as_slice() else {
        panic!("video playback fraction must have a numerator and denominator");
    };
    let value = shrimply_math_core::fraction_new(
        numerator
            .parse()
            .expect("video playback numerator must be an integer"),
        denominator
            .parse()
            .expect("video playback denominator must be an integer"),
    );
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let commit_controller = controller.clone();
    let commit_target = target.clone();
    let commit_name_copy = commit_name.clone();
    let mut picker = NumberPicker::fraction_builder(value)
        .minimum(control.number.minimum)
        .maximum(control.number.maximum)
        .drag_step(control.number.drag_step)
        .digits(
            usize::try_from(control.number.digits)
                .expect("video playback digits must be nonnegative"),
        )
        .width_chars(control.width_characters);
    if !control.number.unit.is_empty() {
        picker = picker.unit_name(control.number.unit);
    }
    let picker = picker
        .on_change_fraction(move |value| {
            if let Err(error) = controller.set_video_fraction(&target, &path, value, &commit_name) {
                tracing::error!(%error, "Could not change GTK video playback fraction");
            }
        })
        .on_commit(move |_| {
            if let Err(error) =
                commit_controller.commit_video_field(&commit_target, &commit_name_copy)
            {
                tracing::error!(%error, "Could not commit GTK video playback fraction");
            }
        })
        .build();
    section.add_control_row(&control.label, &picker);
}

fn add_selector(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: InspectorControl,
) {
    assert_eq!(
        control.values.len(),
        control.labels.len(),
        "video playback selector values must have labels",
    );
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
    let commit_immediately = control.commit_immediately;
    let selector = labeled_string_selector(&control.label, &control.value, choices, move |value| {
        if let Err(error) =
            controller.set_video_field(&target, &path, &value, &commit_name, commit_immediately)
        {
            tracing::error!(%error, "Could not change GTK video playback selector");
        }
    });
    selector.widget().set_sensitive(control.sensitive);
    section.add_wide_control(selector.widget());
}

fn add_boolean(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: InspectorControl,
) {
    let active = control
        .value
        .parse::<bool>()
        .expect("video playback boolean must be true or false");
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let commit_immediately = control.commit_immediately;
    let toggle = switch_row(&control.label, None, active, move |active| {
        if let Err(error) = controller.set_video_field(
            &target,
            &path,
            &active.to_string(),
            &commit_name,
            commit_immediately,
        ) {
            tracing::error!(%error, "Could not change GTK video playback toggle");
        }
    });
    toggle.set_sensitive(control.sensitive);
    section.add_wide_control(&toggle);
}

fn add_number(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: InspectorControl,
) {
    let value = control
        .value
        .parse::<f64>()
        .expect("video playback number must be numeric");
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let commit_immediately = control.commit_immediately;
    let mut picker = NumberPicker::builder(value)
        .minimum(control.number.minimum)
        .maximum(control.number.maximum)
        .drag_step(control.number.drag_step)
        .digits(
            usize::try_from(control.number.digits)
                .expect("video playback digits must be nonnegative"),
        );
    if !control.number.unit.is_empty() {
        picker = picker.unit_name(control.number.unit);
    }
    let picker = picker
        .on_change(move |value| {
            if let Err(error) = controller.set_video_field(
                &target,
                &path,
                &value.to_string(),
                &commit_name,
                commit_immediately,
            ) {
                tracing::error!(%error, "Could not change GTK video playback number");
            }
        })
        .build();
    picker.set_sensitive(control.sensitive);
    section.add_control_row(&control.label, &picker);
}
