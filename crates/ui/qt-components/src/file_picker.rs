use cxx_qt_lib::{QString, QUrl};
use std::path::{Path, PathBuf};

pub fn open(label: &str, title: &str, filter: &str) -> Option<PathBuf> {
    let initial = shrimply_component_core::file_picker::initial_folder(label)
        .map_or_else(QUrl::default, |path| local_url(&path));
    selected_path(
        label,
        shrimply_qt_helpers::open_file_dialog(
            &initial,
            &QString::from(title),
            &QString::from(filter),
        ),
    )
}

pub fn save(
    label: &str,
    suggested: &Path,
    title: &str,
    filter: &str,
    default_suffix: &str,
) -> Option<PathBuf> {
    let suggested = shrimply_component_core::file_picker::initial_folder(label)
        .and_then(|folder| suggested.file_name().map(|name| folder.join(name)))
        .unwrap_or_else(|| suggested.to_owned());
    let suggested = local_url(&suggested);
    selected_path(
        label,
        shrimply_qt_helpers::save_file_dialog(
            &suggested,
            &QString::from(title),
            &QString::from(filter),
            &QString::from(default_suffix),
        ),
    )
}

fn selected_path(label: &str, url: QUrl) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }
    let path = PathBuf::from(url.to_local_file()?.to_string());
    shrimply_component_core::file_picker::remember_file(label, &path);
    Some(path)
}

fn local_url(path: &Path) -> QUrl {
    QUrl::from_local_file(&QString::from(path.to_string_lossy().as_ref()))
}
