use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use gtk::{glib, prelude::*};
use shrimply_audio_modifiers::CacheModifier;
use shrimply_gtk_components::ui::{ProgressButton, ProgressButtonState, control_row, dropdown};
use shrimply_inspector_core::{
    AudioCachePreset, CacheStatus, ControlKind, InspectorControl, InspectorTarget,
};
use uuid::Uuid;

use crate::InspectorContext;

pub fn rows(value: &CacheModifier, id: Uuid, context: &InspectorContext) -> Vec<gtk::Widget> {
    let out = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let presentation = shrimply_inspector_core::audio_cache_presentation(value, id);
    let current = presentation.status;
    let [format_control, bake_control] = presentation.section.controls.as_slice() else {
        unreachable!("audio cache presentation must contain format and bake controls")
    };
    validate_controls(format_control, bake_control, id);
    let edit_context = context.detached();
    let format = dropdown(
        AudioCachePreset::from_key(&format_control.value)
            .expect("audio cache presentation must contain a valid preset"),
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
    format.set_sensitive(format_control.sensitive);
    out.append(&control_row(&format_control.label, &format));

    let bake = ProgressButton::new(&bake_control.value);
    bake.widget().set_halign(gtk::Align::End);
    let hovered = Rc::new(Cell::new(false));
    let tracker = Rc::new(RefCell::new(
        shrimply_inspector_core::CacheStatusTracker::default(),
    ));
    tracker.borrow_mut().observe((), id, current.clone());
    let motion = gtk::EventControllerMotion::new();
    motion.connect_enter({
        let bake = bake.clone();
        let format = format.clone();
        let hovered = hovered.clone();
        let tracker = tracker.clone();
        move |_, _, _| {
            hovered.set(true);
            let status = tracker
                .borrow()
                .tracked((), id)
                .cloned()
                .expect("audio cache status must remain tracked while its row exists");
            update_controls(&bake, &format, &status, true);
        }
    });
    motion.connect_leave({
        let bake = bake.clone();
        let format = format.clone();
        let hovered = hovered.clone();
        let tracker = tracker.clone();
        move |_| {
            hovered.set(false);
            let status = tracker
                .borrow()
                .tracked((), id)
                .cloned()
                .expect("audio cache status must remain tracked while its row exists");
            update_controls(&bake, &format, &status, false);
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
                update_controls(&bake, &format, &CacheStatus::Failed(error), false);
            }
        }
    });

    if matches!(current, CacheStatus::Baking { .. }) {
        let scope = Rc::downgrade(&context.listener_scope);
        let core = context.inspector_core.clone();
        glib::timeout_add_local(Duration::from_millis(50), move || {
            if scope.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            let poll = tracker
                .borrow_mut()
                .poll(|(), tracked_id| shrimply_inspector_core::audio_cache_status(tracked_id));
            if poll.changed {
                let status = tracker
                    .borrow()
                    .tracked((), id)
                    .cloned()
                    .expect("audio cache status must remain tracked while its row exists");
                update_controls(&bake, &format, &status, hovered.get());
            }
            if poll.finished.contains(&()) {
                core.refresh_audio_cache();
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });
    }

    vec![out.upcast()]
}

fn update_controls(
    bake: &ProgressButton,
    format: &gtk::DropDown,
    status: &CacheStatus,
    hovered: bool,
) {
    let control = shrimply_inspector_core::audio_cache_control(status.clone());
    bake.widget().remove_css_class("destructive-action");
    bake.widget().remove_css_class("suggested-action");
    format.set_sensitive(!control.baking);
    bake.widget().set_sensitive(true);
    bake.set_label(if control.baking && hovered {
        "Cancel"
    } else {
        control.label
    });
    bake.widget()
        .set_tooltip_text((!control.tooltip.is_empty()).then_some(control.tooltip.as_str()));
    if control.baking && hovered {
        bake.widget().add_css_class("destructive-action");
    } else if !control.baking {
        bake.widget().add_css_class("suggested-action");
    }
    bake.set_state(if !control.baking {
        ProgressButtonState::Idle
    } else if control.progress < 0.0 {
        ProgressButtonState::Indeterminate
    } else {
        ProgressButtonState::Progress(control.progress)
    });
}

fn validate_controls(format: &InspectorControl, bake: &InspectorControl, id: Uuid) {
    assert_eq!(format.kind, ControlKind::AudioCachePreset);
    assert_eq!(format.target_id, Some(id));
    assert_eq!(format.commit_name, "audio-cache-format");
    assert_eq!(bake.kind, ControlKind::AudioCache);
    assert_eq!(bake.target_id, Some(id));
}

fn target(context: &InspectorContext) -> Option<InspectorTarget> {
    context.selected_item.clone().map(InspectorTarget::Item)
}
