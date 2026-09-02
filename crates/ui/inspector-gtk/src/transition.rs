use shrimply_gtk_components::{
    tr,
    ui::{ColorPicker, StringChoice, labeled_string_selector, switch_row},
};
use shrimply_inspector_core::{
    ControlKind, InspectorControl, InspectorTarget, TransitionPresentation,
};
use shrimply_project::project::{Color, ItemAddress, Project, TransitionSide};

use crate::ui::NumberPicker;

use super::{Inspectable, InspectorContext, section::InspectorSection};

pub(super) struct TransitionInspection(TransitionPresentation);

pub(super) fn resolve(
    project: &Project,
    item: &ItemAddress,
    side: TransitionSide,
) -> Option<TransitionInspection> {
    shrimply_inspector_core::transition::presentation(project, item, side).map(TransitionInspection)
}

impl Inspectable for TransitionInspection {
    fn title(&self) -> &'static str {
        self.0.title
    }

    fn add_rows(&self, section: &InspectorSection, context: &InspectorContext) {
        let Some(item) = context.selected_item.clone() else {
            return;
        };
        let target = InspectorTarget::Transition {
            item,
            side: self.0.side,
        };
        for control in self.0.section().controls {
            add_control(section, context, &target, control);
        }
    }
}

fn add_control(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: InspectorControl,
) {
    match control.kind {
        ControlKind::Selector => add_selector(section, context, target, control),
        ControlKind::Number => add_number(section, context, target, control),
        ControlKind::Boolean => add_boolean(section, context, target, control),
        ControlKind::Color => add_color(section, context, target, control),
        _ => panic!("unsupported GTK transition control kind"),
    }
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
        "transition selector values must have labels",
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
        if let Err(error) = controller.set_transition_field(
            &target,
            &path,
            &value,
            &commit_name,
            commit_immediately,
        ) {
            tracing::error!(%error, "Could not change GTK transition selector");
        }
    });
    section.add_wide_control(selector.widget());
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
        .expect("transition number presentation must be numeric");
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let store_multiplier = control.store_multiplier;
    let mut picker = NumberPicker::builder(value)
        .minimum(control.number.minimum)
        .maximum(control.number.maximum)
        .drag_step(control.number.drag_step)
        .digits(
            usize::try_from(control.number.digits)
                .expect("transition number digits must be nonnegative"),
        );
    if !control.number.unit.is_empty() {
        picker = picker.unit_name(control.number.unit);
    }
    let commit_controller = controller.clone();
    let commit_target = target.clone();
    let commit_name_copy = commit_name.clone();
    let picker = picker
        .on_change(move |value| {
            let value = value * store_multiplier;
            if let Err(error) = controller.set_transition_field(
                &target,
                &path,
                &value.to_string(),
                &commit_name,
                false,
            ) {
                tracing::error!(%error, "Could not change GTK transition number");
            }
        })
        .on_commit(move |_| {
            if let Err(error) =
                commit_controller.commit_transition_field(&commit_target, &commit_name_copy)
            {
                tracing::error!(%error, "Could not commit GTK transition number");
            }
        })
        .build();
    section.add_control_row(&control.label, &picker);
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
        .expect("transition boolean presentation must be true or false");
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let commit_immediately = control.commit_immediately;
    let subtitle = (!control.subtitle.is_empty()).then_some(control.subtitle.as_str());
    let toggle = switch_row(&control.label, subtitle, active, move |active| {
        if let Err(error) = controller.set_transition_field(
            &target,
            &path,
            &active.to_string(),
            &commit_name,
            commit_immediately,
        ) {
            tracing::error!(%error, "Could not change GTK transition toggle");
        }
    });
    section.add_wide_control(&toggle);
}

fn add_color(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: InspectorControl,
) {
    let [red, green, blue, alpha] = control.components.as_slice() else {
        panic!("transition color presentation must have four components");
    };
    let color = Color::from_rgba(
        component(red),
        component(green),
        component(blue),
        component(alpha),
    );
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let commit_immediately = control.commit_immediately;
    let title = if control.tooltip.is_empty() {
        control.label.clone()
    } else {
        control.tooltip.clone()
    };
    let picker = ColorPicker::builder(color)
        .title(tr!(&title).as_ref())
        .with_alpha(control.with_alpha)
        .hexpand(true)
        .on_change(move |color| {
            let values = color
                .to_array()
                .into_iter()
                .enumerate()
                .map(|(index, value)| (index, value.to_string()))
                .collect::<Vec<_>>();
            if let Err(error) = controller.set_transition_components(
                &target,
                &path,
                &values,
                &commit_name,
                commit_immediately,
            ) {
                tracing::error!(%error, "Could not change GTK transition color");
            }
        })
        .build();
    section.add_control_row(&control.label, &picker);
}

fn component(value: &str) -> u8 {
    value
        .parse()
        .expect("transition color component must be an integer")
}
