use super::media::ScopedUrl;
use block2::StackBlock;
use objc2::{DefinedClass, MainThreadOnly, define_class, rc::Retained, sel};
use objc2_app_kit::{
    NSAlert, NSAlertSecondButtonReturn, NSApplication, NSModalResponseOK, NSPopUpButton,
    NSSavePanel, NSWindow,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSURL, ns_string};
use objc2_uniform_type_identifiers::{UTType, UTTypeData};
use shrimply_cross_ui_core::{
    editor::{EditorSession, suggested_save_as_path},
    project_save::ProjectFormat,
};
use std::path::PathBuf;

struct SaveDialogIvars {
    panel: Retained<NSSavePanel>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SaveDialogIvars]
    struct SaveDialog;

    unsafe impl NSObjectProtocol for SaveDialog {}

    impl SaveDialog {
        #[unsafe(method(formatChanged:))]
        fn format_changed(&self, sender: &NSPopUpButton) {
            let format = ProjectFormat::ALL[sender.indexOfSelectedItem() as usize];
            let panel = &self.ivars().panel;
            let content_type = UTType::typeWithFilenameExtension_conformingToType(
                &NSString::from_str(format.extension()), unsafe { UTTypeData },
            ).expect("macOS must provide a project file type");
            panel.setAllowedContentTypes(&NSArray::from_retained_slice(&[content_type]));
            let name = format.normalize_path(PathBuf::from(panel.nameFieldStringValue().to_string()));
            panel.setNameFieldStringValue(&NSString::from_str(&name.to_string_lossy()));
        }
    }
);

pub(super) fn show(window: &NSWindow, session: &EditorSession) -> Result<(), String> {
    let mtm = window.mtm();
    let suggested = suggested_save_as_path();
    let format = ProjectFormat::from_path(&suggested);
    let panel = NSSavePanel::savePanel(mtm);
    panel.setTitle(Some(ns_string!("Save Project As")));
    panel.setNameFieldStringValue(&NSString::from_str(
        &suggested
            .file_name()
            .expect("save suggestion has a filename")
            .to_string_lossy(),
    ));
    if let Some(parent) = suggested.parent() {
        panel.setDirectoryURL(NSURL::from_directory_path(parent).as_deref());
    }
    panel.setCanCreateDirectories(true);
    panel.setAllowsOtherFileTypes(false);
    let (row, formats) = shrimply_components_appkit::export_dialog::popup_row(
        "Format",
        &ProjectFormat::ALL.map(ProjectFormat::label),
        0,
        mtm,
    );
    formats.selectItemAtIndex(
        ProjectFormat::ALL
            .iter()
            .position(|value| *value == format)
            .expect("known project format") as isize,
    );
    panel.setAccessoryView(Some(&row));
    let dialog = SaveDialog::alloc(mtm).set_ivars(SaveDialogIvars {
        panel: panel.clone(),
    });
    let dialog: Retained<SaveDialog> = unsafe { objc2::msg_send![super(dialog), init] };
    unsafe {
        formats.setTarget(Some(&dialog));
        formats.setAction(Some(sel!(formatChanged:)));
    }
    unsafe {
        let _: () = objc2::msg_send![&dialog, formatChanged: &*formats];
    }
    let completion = StackBlock::new(move |response| {
        NSApplication::sharedApplication(mtm).stopModalWithCode(response);
    });
    panel.beginSheetModalForWindow_completionHandler(window, &completion);
    if panel.runModal() != NSModalResponseOK {
        return Ok(());
    }
    let url = panel
        .URL()
        .ok_or("The save panel returned no destination.")?;
    let selected = url
        .to_file_path()
        .ok_or("Projects must be saved to a local file.")?;
    let _scope = ScopedUrl::new(url);
    let format = ProjectFormat::ALL[formats.indexOfSelectedItem() as usize];
    let path = format.normalize_path(selected.clone());
    if path != selected && path.exists() {
        let alert = NSAlert::new(mtm);
        alert.setMessageText(ns_string!("Replace existing project?"));
        alert.setInformativeText(&NSString::from_str(&path.to_string_lossy()));
        alert.addButtonWithTitle(ns_string!("Cancel"));
        alert.addButtonWithTitle(ns_string!("Replace"));
        if alert.runModal() != NSAlertSecondButtonReturn {
            return Ok(());
        }
    }
    session.save_as(path)?;
    Ok(())
}
