use shrimply_gtk_components::ui::{ColorPicker, NumberPicker, control_row, switch_row};
use shrimply_inspector_core::{
    ControlKind, InspectorControl, InspectorTarget, VideoCard,
    manim_parameters::{self as shared, ManimParametersReset},
};
use shrimply_project::project::{Color, ManimParameterValue};

use super::super::{
    InspectorContext,
    item::{DefaultInspectorItem, InspectorListItem},
    section::InspectorSection,
};

pub(super) fn item(card: VideoCard, reset: ManimParametersReset) -> InspectorListItem {
    let default = card.clone();
    DefaultInspectorItem::new_with_default(
        card.key,
        card.title,
        card,
        controls,
        move |_| default.clone(),
        move |context, _| {
            let Some(item) = context.selected_item.clone() else {
                return;
            };
            if let Err(error) = context
                .inspector_core
                .reset_manim_parameters(&InspectorTarget::Item(item), &reset)
            {
                tracing::error!(%error, "Could not reset GTK Manim parameters");
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
    for control in &card.section.controls {
        match control.kind {
            ControlKind::Number => add_number(&section, context, &target, control),
            ControlKind::Fraction => add_fraction(&section, context, &target, control),
            ControlKind::Color => add_color(&section, context, &target, control),
            ControlKind::Selector => add_selector(&section, context, &target, control),
            ControlKind::Boolean => add_boolean(&section, context, &target, control),
            ControlKind::Text => add_text(&section, context, &target, control),
            _ => panic!("unsupported shared Manim parameter control kind"),
        }
    }
    vec![section.into_widget()]
}

fn add_number(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: &InspectorControl,
) {
    let key = parameter_key(control);
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let commit_name = control.commit_name.clone();
    let value = control
        .value
        .parse::<f64>()
        .expect("shared Manim number must be numeric");
    let mut picker = NumberPicker::builder(value)
        .minimum(control.number.minimum)
        .maximum(control.number.maximum)
        .drag_step(control.number.drag_step)
        .digits(
            usize::try_from(control.number.digits)
                .expect("shared Manim number digits must be nonnegative"),
        );
    picker = if control.integer {
        picker.on_commit_integer(move |value: i64| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::Integer(value),
                &commit_name,
            );
        })
    } else {
        picker.on_commit(move |value| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::Float(value),
                &commit_name,
            );
        })
    };
    section.add_wide_control(&control_row(&control.label, &picker.build()));
}

fn add_fraction(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: &InspectorControl,
) {
    let [numerator, denominator] = control.components.as_slice() else {
        panic!("shared Manim fraction must have numerator and denominator")
    };
    let value = shrimply_math_core::fraction_new(
        numerator
            .parse()
            .expect("shared Manim fraction numerator must be an integer"),
        denominator
            .parse()
            .expect("shared Manim fraction denominator must be an integer"),
    );
    let key = parameter_key(control);
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let commit_name = control.commit_name.clone();
    let picker = NumberPicker::fraction_builder(value)
        .minimum(control.number.minimum)
        .maximum(control.number.maximum)
        .drag_step(control.number.drag_step)
        .digits(
            usize::try_from(control.number.digits)
                .expect("shared Manim fraction digits must be nonnegative"),
        )
        .on_commit_fraction(move |value| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::Fraction {
                    numerator: shrimply_math_core::fraction_numerator(value),
                    denominator: shrimply_math_core::fraction_denominator(value),
                },
                &commit_name,
            );
        })
        .build();
    section.add_wide_control(&control_row(&control.label, &picker));
}

fn add_color(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: &InspectorControl,
) {
    let [red, green, blue, alpha] = control.components.as_slice() else {
        panic!("shared Manim color must have four channels")
    };
    let color = Color::new(channel(red), channel(green), channel(blue), channel(alpha));
    let key = parameter_key(control);
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let commit_name = control.commit_name.clone();
    let picker = ColorPicker::builder(color)
        .title(&control.label)
        .with_alpha(control.with_alpha)
        .hexpand(true)
        .on_change(move |value| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::Color(value),
                &commit_name,
            );
        })
        .build();
    section.add_wide_control(&control_row(&control.label, &picker));
}

fn add_selector(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: &InspectorControl,
) {
    assert_eq!(
        control.values.len(),
        control.labels.len(),
        "shared Manim selector values must have labels",
    );
    let key = parameter_key(control);
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let commit_name = control.commit_name.clone();
    let selector = shrimply_gtk_components::ui::string_selector(
        &control.label,
        &control.value,
        control.values.clone(),
        move |value| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::Option(value),
                &commit_name,
            );
        },
    );
    section.add_wide_control(selector.widget());
}

fn add_boolean(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: &InspectorControl,
) {
    let key = parameter_key(control);
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let commit_name = control.commit_name.clone();
    section.add_wide_control(&switch_row(
        &control.label,
        None,
        control
            .value
            .parse()
            .expect("shared Manim boolean must be true or false"),
        move |value| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::Boolean(value),
                &commit_name,
            );
        },
    ));
}

fn add_text(
    section: &InspectorSection,
    context: &InspectorContext,
    target: &InspectorTarget,
    control: &InspectorControl,
) {
    let key = parameter_key(control);
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let commit_name = control.commit_name.clone();
    let input = shrimply_gtk_components::ui::SingleLineTextInput::builder(&control.value)
        .on_commit(move |value| {
            set_parameter(
                &controller,
                &target,
                &key,
                ManimParameterValue::String(value),
                &commit_name,
            );
        })
        .build();
    section.add_wide_control(&control_row(&control.label, &input));
}

fn set_parameter(
    controller: &shrimply_inspector_core::InspectorController,
    target: &InspectorTarget,
    key: &str,
    value: ManimParameterValue,
    commit_name: &str,
) {
    if let Err(error) = controller.set_manim_parameter(target, key, value, commit_name) {
        tracing::error!(%error, "Could not update GTK Manim parameter");
    }
}

fn parameter_key(control: &InspectorControl) -> String {
    shared::parameter_key(&control.path)
        .expect("shared Manim parameter control must have a parameter path")
        .to_string()
}

fn channel(value: &str) -> u8 {
    value
        .parse()
        .expect("shared Manim color channel must be an integer")
}
