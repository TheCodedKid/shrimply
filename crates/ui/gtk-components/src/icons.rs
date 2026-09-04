use std::path::{Path, PathBuf};

pub fn register_bundled() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };
    gtk::IconTheme::for_display(&display).add_search_path(bundled_icons_dir());
}

fn bundled_icons_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let installed = bin_dir.join("../share/shrimply/icons");
            if installed.is_dir() {
                return installed;
            }
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../assets/icons")
}