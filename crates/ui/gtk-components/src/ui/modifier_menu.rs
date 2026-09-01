use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use gtk::{gdk, glib};

use super::StringChoice;

type SearchItems = Rc<dyn Fn(&str) -> Vec<SearchMenuItem>>;

pub struct SearchMenuItem {
    pub label: String,
    selected: bool,
    tooltip: Option<String>,
    activate: Rc<dyn Fn()>,
}

impl SearchMenuItem {
    pub fn new(label: impl Into<String>, activate: impl Fn() + 'static) -> Self {
        Self {
            label: label.into(),
            selected: false,
            tooltip: None,
            activate: Rc::new(activate),
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

pub fn searchable_menu(
    label: &str,
    placeholder: &str,
    items: impl Fn(&str) -> Vec<SearchMenuItem> + 'static,
) -> gtk::MenuButton {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    content.append(&gtk::Image::from_icon_name("list-add-symbolic"));
    content.append(&gtk::Label::new(Some(label)));
    let button = gtk::MenuButton::builder()
        .child(&content)
        .halign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    let popover = searchable_popover(placeholder, 280, 240, 360, items);
    popover.set_has_arrow(false);
    button.set_popover(Some(&popover));
    button
}

pub fn searchable_popover(
    placeholder: &str,
    minimum_width: i32,
    minimum_height: i32,
    maximum_height: i32,
    items: impl Fn(&str) -> Vec<SearchMenuItem> + 'static,
) -> gtk::Popover {
    let search = gtk::SearchEntry::builder()
        .placeholder_text(placeholder)
        .hexpand(true)
        .build();
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let scroller = gtk::ScrolledWindow::builder()
        .child(&list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_width(minimum_width)
        .min_content_height(minimum_height)
        .max_content_height(maximum_height)
        .build();
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    content.append(&search);
    content.append(&scroller);
    let popover = gtk::Popover::builder()
        .child(&content)
        .autohide(true)
        .build();
    popover.add_css_class("menu");
    let items: SearchItems = Rc::new(items);
    let initial = Rc::new(RefCell::new(populate(&list, "", &items, &popover)));
    search.connect_search_changed({
        let initial = initial.clone();
        let items = items.clone();
        let list = list.clone();
        let popover = popover.clone();
        move |search| {
            *initial.borrow_mut() = populate(&list, search.text().as_str(), &items, &popover);
        }
    });
    let keys = gtk::EventControllerKey::new();
    keys.connect_key_pressed({
        let initial = initial.clone();
        let list = list.clone();
        let popover = popover.clone();
        move |_, key, _, _| match key {
            gdk::Key::Down => {
                if let Some(row) = list.first_child() {
                    row.grab_focus();
                }
                glib::Propagation::Stop
            }
            gdk::Key::Up => {
                if let Some(row) = list.last_child() {
                    row.grab_focus();
                }
                glib::Propagation::Stop
            }
            gdk::Key::Return | gdk::Key::KP_Enter => {
                if let Some(row) = initial.borrow().as_ref() {
                    row.emit_clicked();
                }
                glib::Propagation::Stop
            }
            gdk::Key::Escape => {
                popover.popdown();
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    search.add_controller(keys);
    popover.connect_show({
        let initial = initial.clone();
        let items = items.clone();
        let list = list.clone();
        let popover = popover.clone();
        move |_| {
            if search.text().is_empty() {
                *initial.borrow_mut() = populate(&list, "", &items, &popover);
            } else {
                search.set_text("");
            }
            search.grab_focus();
        }
    });
    popover
}

pub fn modifier_menu(
    choices: Vec<StringChoice>,
    selected: impl Fn(String) + 'static,
) -> gtk::MenuButton {
    let choices = Rc::new(choices);
    let selected: Rc<dyn Fn(String)> = Rc::new(selected);
    searchable_menu("Add modifier", "Search modifiers", move |query| {
        choices
            .iter()
            .filter(|choice| shrimply_component_core::selector::matches_query(&choice.label, query))
            .map(|choice| {
                let value = choice.value.clone();
                let selected = selected.clone();
                SearchMenuItem::new(choice.label.clone(), move || selected(value.clone()))
            })
            .collect()
    })
}

fn populate(
    list: &gtk::Box,
    query: &str,
    items: &SearchItems,
    popover: &gtk::Popover,
) -> Option<gtk::Button> {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let mut first = None;
    let mut selected = None;
    for item in items(query) {
        let label = gtk::Label::builder()
            .label(&item.label)
            .halign(gtk::Align::Start)
            .xalign(0.0)
            .hexpand(true)
            .build();
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        content.append(&label);
        if item.selected {
            content.append(&gtk::Image::from_icon_name("object-select-symbolic"));
        }
        let row = gtk::Button::builder()
            .child(&content)
            .halign(gtk::Align::Fill)
            .hexpand(true)
            .css_classes(["flat"])
            .build();
        first.get_or_insert_with(|| row.clone());
        if item.selected {
            selected = Some(row.clone());
        }
        if let Some(tooltip) = &item.tooltip {
            row.set_tooltip_text(Some(tooltip));
        }
        let activate = item.activate;
        let click_popover = popover.clone();
        row.connect_clicked(move |_| {
            activate();
            click_popover.popdown();
        });
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed({
            let row = row.clone();
            let popover = popover.clone();
            move |_, key, _, _| match key {
                gdk::Key::Down => {
                    if let Some(next) = row.next_sibling() {
                        next.grab_focus();
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Up => {
                    if let Some(previous) = row.prev_sibling() {
                        previous.grab_focus();
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    popover.popdown();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        row.add_controller(keys);
        list.append(&row);
    }
    selected.or(first)
}
