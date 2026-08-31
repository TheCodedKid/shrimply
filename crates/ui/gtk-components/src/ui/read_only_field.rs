pub fn read_only_field(value: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(value)
        .selectable(true)
        .halign(gtk::Align::Fill)
        .xalign(0.0)
        .wrap(true)
        .build()
}
