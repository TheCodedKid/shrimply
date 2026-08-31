use std::rc::Rc;

use gtk::prelude::*;

use super::StringChoice;

type SearchItems = Rc<dyn Fn(&str) -> Vec<SearchMenuItem>>;

pub struct SearchMenuItem {
    pub label: String,
    activate: Rc<dyn Fn()>,
}

impl SearchMenuItem {
    pub fn new(label: impl Into<String>, activate: impl Fn() + 'static) -> Self {
        Self {
            label: label.into(),
            activate: Rc::new(activate),
        }
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
    let search = gtk::SearchEntry::builder()
        .placeholder_text(placeholder)
        .hexpand(true)
        .build();
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let scroller = gtk::ScrolledWindow::builder()
        .child(&list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_width(280)
        .min_content_height(240)
        .max_content_height(360)
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
        .has_arrow(false)
        .build();
    popover.add_css_class("menu");
    button.set_popover(Some(&popover));

    let items: SearchItems = Rc::new(items);
    populate(&list, "", &items, &popover);
    search.connect_search_changed({
        let items = items.clone();
        let list = list.clone();
        let popover = popover.clone();
        move |search| populate(&list, search.text().as_str(), &items, &popover)
    });
    popover.connect_show(move |_| {
        search.grab_focus();
    });
    button
}

pub fn modifier_menu(
    choices: Vec<StringChoice>,
    selected: impl Fn(String) + 'static,
) -> gtk::MenuButton {
    let choices = Rc::new(choices);
    let selected: Rc<dyn Fn(String)> = Rc::new(selected);
    searchable_menu("Add modifier", "Search modifiers", move |query| {
        let query = query.trim().to_lowercase();
        choices
            .iter()
            .filter(|choice| choice.label.to_lowercase().contains(&query))
            .map(|choice| {
                let value = choice.value.clone();
                let selected = selected.clone();
                SearchMenuItem::new(choice.label.clone(), move || selected(value.clone()))
            })
            .collect()
    })
}

fn populate(list: &gtk::Box, query: &str, items: &SearchItems, popover: &gtk::Popover) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for item in items(query) {
        let row = gtk::Button::builder()
            .label(&item.label)
            .halign(gtk::Align::Fill)
            .hexpand(true)
            .css_classes(["flat"])
            .build();
        let activate = item.activate;
        let popover = popover.clone();
        row.connect_clicked(move |_| {
            activate();
            popover.popdown();
        });
        list.append(&row);
    }
}
