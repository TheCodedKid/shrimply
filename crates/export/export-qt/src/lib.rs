mod backend;

use shrimply_cross_ui_core::editor::EditorSession;

pub fn init() {
    cxx_qt::init_crate!(shrimply_export_qt);
    cxx_qt::init_qml_module!("dev.shrimply.export");
}

pub fn install(session: &EditorSession) {
    backend::install(session);
}
