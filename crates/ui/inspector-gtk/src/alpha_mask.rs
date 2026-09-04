use std::rc::Rc;

use gtk::prelude::*;
use shrimply_gtk_components::ui::switch_row;
use shrimply_inspector_core::{ControlKind, InspectorControl, InspectorTarget, NumberSpec};
use shrimply_project::project::{AlphaMaskShape, VisualAlphaMaskTarget};

use crate::{
    InspectorContext,
    item::HeaderButtonToggle,
    player_state::ProjectChange,
    preview_focus::{self, FocusedPreview},
    timeline_value::{
        scalar::{ScalarAccess, ScalarSpec, ScalarTarget, scalar_control},
        vector::vec2::{VecAccess, VecSpec, VecTarget, vec_control},
    },
};

pub(crate) fn button_toggle(
    mask_target: VisualAlphaMaskTarget,
    context: &InspectorContext,
) -> HeaderButtonToggle {
    let active = context.selected_item.as_ref().is_some_and(|key| {
        context
            .inspector_core
            .alpha_mask_presentation(&InspectorTarget::Item(key.clone()), mask_target)
            .is_ok_and(|presentation| presentation.active)
    });
    let context = context.detached();
    HeaderButtonToggle {
        icon: "select-symbolic",
        active,
        tooltip: "Mask",
        activate: Rc::new(move |active| set_open(&context, mask_target, active)),
    }
}

pub(crate) fn widget(
    mask_target: VisualAlphaMaskTarget,
    context: &InspectorContext,
) -> gtk::Widget {
    let out = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let Some(key) = context.selected_item.clone() else {
        return out.upcast();
    };
    let target = InspectorTarget::Item(key.clone());
    let Ok(presentation) = context
        .inspector_core
        .alpha_mask_presentation(&target, mask_target)
    else {
        return out.upcast();
    };
    if !presentation.active {
        return out.upcast();
    }
    let mask = context
        .project
        .borrow()
        .video_item(&key)
        .and_then(|item| item.alpha_mask(mask_target))
        .cloned()
        .expect("active alpha-mask presentation must have a live mask");

    out.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let click = gtk::GestureClick::new();
    click.set_button(0);
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    click.connect_pressed({
        let context = context.detached();
        move |_, _, _, _| focus(&context, mask_target, true)
    });
    out.add_controller(click);

    let mut controls = presentation.section.controls.into_iter();
    let shape = controls
        .next()
        .expect("alpha-mask shape control is missing");
    assert_eq!(shape.kind, ControlKind::Selector);
    let shape_choices = shrimply_inspector_core::alpha_mask::SHAPE_CHOICES;
    assert_eq!(
        shape.value,
        shape_choices
            .iter()
            .find_map(|(candidate, value, _)| (*candidate == mask.shape).then_some(*value))
            .expect("current alpha-mask shape must be a declared choice")
    );
    assert_eq!(
        shape.values,
        shape_choices
            .iter()
            .map(|(_, value, _)| (*value).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        shape.labels,
        shape_choices
            .iter()
            .map(|(_, _, label)| (*label).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        shape.commit_name,
        shrimply_inspector_core::alpha_mask::SHAPE_COMMIT
    );
    assert!(shape.commit_immediately);
    out.append(&crate::ui::selector(
        &shape.label,
        mask.shape,
        shape_choices.map(|(shape, _, label)| (shape, label)),
        {
            let context = context.detached();
            move |shape| {
                let Some(key) = context.selected_item.clone() else {
                    return;
                };
                if let Err(error) = context.inspector_core.set_alpha_mask_shape(
                    &InspectorTarget::Item(key),
                    mask_target,
                    shape,
                ) {
                    tracing::error!(%error, "Could not update GTK alpha-mask shape");
                    return;
                }
                focus(&context, mask_target, true);
                schedule_refresh(&context);
            }
        },
    ));

    let invert = controls
        .next()
        .expect("alpha-mask inversion control is missing");
    assert_eq!(invert.kind, ControlKind::Boolean);
    assert_eq!(invert.value, mask.invert.to_string());
    assert_eq!(
        invert.commit_name,
        shrimply_inspector_core::alpha_mask::INVERT_COMMIT
    );
    assert!(invert.commit_immediately);
    out.append(&switch_row(&invert.label, None, mask.invert, {
        let context = context.detached();
        move |invert| {
            let Some(key) = context.selected_item.clone() else {
                return;
            };
            if let Err(error) = context.inspector_core.set_alpha_mask_inverted(
                &InspectorTarget::Item(key),
                mask_target,
                invert,
            ) {
                tracing::error!(%error, "Could not update GTK alpha-mask inversion");
            }
        }
    }));

    let center = controls
        .next()
        .expect("alpha-mask center control is missing");
    out.append(&vector_widget(&center, &mask.center, context, mask_target));
    let size = controls.next().expect("alpha-mask size control is missing");
    out.append(&vector_widget(&size, &mask.size, context, mask_target));
    let rotation = controls
        .next()
        .expect("alpha-mask rotation control is missing");
    out.append(&scalar_widget(
        &rotation,
        &mask.rotation_degrees,
        context,
        mask_target,
    ));
    if mask.shape == AlphaMaskShape::Rectangle {
        let rounding = controls
            .next()
            .expect("rectangular alpha-mask roundness control is missing");
        out.append(&scalar_widget(
            &rounding,
            &mask.rounding,
            context,
            mask_target,
        ));
    }
    let feather = controls
        .next()
        .expect("alpha-mask feather control is missing");
    out.append(&scalar_widget(
        &feather,
        &mask.feather,
        context,
        mask_target,
    ));
    assert!(
        controls.next().is_none(),
        "alpha-mask has unexpected controls"
    );

    out.upcast()
}

fn vector_widget(
    control: &InspectorControl,
    timeline: &shrimply_core::timeline_value::TimelineValue<glam::Vec2>,
    context: &InspectorContext,
    mask_target: VisualAlphaMaskTarget,
) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::LayeredVector2);
    assert_eq!(control.timeline_id, Some(timeline.id));
    let prefixes = match control.prefixes.as_slice() {
        [first, second] if first == "X" && second == "Y" => ("X", "Y"),
        [first, second] if first == "W" && second == "H" => ("W", "H"),
        prefixes => panic!("unsupported alpha-mask vector prefixes: {prefixes:?}"),
    };
    assert_eq!(control.width_characters, 7);
    assert_eq!(
        control.commit_name,
        shrimply_inspector_core::alpha_mask::VECTOR_COMMIT
    );
    assert_eq!(control.store_multiplier, 1.0);
    assert!(!control.integer);
    assert!(!control.lock);
    let defaults = NumberSpec::default();
    vec_control(
        &control.label,
        timeline,
        context,
        VecTarget {
            access: VecAccess::AlphaMask {
                target: mask_target,
                value_id: timeline.id,
            },
            scope_id: Some(timeline.id),
            local_time: crate::video::visual_local_time,
            duration: crate::video::visual_duration,
            refresh: ProjectChange {
                video: true,
                ..Default::default()
            },
            commit_name: shrimply_inspector_core::alpha_mask::VECTOR_COMMIT,
        },
        VecSpec {
            first_prefix: prefixes.0,
            second_prefix: prefixes.1,
            drag_step: control.number.drag_step,
            digits: usize::try_from(control.number.digits)
                .expect("alpha-mask vector digits must be nonnegative"),
            width_chars: control.width_characters,
            minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
            maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
            unit_name: "x",
        },
    )
}

fn scalar_widget(
    control: &InspectorControl,
    timeline: &shrimply_core::timeline_value::TimelineValue<f32>,
    context: &InspectorContext,
    mask_target: VisualAlphaMaskTarget,
) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::LayeredNumber);
    assert_eq!(control.timeline_id, Some(timeline.id));
    assert_eq!(control.width_characters, 8);
    assert_eq!(
        control.commit_name,
        shrimply_inspector_core::alpha_mask::SCALAR_COMMIT
    );
    assert!(!control.integer);
    assert!(!control.lock);
    let defaults = NumberSpec::default();
    let spec = match control.store_multiplier {
        1.0 => {
            assert_eq!(control.number.unit, "°");
            assert!(control.prefix_icon_rotates);
            ScalarSpec {
                drag_step: control.number.drag_step,
                digits: usize::try_from(control.number.digits)
                    .expect("alpha-mask scalar digits must be nonnegative"),
                integer: false,
                width_chars: control.width_characters,
                minimum: (control.number.minimum != defaults.minimum)
                    .then_some(control.number.minimum),
                maximum: (control.number.maximum != defaults.maximum)
                    .then_some(control.number.maximum),
                unit_name: Some("°"),
                rotating_icon: Some((
                    "arrow3-up-symbolic",
                    control.prefix_icon_rotation_offset_degrees,
                )),
                display: f64::from,
                store: |value| value as f32,
                clamp: crate::timeline_value::scalar::ScalarClamp::Function(|value| value),
            }
        }
        0.01 => {
            assert_eq!(control.number.unit, "%");
            assert!(!control.prefix_icon_rotates);
            assert_eq!(
                (control.number.minimum, control.number.maximum),
                (0.0, 100.0)
            );
            ScalarSpec {
                drag_step: control.number.drag_step,
                digits: usize::try_from(control.number.digits)
                    .expect("alpha-mask percentage digits must be nonnegative"),
                integer: false,
                width_chars: control.width_characters,
                minimum: Some(control.number.minimum),
                maximum: Some(control.number.maximum),
                unit_name: Some("%"),
                rotating_icon: None,
                display: |value| f64::from(value) * 100.0,
                store: |value| (value / 100.0) as f32,
                clamp: crate::timeline_value::scalar::ScalarClamp::Function(|value| {
                    value.clamp(0.0, 1.0)
                }),
            }
        }
        multiplier => panic!("unsupported alpha-mask storage multiplier: {multiplier}"),
    };
    scalar_control(
        &control.label,
        timeline,
        context,
        ScalarTarget {
            access: ScalarAccess::AlphaMask {
                target: mask_target,
                value_id: timeline.id,
            },
            scope_id: Some(timeline.id),
            local_time: crate::video::visual_local_time,
            duration: crate::video::visual_duration,
            refresh: ProjectChange {
                video: true,
                ..Default::default()
            },
            commit_name: shrimply_inspector_core::alpha_mask::SCALAR_COMMIT,
        },
        spec,
    )
}

fn set_open(context: &InspectorContext, mask_target: VisualAlphaMaskTarget, open: bool) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    if let Err(error) = context.inspector_core.set_alpha_mask_enabled(
        &InspectorTarget::Item(key),
        mask_target,
        open,
    ) {
        tracing::error!(%error, "Could not toggle GTK alpha mask");
        return;
    }
    focus(context, mask_target, open);
    schedule_refresh(context);
}

fn focus(context: &InspectorContext, mask_target: VisualAlphaMaskTarget, mask: bool) {
    let Some(item) = context.preview_item.clone() else {
        return;
    };
    let Some(item_id) = context
        .project
        .borrow()
        .video_item(&item)
        .map(|item| item.id)
    else {
        return;
    };
    let focus = shrimply_inspector_core::alpha_mask::preview_focus(item_id, mask_target, mask);
    preview_focus::set(
        &context.preview_focus,
        FocusedPreview {
            item,
            card_key: focus.card_key,
            target: focus.target.resolve(item_id),
        },
    );
}

fn schedule_refresh(context: &InspectorContext) {
    let refresh = context.refresh.clone();
    gtk::glib::idle_add_local_once(move || refresh());
}
