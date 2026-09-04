use gtk::prelude::*;
use shrimply_core::modifier_model::ModifierModel;
use shrimply_gtk_components::{
    tr,
    ui::{MultilineTextInput, StringChoice, labeled_string_selector},
};
use shrimply_inspector_core::{
    ControlKind, InspectorCommit, InspectorControl, InspectorTarget, NumberSpec, TimelineModeChange,
};
use shrimply_video_modifiers::scene_3d::Text3dModifier;
use uuid::Uuid;

use crate::{InspectorContext, font_selector::font_selector_list};

use super::{ScalarOptions, color_row, integer_scalar_row, scalar_row, vec3_row, vec3_scale_row};

pub fn add_rows(value: &Text3dModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    let target = InspectorTarget::Item(
        context
            .selected_item
            .clone()
            .expect("3D text inspector must have a selected item"),
    );
    let section = context
        .inspector_core
        .text_3d_presentation(&target, id)
        .expect("3D text modifier must still be available");
    for control in section
        .controls
        .into_iter()
        .filter(|control| control.visible)
    {
        assert_eq!(control.target_id, Some(id), "3D text control changed");
        let widget = match control.kind {
            ControlKind::LayeredText => text_row(value, id, &target, &control, context),
            ControlKind::FontFamilies => fonts_row(value, id, &target, &control, context),
            ControlKind::Selector => selector_row(id, &target, control, context),
            ControlKind::Number => variation_row(id, &target, &control, context),
            ControlKind::LayeredNumber => number_row(value, id, &control, context),
            ControlKind::LayeredVector3 => vector_row(value, id, &control, context),
            ControlKind::LayeredColor => color_control(value, id, &control, context),
            kind => panic!("unsupported shared 3D text control: {kind:?}"),
        };
        out.append(&widget);
    }
}

fn text_row(
    value: &Text3dModifier,
    id: Uuid,
    target: &InspectorTarget,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = control.timeline_id.expect("3D text timeline ID is missing");
    assert_eq!(timeline_id, value.text.id);
    let path = control.path.clone();
    let edit_target = target.clone();
    let edit_controller = context.inspector_core.clone();
    let commit_target = target.clone();
    let commit_path = control.path.clone();
    let commit = control.commit_name.clone();
    let commit_controller = context.inspector_core.clone();
    let input = MultilineTextInput::builder(&control.value)
        .min_content_height(86)
        .on_change(move |next| {
            match edit_controller
                .ensure_visual_modifier(&edit_target, &path, id)
                .and_then(|()| {
                    edit_controller.set_text_value(
                        &edit_target,
                        &path,
                        timeline_id,
                        next,
                        InspectorCommit::Deferred,
                    )
                }) {
                Ok(()) => true,
                Err(error) => {
                    tracing::error!(%error, "Could not update GTK 3D text");
                    false
                }
            }
        })
        .on_commit(move || {
            if let Err(error) = commit_controller
                .ensure_visual_modifier(&commit_target, &commit_path, id)
                .and_then(|()| {
                    commit_controller.ensure_visual_modifier_text(
                        &commit_target,
                        &commit_path,
                        timeline_id,
                    )
                })
                .and_then(|()| commit_controller.commit_video_field(&commit_target, &commit))
            {
                tracing::error!(%error, "Could not commit GTK 3D text");
            }
        })
        .build();
    let mut sections = crate::timeline_value::LayeredSections::default();
    if control.layered.expression {
        let expression_target = target.clone();
        let expression_path = control.path.clone();
        let expression_controller = context.inspector_core.clone();
        let expression_commit = control.expression_commit_name.clone();
        sections.push_expression(crate::rhai_editor::editor(
            Some(control.layered.expression_source.clone()),
            crate::rhai_editor::ExpressionValue::Text,
            move |source| {
                if let Err(error) = expression_controller
                    .ensure_visual_modifier(&expression_target, &expression_path, id)
                    .and_then(|()| {
                        expression_controller.ensure_visual_modifier_text(
                            &expression_target,
                            &expression_path,
                            timeline_id,
                        )
                    })
                    .and_then(|()| {
                        expression_controller.set_expression_source_with_commit(
                            &expression_target,
                            &expression_path,
                            &source,
                            InspectorCommit::Coalesced(&expression_commit),
                        )
                    })
                {
                    tracing::error!(%error, "Could not update GTK 3D text expression");
                }
            },
        ));
    }
    let keyframe_target = target.clone();
    let keyframe_path = control.path.clone();
    let keyframe_controller = context.inspector_core.clone();
    let keyframe_commit = control.keyframe_commit_name.clone();
    let expression_target = target.clone();
    let expression_path = control.path.clone();
    let expression_controller = context.inspector_core.clone();
    let expression_commit = control.expression_commit_name.clone();
    let current = control.value.clone();
    crate::timeline_value::layered_wide_control(
        &control.label,
        &value.text,
        input.widget().clone(),
        sections,
        move |enabled| {
            if let Err(error) = keyframe_controller
                .ensure_visual_modifier(&keyframe_target, &keyframe_path, id)
                .and_then(|()| {
                    keyframe_controller.ensure_visual_modifier_text(
                        &keyframe_target,
                        &keyframe_path,
                        timeline_id,
                    )
                })
                .and_then(|()| {
                    keyframe_controller.set_text_keyframes_enabled(
                        &keyframe_target,
                        &keyframe_path,
                        timeline_id,
                        enabled,
                        InspectorCommit::Immediate(&keyframe_commit),
                    )
                })
            {
                tracing::error!(%error, "Could not toggle GTK 3D text keyframes");
            }
        },
        move |enabled| {
            if let Err(error) = expression_controller
                .ensure_visual_modifier(&expression_target, &expression_path, id)
                .and_then(|()| {
                    expression_controller.ensure_visual_modifier_text(
                        &expression_target,
                        &expression_path,
                        timeline_id,
                    )
                })
                .and_then(|()| {
                    expression_controller.set_timeline_mode_with_commit(
                        &expression_target,
                        &expression_path,
                        TimelineModeChange {
                            keyframes: false,
                            enabled,
                            current: serde_json::Value::String(current.clone()),
                            default_expression: "value",
                        },
                        InspectorCommit::Immediate(&expression_commit),
                    )
                })
            {
                tracing::error!(%error, "Could not toggle GTK 3D text expression");
            }
        },
    )
}

fn fonts_row(
    value: &Text3dModifier,
    id: Uuid,
    target: &InspectorTarget,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit = control.commit_name.clone();
    crate::ui::control_row(
        &control.label,
        &font_selector_list(&value.font_families, move |families| {
            let serialized =
                serde_json::to_string(&families).expect("font families must serialize");
            if let Err(error) = controller
                .ensure_visual_modifier(&target, &path, id)
                .and_then(|()| {
                    controller.set_video_field(&target, &path, &serialized, &commit, true)
                })
            {
                tracing::error!(%error, "Could not update GTK 3D text fonts");
            }
        }),
    )
}

fn selector_row(
    id: Uuid,
    target: &InspectorTarget,
    control: InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let choices = control
        .values
        .iter()
        .cloned()
        .zip(&control.labels)
        .map(|(value, label)| StringChoice {
            value,
            label: tr!(label).into_owned(),
        })
        .collect();
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit = control.commit_name.clone();
    labeled_string_selector(&control.label, &control.value, choices, move |value| {
        if let Err(error) = controller
            .ensure_visual_modifier(&target, &path, id)
            .and_then(|()| controller.set_video_field(&target, &path, &value, &commit, true))
        {
            tracing::error!(%error, "Could not update GTK 3D text selector");
        }
    })
    .widget()
    .clone()
}

fn variation_row(
    id: Uuid,
    target: &InspectorTarget,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let spin = gtk::SpinButton::with_range(
        control.number.minimum,
        control.number.maximum,
        control.number.drag_step,
    );
    spin.set_digits(u32::try_from(control.number.digits).expect("digits must be non-negative"));
    spin.set_width_chars(control.width_characters);
    spin.set_value(
        control
            .value
            .parse()
            .expect("shared font variation must be numeric"),
    );
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let commit = control.commit_name.clone();
    spin.connect_value_changed(move |spin| {
        if let Err(error) = controller
            .ensure_visual_modifier(&target, &path, id)
            .and_then(|()| {
                controller.set_video_field(&target, &path, &spin.value().to_string(), &commit, true)
            })
        {
            tracing::error!(%error, "Could not update GTK 3D text variation");
        }
    });
    crate::ui::control_row(&control.label, &spin)
}

fn number_row(
    value: &Text3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline = value
        .number(control.timeline_id.expect("3D text number ID is missing"))
        .expect("3D text number changed");
    let defaults = NumberSpec::default();
    let options = ScalarOptions {
        minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
        maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
        unit: (!control.number.unit.is_empty()).then_some(control.number.unit),
        rotating: false,
    };
    if control.integer {
        integer_scalar_row(&control.label, timeline, id, options, context)
    } else {
        scalar_row(&control.label, timeline, id, options, context)
    }
}

fn vector_row(
    value: &Text3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline = value
        .number3(control.timeline_id.expect("3D text vector ID is missing"))
        .expect("3D text vector changed");
    if control.lock {
        vec3_scale_row(&control.label, timeline, id, context)
    } else {
        vec3_row(
            &control.label,
            timeline,
            id,
            control.number.unit == "°",
            context,
        )
    }
}

fn color_control(
    value: &Text3dModifier,
    id: Uuid,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.timeline_id, Some(value.material.base_color.id));
    color_row(&control.label, &value.material.base_color, id, context)
}
