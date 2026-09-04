use std::rc::Rc;
use std::time::Duration;

use adw::prelude::{ActionRowExt, PreferencesRowExt};
use gtk::prelude::*;
use shrimply_inspector_core::{ControlKind, InspectorControl, InspectorTarget, VideoCard};
use shrimply_project::project::PdfItem;

use crate::InspectorContext;
use crate::item::{DefaultInspectorItem, InspectorListItem};

pub(super) fn item(pdf: &PdfItem, _context: &InspectorContext) -> InspectorListItem {
    DefaultInspectorItem::new(
        "pdf",
        "PDF",
        pdf.clone(),
        |_, context| controls(context),
        |context, _: PdfItem| {
            let Some(target) = context.selected_item.clone().map(InspectorTarget::Item) else {
                return;
            };
            let Some(reset) = current_card(context).and_then(|card| card.reset) else {
                return;
            };
            if let Err(error) = context.inspector_core.reset_video(&target, &reset) {
                tracing::warn!(%error, "Could not reset GTK PDF page");
            }
        },
    )
    .boxed()
}

fn controls(context: &InspectorContext) -> Vec<gtk::Widget> {
    let out = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let Some(card) = current_card(context) else {
        return vec![out.upcast()];
    };
    if loading(&card) {
        let spinner = adw::Spinner::new();
        spinner.set_size_request(18, 18);
        out.append(&spinner);
        wait_for_pages(&out, context);
    } else {
        populate(&out, card, context);
    }
    vec![out.upcast()]
}

fn wait_for_pages(out: &gtk::Box, context: &InspectorContext) {
    let out = out.downgrade();
    let listener_scope = Rc::downgrade(&context.listener_scope);
    let context = context.clone();
    gtk::glib::spawn_future_local(async move {
        loop {
            gtk::glib::timeout_future(Duration::from_millis(50)).await;
            if listener_scope.upgrade().is_none() {
                return;
            }
            shrimply_inspector_core::video::pdf::poll_pages();
            let Some(card) = current_card(&context) else {
                return;
            };
            if loading(&card) {
                continue;
            }
            let Some(out) = out.upgrade() else {
                return;
            };
            while let Some(child) = out.first_child() {
                out.remove(&child);
            }
            populate(&out, card, &context);
            return;
        }
    });
}

fn current_card(context: &InspectorContext) -> Option<VideoCard> {
    let key = context.selected_item.as_ref()?;
    context
        .project
        .borrow()
        .video_item(key)
        .map(shrimply_inspector_core::video::pdf::card)
}

fn loading(card: &VideoCard) -> bool {
    card.section
        .controls
        .first()
        .is_some_and(|control| control.kind == ControlKind::InfoLoading)
}

fn populate(out: &gtk::Box, card: VideoCard, context: &InspectorContext) {
    let Some(control) = card.section.controls.into_iter().next() else {
        return;
    };
    match control.kind {
        ControlKind::Number => add_page_control(out, control, context),
        ControlKind::ReadOnly => out.append(
            &gtk::Label::builder()
                .label(&control.value)
                .wrap(true)
                .xalign(0.0)
                .build(),
        ),
        kind => panic!("unsupported GTK PDF control kind: {kind:?}"),
    }
}

fn add_page_control(out: &gtk::Box, control: InspectorControl, context: &InspectorContext) {
    let Some(target) = context.selected_item.clone().map(InspectorTarget::Item) else {
        return;
    };
    if let Err(error) = context.inspector_core.normalize_pdf_page(&target) {
        tracing::warn!(%error, "Could not normalize GTK PDF page");
    }
    let page = adw::SpinRow::with_range(
        control.number.minimum,
        control.number.maximum,
        control.number.drag_step,
    );
    page.set_title(shrimply_gtk_components::tr!(&control.label).as_ref());
    page.set_subtitle(&control.subtitle);
    page.set_digits(
        control
            .number
            .digits
            .try_into()
            .expect("shared PDF page digits must be nonnegative"),
    );
    page.set_value(
        control
            .value
            .parse()
            .expect("shared PDF page value must be numeric"),
    );
    let controller = context.inspector_core.clone();
    let commit_name = control.commit_name;
    page.connect_value_notify(move |row| {
        let displayed = row.value().round().max(1.0) as u32;
        if let Err(error) = controller.set_pdf_page(&target, displayed - 1, &commit_name) {
            tracing::warn!(%error, "Could not change GTK PDF page");
        }
    });
    out.append(&page);
}
