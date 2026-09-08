#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
fn main() {
    shrimply_process_reporting::diagnostics::init();
    macos::run();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    panic!("shrimply-components-demo-appkit requires macOS");
}
