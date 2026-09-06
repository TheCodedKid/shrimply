use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use uuid::Uuid;

use super::TimelineRuntime;
use super::external_content::{self, Content, Origin, Placement};

pub(super) fn setup(area: &gtk::GLArea, runtime: Rc<RefCell<TimelineRuntime>>) {
    let mask_drop = gtk::DropTarget::new(glib::Bytes::static_type(), gdk::DragAction::COPY);
    mask_drop.set_preload(true);
    let mask_motion_runtime = runtime.clone();
    mask_drop.connect_motion(move |target, x, y| {
        let modifier_id = target
            .value()
            .and_then(|value| value.get::<glib::Bytes>().ok())
            .and_then(|bytes| std::str::from_utf8(bytes.as_ref()).ok().map(str::to_owned))
            .and_then(|text| Uuid::parse_str(&text).ok());
        if modifier_id.is_some_and(|modifier_id| {
            mask_motion_runtime
                .borrow()
                .scene
                .mask_drop_target(modifier_id, super::vec2(x as f32, y as f32))
        }) {
            gdk::DragAction::COPY
        } else {
            gdk::DragAction::empty()
        }
    });
    let mask_area = area.clone();
    let mask_runtime = runtime.clone();
    mask_drop.connect_drop(move |_, value, x, y| {
        let Ok(bytes) = value.get::<glib::Bytes>() else {
            return false;
        };
        let Ok(text) = std::str::from_utf8(bytes.as_ref()) else {
            return false;
        };
        let Ok(modifier_id) = Uuid::parse_str(text) else {
            return false;
        };
        assign_mask_source(&mask_area, &mask_runtime, modifier_id, x, y)
    });
    area.add_controller(mask_drop);

    let formats = gdk::ContentFormats::for_type(gdk::FileList::static_type())
        .union(&gdk::ContentFormats::for_type(gio::File::static_type()))
        .union(&gdk::ContentFormats::for_type(gdk::Texture::static_type()))
        .union(&gdk::ContentFormats::for_type(String::static_type()))
        .union(&gdk::ContentFormats::new(&[
            "text/uri-list",
            "x-special/gnome-copied-files",
        ]));
    let drop = gtk::DropTarget::builder()
        .formats(&formats)
        .actions(gdk::DragAction::COPY)
        .preload(true)
        .build();
    let enter_area = area.clone();
    let enter_runtime = runtime.clone();
    drop.connect_enter(move |target, x, y| {
        let accepted = update_preview(target.value().as_ref(), &enter_runtime, x, y);
        enter_area.queue_render();
        if accepted {
            gdk::DragAction::COPY
        } else {
            gdk::DragAction::empty()
        }
    });
    let motion_area = area.clone();
    let motion_runtime = runtime.clone();
    drop.connect_motion(move |target, x, y| {
        let accepted = update_preview(target.value().as_ref(), &motion_runtime, x, y);
        motion_area.queue_render();
        if accepted {
            gdk::DragAction::COPY
        } else {
            gdk::DragAction::empty()
        }
    });
    let leave_area = area.clone();
    let leave_runtime = runtime.clone();
    drop.connect_leave(move |_| {
        let mut runtime = leave_runtime.borrow_mut();
        runtime.scene.clear_drop_preview();
        leave_area.queue_render();
    });
    let drop_area = area.clone();
    drop.connect_drop(move |target, value, x, y| {
        let Some(content) = external_content::from_value(value) else {
            runtime.borrow_mut().scene.clear_drop_preview();
            let source_formats = target
                .current_drop()
                .map(|drop| drop.formats().to_str())
                .unwrap_or_else(|| "unavailable".into());
            tracing::warn!(
                "Unsupported timeline drop payload: value_type={} source_formats={} x={x:.1} y={y:.1}",
                value.type_().name(),
                source_formats,
            );
            return false;
        };
        let content_kind = content.label();
        {
            let mut runtime = runtime.borrow_mut();
            runtime.scene.clear_drop_preview();
        }
        let inserted = external_content::insert(
            &drop_area,
            &runtime,
            content,
            Origin::Drop,
            Placement::Timeline { x, y },
        );
        if !inserted {
            tracing::warn!(
                "Timeline drop could not be inserted: content={content_kind} x={x:.1} y={y:.1}"
            );
        }
        inserted
    });
    area.add_controller(drop);
}

fn assign_mask_source(
    area: &gtk::GLArea,
    runtime: &Rc<RefCell<TimelineRuntime>>,
    modifier_id: Uuid,
    x: f64,
    y: f64,
) -> bool {
    match runtime
        .borrow_mut()
        .scene
        .assign_mask_source_at(modifier_id, super::vec2(x as f32, y as f32))
    {
        Ok(accepted) => {
            if accepted {
                area.queue_render();
            }
            accepted
        }
        Err(error) => {
            tracing::warn!(%error, "Could not assign timeline mask source");
            false
        }
    }
}

fn update_preview(
    value: Option<&glib::Value>,
    runtime: &Rc<RefCell<TimelineRuntime>>,
    x: f64,
    y: f64,
) -> bool {
    let mut runtime = runtime.borrow_mut();
    let point = super::vec2(x as f32, y as f32);
    match value.and_then(external_content::from_value) {
        Some(Content::Text(text)) => runtime.scene.update_text_drop_preview(text, point),
        Some(Content::Files(paths)) => runtime.scene.update_external_files_preview(&paths, point),
        Some(Content::Texture(_)) | Some(Content::Url(_)) => {
            runtime.scene.clear_drop_preview();
            runtime.scene.external_drop_target(point)
        }
        None => {
            runtime.scene.clear_drop_preview();
            false
        }
    }
}
