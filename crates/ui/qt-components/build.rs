use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    unsafe {
        CxxQtBuilder::new_qml_module(QmlModule::new("dev.shrimply.components").qml_files([
            "qml/ColorPicker.qml",
            "qml/Checkerboard.qml",
            "qml/CodeEditor.qml",
            "qml/ControlRow.qml",
            "qml/Dropdown.qml",
            "qml/FrameGraph.qml",
            "qml/MultilineTextInput.qml",
            "qml/NumberPicker.qml",
            "qml/Number2Picker.qml",
            "qml/Number3Picker.qml",
            "qml/PlaybackShortcuts.qml",
            "qml/ProgressButton.qml",
            "qml/ProjectSettingsSelector.qml",
            "qml/ReadOnlyField.qml",
            "qml/Selector.qml",
            "qml/SingleLineTextInput.qml",
            "qml/SplitButton.qml",
            "qml/SwitchRow.qml",
            "qml/Tabs.qml",
            "qml/TypoUnderline.qml",
            "qml/Showcase.qml",
        ]))
        .files(["src/backend.rs", "src/frame_graph.rs"])
        .qrc("qml/assets.qrc")
        .cpp_files([
            "include/drag_input.h",
            "src/drag_input.cpp",
            "include/frame_graph.h",
            "src/frame_graph.cpp",
        ])
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .qt_module("OpenGL")
        .cc_builder(|build| {
            build.include("include");
        })
        .build();
    }
}
