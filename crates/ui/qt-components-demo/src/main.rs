use cxx_qt_lib::{QQmlApplicationEngine, QString, QUrl};
use std::process::ExitCode;

fn main() -> ExitCode {
    shrimply_support::diagnostics::init();
    shrimply_qt_components::i18n::init_system_locale();
    shrimply_qt_components::init();

    let mut app = shrimply_qt_helpers::new_widget_application();
    let Some(mut app) = app.as_mut() else {
        eprintln!("could not create Qt application");
        return ExitCode::FAILURE;
    };
    app.as_mut()
        .set_application_name(&QString::from("shrimply-qt-components-demo"));
    app.as_mut()
        .set_application_display_name(&QString::from("Shrimply Qt Components"));

    let mut engine = QQmlApplicationEngine::new();
    let Some(mut engine) = engine.as_mut() else {
        eprintln!("could not create QML engine");
        return ExitCode::FAILURE;
    };
    cxx_qt::init_crate!(shrimply_qt_components_demo);
    cxx_qt::init_qml_module!("dev.shrimply.components.demo");
    let failed = engine.as_mut().on_object_creation_failed(|_, url| {
        eprintln!("could not load Qt component showcase: {url}");
        std::process::exit(1);
    });
    engine.as_mut().load(&QUrl::from(
        "qrc:/qt/qml/dev/shrimply/components/demo/qml/Showcase.qml",
    ));
    let status = app.exec();
    drop(failed);
    ExitCode::from(status.clamp(0, 255) as u8)
}
