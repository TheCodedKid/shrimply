pub mod backend;

pub fn init() {
    cxx_qt::init_crate!(shrimply_components_demo_qt);
    cxx_qt::init_qml_module!("dev.shrimply.components.demo");
}
