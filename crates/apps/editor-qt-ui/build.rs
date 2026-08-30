use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    unsafe {
        CxxQtBuilder::new_qml_module(
            QmlModule::new("dev.shrimply.editor").qml_file("qml/Main.qml"),
        )
        .files(["src/backend.rs"])
        .cpp_files(["include/gpu_surface.h", "src/gpu_surface.cpp"])
        .qt_module("Quick")
        .qt_module("OpenGL")
        .cc_builder(|build| {
            build.include("include");
        })
        .build();
    }
}
