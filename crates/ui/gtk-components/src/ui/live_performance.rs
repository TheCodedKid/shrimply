use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use adw::prelude::*;
use gtk::glib;

const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

thread_local! {
    static CSS_INSTALLED: Cell<bool> = const { Cell::new(false) };
}

pub fn live_performance() -> gtk::Widget {
    let status = adw::ExpanderRow::builder()
        .title("Live Performance")
        .build();
    let clear = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text("Clear")
        .valign(gtk::Align::Center)
        .css_classes(["flat", "live-performance-action"])
        .build();
    let copy = gtk::Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy JSON")
        .valign(gtk::Align::Center)
        .css_classes(["flat", "live-performance-action"])
        .build();
    install_css(&status.display());
    status.add_suffix(&clear);
    status.add_suffix(&copy);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.add_css_class("card");
    content.append(&status);
    let rows = Rc::new(RefCell::new(Vec::<adw::ActionRow>::new()));

    clear.connect_clicked({
        let status = status.clone();
        let rows = rows.clone();
        move |_| {
            shrimply_component_core::performance::clear();
            if status.is_expanded() {
                refresh(&status, &rows);
            }
        }
    });
    copy.connect_clicked(|button| {
        button
            .display()
            .clipboard()
            .set_text(&shrimply_component_core::performance::report_json());
    });
    status.connect_expanded_notify({
        let rows = rows.clone();
        move |status| {
            if status.is_expanded() {
                refresh(status, &rows);
            } else {
                for row in rows.take() {
                    status.remove(&row);
                }
            }
        }
    });
    let status_weak = status.downgrade();
    let status_timer = status.clone();
    glib::timeout_add_local(REFRESH_INTERVAL, move || {
        if status_weak.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        if status_timer.is_expanded() {
            refresh(&status_timer, &rows);
        }
        glib::ControlFlow::Continue
    });
    content.upcast()
}

fn install_css(display: &gtk::gdk::Display) {
    CSS_INSTALLED.with(|installed| {
        if installed.replace(true) {
            return;
        }
        let provider = gtk::CssProvider::new();
        provider.load_from_string(
            ".live-performance-action, \
             .live-performance-action:hover, \
             .live-performance-action:active { \
                 background: transparent; \
                 box-shadow: none; \
             }",
        );
        gtk::style_context_add_provider_for_display(
            display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
}

fn refresh(status: &adw::ExpanderRow, rows: &RefCell<Vec<adw::ActionRow>>) {
    for row in rows.take() {
        status.remove(&row);
    }
    let next_rows = shrimply_component_core::performance::rows()
        .into_iter()
        .map(|entry| {
            let row = adw::ActionRow::builder()
                .title(entry.title)
                .subtitle(entry.subtitle)
                .build();
            status.add_row(&row);
            row
        })
        .collect();
    *rows.borrow_mut() = next_rows;
}
