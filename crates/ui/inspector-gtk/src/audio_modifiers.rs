use std::{rc::Rc, thread};

use gtk::{glib, prelude::*};
use shrimply_audio_modifiers::{AudioModifier, AudioModifierEffect};
use shrimply_core::modifier_model::ModifierModel;
use shrimply_core::timeline_value::{Interpolation, TimelineValue};
use shrimply_gtk_components::{
    tr,
    ui::{
        I18nWidgetExt, NumberPicker, SearchMenuItem, control_row, search_rank, searchable_menu,
        switch_row,
    },
};
use shrimply_inspector_core::{
    AudioModifierControl, AudioModifierKeyframeMove, AudioModifierScalarPresentation,
    InspectorTarget, LayeredState,
};
use shrimply_project::project::Time;
use uuid::Uuid;

use super::InspectorContext;
use super::item::{DefaultInspectorItem, HeaderAction, HeaderToggle, InspectorListItem, flat};
use super::selector::{StringChoice, labeled_string_selector, string_selector};
use super::timeline_value::{
    ExpressionOutput, LayeredSections, expression_section, layered_control,
};
use crate::keyframe_editor::{self, KeyframeClipboardActions, KeyframeEditorActions};
use crate::player_state;

pub(super) fn items(
    modifiers: &[AudioModifier],
    context: &InspectorContext,
) -> Vec<InspectorListItem> {
    let mut rows = modifiers
        .iter()
        .enumerate()
        .map(|(index, modifier)| modifier_item(modifier, index, modifiers.len(), context))
        .collect::<Vec<_>>();
    rows.push(flat(modifier_buttons(context)));
    rows
}

fn modifier_item(
    modifier: &AudioModifier,
    index: usize,
    len: usize,
    context: &InspectorContext,
) -> InspectorListItem {
    let id = modifier.id;
    let default = shrimply_inspector_core::default_audio_modifier_effect(&modifier.effect);
    DefaultInspectorItem::new_with_default(
        format!("audio-modifier:{}", modifier.id),
        modifier.effect.display_name(),
        modifier.effect.clone(),
        move |effect, context| modifier_rows(effect, id, context),
        move |_| default.clone(),
        move |context, effect| reset(context, id, effect),
    )
    .toggle(modifier_toggle(modifier, context))
    .actions(actions(modifier.id, index, len, context))
    .boxed()
}

fn modifier_rows(
    effect: &AudioModifierEffect,
    id: Uuid,
    context: &InspectorContext,
) -> Vec<gtk::Widget> {
    let mut rows = Vec::new();
    for control in shrimply_inspector_core::audio_modifier_controls(effect) {
        match control {
            AudioModifierControl::Cache(value) => {
                return crate::audio_cache::rows(&value, id, context);
            }
            AudioModifierControl::Scalar {
                path,
                label,
                value,
                presentation,
            } => rows.push(scalar_row(&path, label, &value, id, presentation, context)),
            AudioModifierControl::Boolean { path, label, value } => {
                let context = context.detached();
                rows.push(switch_row(label, None, value, move |value| {
                    set_field(&context, id, &path, &value.to_string());
                }));
            }
            AudioModifierControl::Selector {
                path,
                label,
                value,
                options,
            } => {
                let context = context.detached();
                let selector = labeled_string_selector(
                    label,
                    &value,
                    options
                        .into_iter()
                        .map(|option| StringChoice {
                            value: option.value.to_string(),
                            label: tr!(option.label).into_owned(),
                        })
                        .collect(),
                    move |value| set_field(&context, id, &path, &value),
                );
                rows.push(selector.widget().clone());
            }
            AudioModifierControl::Number {
                path,
                label,
                value,
                minimum,
                maximum,
                step,
                digits,
            } => {
                let change_context = context.detached();
                let commit_context = context.detached();
                let picker = NumberPicker::builder(value)
                    .minimum(minimum)
                    .maximum(maximum)
                    .drag_step(step)
                    .digits(usize::try_from(digits).expect("number digits must be non-negative"))
                    .on_change(move |value| {
                        set_live_field(&change_context, id, &path, &value.to_string());
                    })
                    .on_commit(move |_| commit_context.inspector_core.finish_live_edit())
                    .build();
                rows.push(control_row(label, &picker));
            }
            AudioModifierControl::VoiceModel { path, label, value } => {
                rows.push(voice_model_row(path, label, value, id, context));
            }
        }
    }
    controls(rows)
}

fn scalar_row(
    path: &str,
    label: &str,
    value: &TimelineValue<f32>,
    modifier_id: Uuid,
    presentation: AudioModifierScalarPresentation,
    context: &InspectorContext,
) -> gtk::Widget {
    let target = target(context).expect("audio modifier scalar requires an audio item");
    let value_id = value.id;
    let current = warned(
        "read audio modifier scalar",
        context
            .inspector_core
            .audio_modifier_number_value(&target, modifier_id, value_id),
    )
    .map(|value| (presentation.display)(value as f32))
    .unwrap_or_else(|| (presentation.display)(value.fallback()));
    let change_controller = context.inspector_core.clone();
    let change_target = target.clone();
    let commit_controller = context.inspector_core.clone();
    let picker = NumberPicker::builder(current)
        .minimum(presentation.minimum)
        .maximum(presentation.maximum)
        .drag_step(presentation.drag_step)
        .digits(presentation.digits)
        .unit_name(presentation.unit.unwrap_or_default())
        .on_change(move |value| {
            warned(
                "edit audio modifier scalar",
                change_controller.set_audio_modifier_timeline_base(
                    &change_target,
                    modifier_id,
                    value_id,
                    (presentation.store)(value),
                ),
            );
        })
        .on_commit(move |_| commit_controller.finish_live_edit())
        .build_with_handle();

    let display_controller = context.inspector_core.clone();
    let display_target = target.clone();
    let display_handle = picker.handle.downgrade();
    let alive = Rc::downgrade(&context.listener_scope);
    player_state::connect_while_alive_named(
        &context.player_state,
        "audio modifier scalar display",
        move || alive.upgrade().is_some(),
        move |event| {
            if !matches!(
                event,
                player_state::PlayerEvent::State(_) | player_state::PlayerEvent::Project(_)
            ) {
                return;
            }
            let Some(handle) = display_handle.upgrade() else {
                return;
            };
            if let Ok(value) = display_controller.audio_modifier_number_value(
                &display_target,
                modifier_id,
                value_id,
            ) {
                handle.set_f64((presentation.display)(value as f32));
            }
        },
    );

    let layers = LayeredState::from(value);
    let mut sections = LayeredSections::default();
    if layers.keyframes {
        let runtime = context.inspector_core.snapshot().runtime;
        let built = keyframe_editor::build(
            context,
            shrimply_inspector_core::keyframe_model::scalar_graph(
                value,
                current,
                presentation.display,
            ),
            (Time::ZERO, runtime.duration.unwrap_or(Time::ZERO)),
            format!("audio-modifier:{modifier_id}:{value_id}"),
            scalar_keyframe_actions(
                context,
                target.clone(),
                modifier_id,
                value_id,
                presentation.store_multiplier,
            ),
        );
        let graph_controller = context.inspector_core.clone();
        let graph_target = target.clone();
        keyframe_editor::connect_graph_refresh(
            context,
            "audio modifier scalar graph refresh",
            &built,
            move || {
                let value = graph_controller
                    .audio_modifier_timeline(&graph_target, modifier_id, value_id)
                    .ok()?;
                let current = graph_controller
                    .audio_modifier_number_value(&graph_target, modifier_id, value_id)
                    .ok()
                    .map(|value| (presentation.display)(value as f32))
                    .unwrap_or_else(|| (presentation.display)(value.fallback()));
                Some(shrimply_inspector_core::keyframe_model::scalar_graph(
                    &value,
                    current,
                    presentation.display,
                ))
            },
        );
        sections.set_keyframe(built.widget);
    }
    if layers.expression {
        sections.push_expression(scalar_expression_section(
            context,
            target.clone(),
            modifier_id,
            value_id,
            path.to_string(),
            layers.expression_source,
            presentation,
        ));
    }

    let keyframe_controller = context.inspector_core.clone();
    let keyframe_target = target.clone();
    let expression_controller = context.inspector_core.clone();
    let expression_path = path.to_string();
    layered_control(
        label,
        value,
        picker.widget,
        sections,
        move |enabled| {
            warned(
                "toggle audio modifier keyframes",
                keyframe_controller.set_audio_modifier_keyframes_enabled(
                    &keyframe_target,
                    modifier_id,
                    value_id,
                    enabled,
                ),
            );
        },
        move |enabled| {
            warned(
                "toggle audio modifier expression",
                expression_controller.set_audio_modifier_expression_enabled(
                    &target,
                    modifier_id,
                    &expression_path,
                    enabled,
                    "value",
                ),
            );
        },
    )
}

fn scalar_expression_section(
    context: &InspectorContext,
    target: InspectorTarget,
    modifier_id: Uuid,
    value_id: Uuid,
    path: String,
    source: String,
    presentation: AudioModifierScalarPresentation,
) -> gtk::Widget {
    let editor_controller = context.inspector_core.clone();
    let editor_target = target.clone();
    let output_controller = context.inspector_core.clone();
    expression_section(
        context,
        "audio modifier scalar expression output",
        move |refresh| {
            crate::rhai_editor::editor(
                Some(source),
                crate::rhai_editor::ExpressionValue::Scalar,
                move |source| {
                    warned(
                        "edit audio modifier expression",
                        editor_controller.set_audio_modifier_expression_source(
                            &editor_target,
                            modifier_id,
                            &path,
                            &source,
                        ),
                    );
                    refresh();
                },
            )
        },
        move |_, _, _, _| {
            let output = output_controller
                .audio_modifier_expression_output(&target, modifier_id, value_id)
                .ok()?;
            Some(ExpressionOutput {
                value: format!(
                    "{:.*}{}",
                    presentation.digits,
                    (presentation.display)(output.value),
                    presentation.unit.unwrap_or_default(),
                ),
                error: output.error,
            })
        },
    )
}

fn scalar_keyframe_actions(
    context: &InspectorContext,
    target: InspectorTarget,
    modifier_id: Uuid,
    value_id: Uuid,
    store_multiplier: f64,
) -> KeyframeEditorActions {
    let controller = context.inspector_core.clone();
    let add_controller = controller.clone();
    let add_target = target.clone();
    let delete_controller = controller.clone();
    let delete_target = target.clone();
    let move_controller = controller.clone();
    let move_target = target.clone();
    let copy_controller = controller.clone();
    let copy_target = target.clone();
    let paste_controller = controller.clone();
    let paste_target = target.clone();
    let interpolation_controller = controller.clone();
    let interpolation_target = target;
    KeyframeEditorActions {
        add_at_time: Rc::new(move |time| {
            warned(
                "add audio modifier keyframe",
                add_controller.add_audio_modifier_keyframe(
                    &add_target,
                    modifier_id,
                    value_id,
                    time,
                ),
            );
        }),
        delete_at_time: Rc::new(move |time| {
            warned(
                "delete audio modifier keyframe",
                delete_controller.delete_audio_modifier_keyframe(
                    &delete_target,
                    modifier_id,
                    value_id,
                    time,
                ),
            );
        }),
        update_point: Rc::new(move |old_time, time, displayed_value| {
            warned(
                "move audio modifier keyframe",
                move_controller.move_audio_modifier_keyframe(
                    &move_target,
                    modifier_id,
                    value_id,
                    AudioModifierKeyframeMove {
                        old_time,
                        time,
                        displayed_value,
                        store_multiplier,
                    },
                ),
            );
        }),
        clipboard: KeyframeClipboardActions::Managed {
            copy: Rc::new(move |times| {
                warned(
                    "copy audio modifier keyframes",
                    copy_controller.copy_audio_modifier_keyframes(
                        &copy_target,
                        modifier_id,
                        value_id,
                        times,
                    ),
                )
            }),
            paste: Rc::new(move |time| {
                warned(
                    "paste audio modifier keyframes",
                    paste_controller.paste_audio_modifier_keyframes(
                        &paste_target,
                        modifier_id,
                        value_id,
                        time,
                    ),
                )
            }),
        },
        set_interpolation: Some(Rc::new(move |owner_id, interpolation| {
            let interpolation = Interpolation::KEYFRAME
                .iter()
                .position(|candidate| *candidate == interpolation)
                .expect("audio modifier interpolation must be available");
            warned(
                "change audio modifier interpolation",
                interpolation_controller.set_audio_modifier_keyframe_interpolation(
                    &interpolation_target,
                    modifier_id,
                    value_id,
                    owner_id,
                    interpolation,
                ),
            );
        })),
        text_interpolation: None,
        toggle_playback: Rc::new(move || controller.toggle_keyframe_playback()),
    }
}

fn warned<T>(operation: &str, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::warn!("Could not {operation}: {error}");
            None
        }
    }
}

fn voice_model_row(
    path: String,
    label: &'static str,
    current: String,
    id: Uuid,
    context: &InspectorContext,
) -> gtk::Widget {
    let server_url = shrimply_state::preferences::snapshot(&context.preferences).compute_server_url;
    let detached = context.detached();
    let model = string_selector(label, &current, vec![current.clone()], move |value| {
        set_field(&detached, id, &path, &value);
    });
    if let Some(models) = shrimply_inspector_core::cached_voice_change_models(&server_url, &current)
    {
        model.set_options(&current, models);
        return model.widget().clone();
    }
    model.set_sensitive(false);
    model
        .widget()
        .set_tooltip_text(Some(tr!("Loading Pneuma models…").as_ref()));
    let model_update = model.clone();
    let (sender, receiver) = async_channel::bounded(1);
    let requested = current.clone();
    let request_current = current.clone();
    thread::spawn(move || {
        let result = shrimply_inspector_core::voice_change_models(&server_url, &request_current);
        let _ = sender.send_blocking(result);
    });
    glib::spawn_future_local(async move {
        match receiver.recv().await {
            Ok(Ok(models)) => {
                model_update.set_options(&requested, models);
                model_update.set_sensitive(true);
                model_update.widget().set_tooltip_text(None);
            }
            Ok(Err(error)) => model_update.widget().set_tooltip_text(Some(&error)),
            Err(_) => model_update
                .widget()
                .set_tooltip_i18n("Pneuma model request stopped unexpectedly"),
        }
    });
    model.widget().clone()
}

fn controls(rows: impl IntoIterator<Item = gtk::Widget>) -> Vec<gtk::Widget> {
    let out = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for row in rows {
        out.append(&row);
    }
    vec![out.upcast()]
}

fn modifier_toggle(modifier: &AudioModifier, context: &InspectorContext) -> HeaderToggle {
    let id = modifier.id;
    let context = context.detached();
    HeaderToggle {
        active: modifier.enabled,
        tooltip: if modifier.enabled {
            "Disable modifier"
        } else {
            "Enable modifier"
        },
        activate: Rc::new(move |enabled| {
            let Some(target) = target(&context) else {
                return;
            };
            if let Err(error) = context
                .inspector_core
                .set_audio_modifier_enabled(&target, id, enabled)
            {
                tracing::warn!("Could not toggle audio modifier: {error}");
            }
        }),
    }
}

fn set_field(context: &InspectorContext, id: Uuid, path: &str, value: &str) {
    let Some(target) = target(context) else {
        return;
    };
    if let Err(error) = context
        .inspector_core
        .set_audio_modifier_field(&target, id, path, value)
    {
        tracing::warn!("Could not update audio modifier: {error}");
    }
}

fn set_live_field(context: &InspectorContext, id: Uuid, path: &str, value: &str) {
    let Some(target) = target(context) else {
        return;
    };
    if let Err(error) = context
        .inspector_core
        .set_audio_modifier_live_field(&target, id, path, value)
    {
        tracing::warn!("Could not update audio modifier: {error}");
    }
}

fn reset(context: &InspectorContext, id: Uuid, effect: AudioModifierEffect) {
    let Some(target) = target(context) else {
        return;
    };
    if let Err(error) = context
        .inspector_core
        .reset_audio_modifier_effect(&target, id, effect)
    {
        tracing::warn!("Could not reset audio modifier: {error}");
    }
}

#[derive(Clone, Copy)]
enum Action {
    Copy,
    Up,
    Down,
    Remove,
}

fn actions(id: Uuid, index: usize, len: usize, context: &InspectorContext) -> Vec<HeaderAction> {
    [
        ("edit-copy-symbolic", "Copy", Action::Copy, true),
        ("go-up-symbolic", "Move up", Action::Up, index > 0),
        (
            "go-down-symbolic",
            "Move down",
            Action::Down,
            index + 1 < len,
        ),
        ("user-trash-symbolic", "Remove", Action::Remove, true),
    ]
    .into_iter()
    .map(|(icon, tooltip, action, sensitive)| {
        let context = context.detached();
        HeaderAction {
            icon,
            tooltip,
            sensitive,
            activate: Rc::new(move || apply_action(&context, id, action)),
        }
    })
    .collect()
}

fn apply_action(context: &InspectorContext, id: Uuid, action: Action) {
    let Some(target) = target(context) else {
        return;
    };
    let result = match action {
        Action::Copy => context
            .inspector_core
            .copy_audio_modifier(&target, id, &context.property_clipboard)
            .map(Some),
        Action::Up => context
            .inspector_core
            .move_audio_modifier(&target, id, -1)
            .map(|()| None),
        Action::Down => context
            .inspector_core
            .move_audio_modifier(&target, id, 1)
            .map(|()| None),
        Action::Remove => context
            .inspector_core
            .remove_audio_modifier(&target, id)
            .map(|()| None),
    };
    match result {
        Ok(Some(name)) => {
            let message =
                shrimply_gtk_components::i18n::text_args("%{name} copied", &[("name", name)]);
            shrimply_gtk_components::toast::show_confirmation_text_for_widget(
                &context.category_bar,
                &message,
            );
        }
        Ok(None) => {}
        Err(error) => tracing::warn!("Could not edit audio modifier chain: {error}"),
    }
}

fn modifier_buttons(context: &InspectorContext) -> gtk::Widget {
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    buttons.set_halign(gtk::Align::Center);
    buttons.append(&add_button(context));
    let can_paste = target(context).is_some_and(|target| {
        context
            .inspector_core
            .can_paste_audio_modifiers(&target, &context.property_clipboard)
    });
    if can_paste {
        let paste = gtk::Button::builder()
            .icon_name("edit-paste-symbolic")
            .tooltip_text(tr!("Paste Modifier").as_ref())
            .build();
        let context = context.detached();
        paste.connect_clicked(move |_| paste_modifiers(&context));
        buttons.append(&paste);
    }
    buttons.upcast()
}

fn paste_modifiers(context: &InspectorContext) {
    let Some(target) = target(context) else {
        return;
    };
    match context
        .inspector_core
        .paste_audio_modifiers(&target, &context.property_clipboard)
    {
        Ok(0) => {}
        Ok(count) => {
            let message = if count == 1 {
                tr!("1 effect pasted").into_owned()
            } else {
                shrimply_gtk_components::i18n::text_args(
                    "%{count} effects pasted",
                    &[("count", count.to_string())],
                )
            };
            shrimply_gtk_components::toast::show_confirmation_text_for_widget(
                &context.category_bar,
                &message,
            );
        }
        Err(error) => tracing::warn!("Could not paste audio modifiers: {error}"),
    }
}

fn add_button(context: &InspectorContext) -> gtk::Widget {
    let context = context.detached();
    searchable_menu(
        tr!("Add modifier").as_ref(),
        tr!("Search modifiers").as_ref(),
        move |query| {
            let mut choices = shrimply_inspector_core::audio_modifier_catalog()
                .into_iter()
                .filter_map(|choice| {
                    let rank = search_rank(choice.label, [&*choice.search_text], query)?;
                    Some((rank, choice))
                })
                .collect::<Vec<_>>();
            choices.sort_by_key(|(rank, _)| *rank);
            choices
                .into_iter()
                .map(|(_, choice)| {
                    let label = tr!(choice.label).into_owned();
                    let context = context.detached();
                    SearchMenuItem::new(label, move || add(&context, &choice.key))
                })
                .collect()
        },
    )
    .upcast()
}

fn add(context: &InspectorContext, kind: &str) {
    let Some(target) = target(context) else {
        return;
    };
    if let Err(error) = context.inspector_core.add_audio_modifier(&target, kind) {
        tracing::warn!("Could not add audio modifier: {error}");
    }
}

fn target(context: &InspectorContext) -> Option<InspectorTarget> {
    context.selected_item.clone().map(InspectorTarget::Item)
}
