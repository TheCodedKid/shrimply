use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    unsafe {
        CxxQtBuilder::new_qml_module(
            QmlModule::new("dev.shrimply.inspector")
                .qml_file("qml/InspectorView.qml")
                .qml_file("qml/TtsEditor.qml"),
        )
        .files(["src/audio.rs", "src/backend.rs", "src/info.rs"])
        .cpp_files(["include/inspector_locale.h", "src/locale.cpp"])
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .cc_builder(|build| {
            build.include("include");
            build.flag_if_supported("-Wno-sfinae-incomplete");
        })
        .build();
    }
}
