use cxx_qt_lib::{QQmlApplicationEngine, QString, QUrl};
use std::process::ExitCode;

fn main() -> ExitCode {
    shrimply_process_reporting::diagnostics::init();
    shrimply_components_qt::i18n::init_system_locale();
    shrimply_components_qt::init();
    std::thread::spawn(|| {
        loop {
            let measurement = shrimply_profiling::measure("Demo / Background refresh");
            shrimply_profiling::increment("Demo / Refresh count");
            drop(measurement);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let mut app = shrimply_application_qt::new_widget_application();
    let Some(mut app) = app.as_mut() else {
        eprintln!("could not create Qt application");
        return ExitCode::FAILURE;
    };
    app.as_mut()
        .set_application_name(&QString::from("shrimply-components-demo-qt"));
    app.as_mut()
        .set_application_display_name(&QString::from("Shrimply Qt Components"));

    let mut engine = QQmlApplicationEngine::new();
    let Some(mut engine) = engine.as_mut() else {
        eprintln!("could not create QML engine");
        return ExitCode::FAILURE;
    };
    shrimply_components_demo_qt::init();
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
