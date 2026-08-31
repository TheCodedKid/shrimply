use gtk::{gio, glib, prelude::*};
use shrimply_component_core::file_picker;

pub fn open<P: IsA<gtk::Window>>(
    label: &str,
    dialog: &gtk::FileDialog,
    parent: Option<&P>,
    selected: impl FnOnce(Result<gio::File, glib::Error>) + 'static,
) {
    restore_folder(label, dialog);
    let label = label.to_owned();
    dialog.open(parent, None::<&gio::Cancellable>, move |result| {
        remember_parent(&label, &result);
        selected(result);
    });
}

pub fn save<P: IsA<gtk::Window>>(
    label: &str,
    dialog: &gtk::FileDialog,
    parent: Option<&P>,
    selected: impl FnOnce(Result<gio::File, glib::Error>) + 'static,
) {
    restore_folder(label, dialog);
    let label = label.to_owned();
    dialog.save(parent, None::<&gio::Cancellable>, move |result| {
        remember_parent(&label, &result);
        selected(result);
    });
}

fn restore_folder(label: &str, dialog: &gtk::FileDialog) {
    if let Some(folder) = file_picker::initial_folder(label) {
        dialog.set_initial_folder(Some(&gio::File::for_path(folder)));
    }
}

fn remember_parent(label: &str, result: &Result<gio::File, glib::Error>) {
    let Some(path) = result.as_ref().ok().and_then(gio::File::path) else {
        return;
    };
    file_picker::remember_file(label, &path);
}
