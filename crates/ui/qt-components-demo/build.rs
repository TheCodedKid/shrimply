use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("dev.shrimply.components.demo").qml_files(["qml/Showcase.qml"]),
    )
        .qrc("qml/assets.qrc")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .build();
}
