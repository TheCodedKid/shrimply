#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    let mut args = std::env::args_os().skip(1);
    if let Some(path) = args.next() {
        if args.next().is_some() {
            eprintln!("usage: shrimply-appkit [PROJECT]");
            return std::process::ExitCode::FAILURE;
        }
        let mtm = objc2_foundation::MainThreadMarker::new()
            .expect("AppKit must start on the main thread");
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        assert!(app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory));
        return match shrimply_cross_ui_core::launcher::launch_appkit_editor(std::path::Path::new(
            &path,
        ))
        .and_then(|mut child| child.wait().map_err(|error| error.to_string()))
        {
            Ok(status) if status.success() => std::process::ExitCode::SUCCESS,
            Ok(status) => {
                eprintln!("editor exited with {status}");
                std::process::ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    macos::run();
    std::process::ExitCode::SUCCESS
}

#[cfg(not(target_os = "macos"))]
fn main() {
    panic!("shrimply-appkit requires macOS");
}
