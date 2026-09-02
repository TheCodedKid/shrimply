use std::{cell::Cell, rc::Rc, time::Duration};

use gtk::{glib, prelude::*};
use shrimply_audio_modifiers::CacheModifier;
use shrimply_gtk_components::ui::{ProgressButton, ProgressButtonState, control_row, dropdown};
use shrimply_inspector_core::{AudioCachePreset, AudioCacheStatus, InspectorTarget};
use uuid::Uuid;

use crate::InspectorContext;

pub fn rows(value: &CacheModifier, id: Uuid, context: &InspectorContext) -> Vec<gtk::Widget> {
    let out = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let current = shrimply_inspector_core::audio_cache_status(id);
    let edit_context = context.detached();
    let format = dropdown(
        shrimply_inspector_core::audio_cache_preset(value),
        AudioCachePreset::OPTIONS.iter().copied(),
        move |preset| {
            let Some(target) = target(&edit_context) else {
                return;
            };
            if let Err(error) =
                edit_context
                    .inspector_core
                    .set_audio_cache_preset(&target, id, preset.key())
            {
                tracing::warn!("Could not update audio cache format: {error}");
            }
        },
    );
    out.append(&control_row("Format", &format));

    let bake = ProgressButton::new("Bake");
    bake.widget().set_halign(gtk::Align::End);
    let hovered = Rc::new(Cell::new(false));
    let motion = gtk::EventControllerMotion::new();
    motion.connect_enter({
        let bake = bake.clone();
        let format = format.clone();
        let hovered = hovered.clone();
        move |_, _, _| {
            hovered.set(true);
            update_controls(
                &bake,
                &format,
                &shrimply_inspector_core::audio_cache_status(id),
                true,
            );
        }
    });
    motion.connect_leave({
        let bake = bake.clone();
        let format = format.clone();
        let hovered = hovered.clone();
        move |_| {
            hovered.set(false);
            update_controls(
                &bake,
                &format,
                &shrimply_inspector_core::audio_cache_status(id),
                false,
            );
        }
    });
    bake.widget().add_controller(motion);
    update_controls(&bake, &format, &current, false);
    out.append(bake.widget());

    bake.widget().connect_clicked({
        let context = context.detached();
        let bake = bake.clone();
        let format = format.clone();
        move |_| {
            let Some(target) = target(&context) else {
                return;
            };
            if let Err(error) = context.inspector_core.toggle_audio_cache(&target, id) {
                update_controls(&bake, &format, &AudioCacheStatus::Failed(error), false);
            }
        }
    });

    let scope = Rc::downgrade(&context.listener_scope);
    let core = context.inspector_core.clone();
    let was_baking = Rc::new(Cell::new(matches!(
        current,
        AudioCacheStatus::Baking { .. }
    )));
    glib::timeout_add_local(Duration::from_millis(50), move || {
        if scope.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let current = shrimply_inspector_core::audio_cache_status(id);
        update_controls(&bake, &format, &current, hovered.get());
        if matches!(current, AudioCacheStatus::Baking { .. }) {
            was_baking.set(true);
        } else if was_baking.replace(false) {
            core.refresh_audio_cache();
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });

    vec![out.upcast()]
}

fn update_controls(
    bake: &ProgressButton,
    format: &gtk::DropDown,
    status: &AudioCacheStatus,
    hovered: bool,
) {
    bake.widget().remove_css_class("destructive-action");
    bake.widget().remove_css_class("suggested-action");
    format.set_sensitive(!matches!(status, AudioCacheStatus::Baking { .. }));
    bake.widget().set_sensitive(true);
    match status {
        AudioCacheStatus::Missing => {
            bake.set_label("Bake");
            bake.widget().add_css_class("suggested-action");
            bake.set_state(ProgressButtonState::Idle);
        }
        AudioCacheStatus::Baking { completed, total } => {
            bake.set_label(if hovered { "Cancel" } else { "Baking…" });
            if hovered {
                bake.widget().add_css_class("destructive-action");
            }
            bake.set_state(if *total == 0 {
                ProgressButtonState::Indeterminate
            } else {
                ProgressButtonState::Progress(*completed as f64 / *total as f64)
            });
        }
        AudioCacheStatus::Ready => {
            bake.set_label("Rebake");
            bake.widget().add_css_class("suggested-action");
            bake.set_state(ProgressButtonState::Idle);
        }
        AudioCacheStatus::Failed(_) => {
            bake.set_label("Bake");
            bake.widget().add_css_class("suggested-action");
            bake.set_state(ProgressButtonState::Idle);
        }
    }
}

fn target(context: &InspectorContext) -> Option<InspectorTarget> {
    context.selected_item.clone().map(InspectorTarget::Item)
}
