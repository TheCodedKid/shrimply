use super::*;
use shrimply_export_core::audio::Format;

const HEIGHT: f64 = 105.0;
const FORMATS: [(&str, Format); 5] = [
    ("WAV", Format::Wav),
    ("FLAC", Format::Flac),
    ("MP3", Format::Mp3),
    ("OGG Vorbis", Format::Ogg),
    ("Opus", Format::Opus),
];

pub fn choose_audio_format(parent: &NSWindow) -> Option<Format> {
    let mtm = parent.mtm();
    let sheet = export_dialog::new("Export Selected Audio", HEIGHT, mtm);
    let content = sheet
        .contentView()
        .expect("export dialog content installed");
    let choices = FORMATS.map(|(label, _)| label);
    let (format_row, format) = popup_row("Format", &choices, 0, mtm);
    format.selectItemAtIndex(0);
    content.addSubview(&format_row);

    if !export_dialog::run(parent, &sheet) {
        return None;
    }
    Some(FORMATS[format.indexOfSelectedItem() as usize].1)
}
