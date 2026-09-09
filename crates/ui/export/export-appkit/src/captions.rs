use super::*;
use shrimply_caption_export_core::{CaptionFormat, ExportMode, ExportSettings};

const CAPTION_OPTIONS_HEIGHT: f64 = 135.0;

pub fn choose_caption_settings(parent: &NSWindow) -> Option<ExportSettings> {
    let mtm = parent.mtm();
    let sheet = export_dialog::new("Export Captions", CAPTION_OPTIONS_HEIGHT, mtm);
    let content = sheet.contentView().expect("export dialog content installed");
    let labels = CaptionFormat::ALL.map(CaptionFormat::label);
    let (format_row, format) = popup_row("Format", &labels, 1, mtm);
    format.selectItemAtIndex(0);
    content.addSubview(&format_row);
    let (mode_row, mode) = popup_row(
        "Tracks",
        &["Merge into one file", "Export each track separately"],
        0,
        mtm,
    );
    mode.selectItemAtIndex(0);
    content.addSubview(&mode_row);
    if !export_dialog::run(parent, &sheet) {
        return None;
    }
    Some(ExportSettings {
        format: CaptionFormat::ALL[format.indexOfSelectedItem() as usize],
        mode: if mode.indexOfSelectedItem() == 0 {
            ExportMode::Merge
        } else {
            ExportMode::Separate
        },
    })
}
