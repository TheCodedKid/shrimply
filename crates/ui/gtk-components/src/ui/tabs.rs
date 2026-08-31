use gtk::prelude::*;

pub fn tabs(pages: impl IntoIterator<Item = (String, String, gtk::Widget)>) -> gtk::Widget {
    let selector = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    selector.set_hexpand(true);
    selector.set_homogeneous(true);
    selector.set_margin_top(12);
    selector.set_margin_bottom(8);
    selector.set_margin_start(16);
    selector.set_margin_end(16);
    selector.add_css_class("linked");
    let stack = gtk::Stack::builder()
        .hexpand(true)
        .vexpand(true)
        .vhomogeneous(false)
        .build();
    let mut first = None;
    for (index, (title, icon, child)) in pages.into_iter().enumerate() {
        let name = format!("tab-{index}");
        stack.add_named(&child, Some(&name));
        let label = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        label.set_halign(gtk::Align::Center);
        label.append(&gtk::Image::from_icon_name(&icon));
        label.append(&gtk::Label::new(Some(&title)));
        let mut button = gtk::ToggleButton::builder()
            .active(index == 0)
            .hexpand(true)
            .child(&label);
        if let Some(first) = &first {
            button = button.group(first);
        }
        let button = button.build();
        let page_stack = stack.clone();
        button.connect_toggled(move |button| {
            if button.is_active() {
                page_stack.set_visible_child_name(&name);
            }
        });
        first.get_or_insert_with(|| button.clone());
        selector.append(&button);
    }
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&selector);
    content.append(&stack);
    content.upcast()
}
