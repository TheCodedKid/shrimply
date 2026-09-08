use std::path::PathBuf;

pub fn project_file_filter() -> String {
    let label = shrimply_i18n_qt::text("Shrimply projects").to_string();
    shrimply_cross_ui_core::launcher::project_file_name_filter(&label)
}

pub fn open_project() -> Option<PathBuf> {
    crate::file_picker::open(
        "Open Project",
        &shrimply_i18n_qt::text("Open Project").to_string(),
        &project_file_filter(),
    )
}
