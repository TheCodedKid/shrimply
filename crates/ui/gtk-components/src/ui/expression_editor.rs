use gtk::prelude::*;

pub struct ExpressionEditor {
    root: gtk::Box,
    output: gtk::Label,
}

impl ExpressionEditor {
    pub fn new(
        value: &str,
        language: Option<&str>,
        output: &str,
        edited: impl Fn(String) -> String + 'static,
    ) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let output = super::read_only_field(output);
        root.append(&super::code_editor(value, language, {
            let output = output.clone();
            move |value| output.set_label(&edited(value))
        }));
        root.append(&output);
        Self { root, output }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub fn set_output(&self, value: &str) {
        self.output.set_label(value);
    }
}
