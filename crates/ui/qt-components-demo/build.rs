use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    unsafe {
        CxxQtBuilder::new_qml_module(
            QmlModule::new("dev.shrimply.components.demo").qml_files(["qml/Showcase.qml"]),
        )
        .files(["src/backend.rs"])
        .qrc("qml/demo_assets.qrc")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .cc_builder(|build| {
            build.flag_if_supported("-Wno-sfinae-incomplete");
        })
        .build();
    }
}
