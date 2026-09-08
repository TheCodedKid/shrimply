use gtk::gio::prelude::AppLaunchContextExt;
use gtk::glib;
use gtk::prelude::{Cast, DisplayExt, GdkAppLaunchContextExt, WidgetExt};
use gtk::{gdk, gio};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn show_path_in_folder(widget: &gtk::Widget, path: PathBuf) -> Result<(), String> {
    let token = portal_activation_token(widget, &path);
    match shrimply_cross_ui_core::desktop_open::prepare(&path, token.as_deref())? {
        shrimply_cross_ui_core::desktop_open::Action::Open(path) => launch_file(widget, path),
        shrimply_cross_ui_core::desktop_open::Action::FocusRevealed(path) => {
            activate_parent_folder_after_reveal(widget, path)
        }
    }
    Ok(())
}

pub fn reveal_file(widget: &gtk::Widget, path: PathBuf) {
    let token = portal_activation_token(widget, &path);
    match shrimply_cross_ui_core::desktop_open::prepare(&path, token.as_deref()) {
        Ok(shrimply_cross_ui_core::desktop_open::Action::Open(path)) => launch_file(widget, path),
        Ok(shrimply_cross_ui_core::desktop_open::Action::FocusRevealed(path)) => {
            activate_parent_folder_after_reveal(widget, path)
        }
        Err(error) => {
            tracing::warn!("file reveal failed path={}: {error}", path.display());
        }
    }
}

fn parent_window(widget: &gtk::Widget) -> Option<gtk::Window> {
    widget.root()?.downcast::<gtk::Window>().ok()
}

fn launch_file(widget: &gtk::Widget, path: PathBuf) {
    let path_display = path.display().to_string();
    let file = gio::File::for_path(path);
    let launcher = gtk::FileLauncher::new(Some(&file));
    let parent = parent_window(widget);
    launcher.launch(
        parent.as_ref(),
        None::<&gio::Cancellable>,
        move |result| match result {
            Ok(()) => tracing::info!("file manager opened path={path_display}"),
            Err(error) => tracing::warn!("file manager open failed path={path_display}: {error}"),
        },
    );
}

fn portal_activation_token(widget: &gtk::Widget, path: &Path) -> Option<glib::GString> {
    let context = widget.display().app_launch_context();
    context.set_timestamp(gdk::CURRENT_TIME);
    let files = [gio::File::for_path(path)];
    context.startup_notify_id(gio::AppInfo::NONE, &files)
}

fn activate_parent_folder_after_reveal(widget: &gtk::Widget, parent_dir: PathBuf) {
    let widget = widget.clone();
    glib::timeout_add_local_once(Duration::from_millis(120), move || {
        launch_file(&widget, parent_dir);
    });
}
