use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    unsafe {
        CxxQtBuilder::new_qml_module(
            QmlModule::new("dev.shrimply.export").qml_file("qml/ExportWindow.qml"),
        )
        .files(["src/backend.rs"])
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .cc_builder(|build| {
            build.flag_if_supported("-Wno-sfinae-incomplete");
        })
        .build();
    }
}
