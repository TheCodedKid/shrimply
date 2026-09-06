#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    shrimply_support::crash::install();
    shrimply_support::diagnostics::init();
    let mut args = std::env::args_os().skip(1);
    let project = args.next().map(std::path::PathBuf::from);
    assert!(
        args.next().is_none(),
        "usage: shrimply-editor-appkit [PROJECT]"
    );
    if let Some(path) = &project {
        assert!(path.is_file(), "project does not exist: {}", path.display());
    }
    match macos::run(project.as_deref()) {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::from(
            shrimply_cross_ui_core::launcher::EDITOR_OPEN_CANCELED_EXIT_CODE,
        ),
        Err(()) => std::process::ExitCode::FAILURE,
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    panic!("shrimply-editor-appkit requires macOS");
}
