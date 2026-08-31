use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

static LAST_FOLDERS: LazyLock<Mutex<HashMap<String, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn initial_folder(label: &str) -> Option<PathBuf> {
    LAST_FOLDERS
        .lock()
        .expect("file picker cache lock poisoned")
        .get(label)
        .cloned()
}

pub fn remember_file(label: &str, path: &Path) {
    let Some(folder) = path.parent() else {
        return;
    };
    LAST_FOLDERS
        .lock()
        .expect("file picker cache lock poisoned")
        .insert(label.to_owned(), folder.to_owned());
}
