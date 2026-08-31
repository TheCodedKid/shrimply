pub mod backend;

pub fn init() {
    cxx_qt::init_crate!(shrimply_qt_components_demo);
    cxx_qt::init_qml_module!("dev.shrimply.components.demo");
}
