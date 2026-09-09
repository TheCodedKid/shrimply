use shrimply_caption_export_core::{CaptionFormat, ExportMode, ExportSettings, prepare_export, write_export};
use shrimply_math_core::{Fraction, Time};
use shrimply_project_document::{CaptionItem, CaptionTrack, Project, PreviewGuides, DEFAULT_CANVAS_SIZE, caption::CaptionEdgeStyle};
use std::path::Path;

fn main() -> Result<(), String> {
    let mut styled = CaptionItem::new(Time::ZERO, Time { seconds: Fraction::from(4u32) },
        "**Bold** *italic* __under__ & <tag>\n\n{1200}[漢/かん]字 \\{literal\\} \\\\N".into());
    styled.styling_enabled = true;
    styled.layout_enabled = true;
    styled.edge_style = CaptionEdgeStyle::Glow;
    let fractional = CaptionItem::new(Time { seconds: Fraction::new(10u32, 3u32) }, Time { seconds: Fraction::new(20u32, 3u32) }, "Fractional café 🦐".into());
    let mut project = Project {
        format_version: 1, name: "Caption inspection".into(), fps: Fraction::from(30u32), canvas_size: DEFAULT_CANVAS_SIZE,
        caption_tracks: vec![
            CaptionTrack { id: styled.id, enabled: true, language: None, items: vec![fractional.clone(), styled] },
            CaptionTrack { id: fractional.id, enabled: true, language: None, items: vec![CaptionItem::new(Time::ZERO, Time { seconds: Fraction::from(1u32) }, "Second track".into())] },
            CaptionTrack { id: fractional.id, enabled: false, language: None, items: vec![CaptionItem::new(Time::ZERO, Time { seconds: Fraction::from(1u32) }, "DISABLED".into())] },
        ],
        video_tracks: vec![], audio_tracks: vec![], folded_sequences: vec![], expanded_sequence_paths: vec![], cursor_position: None, timeline_zoom: None, preview_guides: Box::new(PreviewGuides::default()),
    };
    let folder = Path::new("target/caption-export-inspection");
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    for format in CaptionFormat::ALL {
        for mode in [ExportMode::Merge, ExportMode::Separate] {
            let files = prepare_export(&project, &folder.join("captions.wrong"), ExportSettings { format, mode })?;
            for path in write_export(files)? { println!("{}", path.display()); }
        }
    }
    project.caption_tracks[0].items[0].start = Time { seconds: Fraction::new(1u32, 3u32) };
    project.caption_tracks[0].items[0].end = Time { seconds: Fraction::new(1001u32, 3000u32) };
    for format in [CaptionFormat::Srt, CaptionFormat::Ass, CaptionFormat::Vtt] {
        write_export(prepare_export(&project, &folder.join("short"), ExportSettings { format, mode: ExportMode::Merge })?)?;
    }
    project.caption_tracks.clear();
    match prepare_export(&project, &folder.join("empty"), ExportSettings { format: CaptionFormat::Txt, mode: ExportMode::Merge }) {
        Ok(_) => return Err("Empty export unexpectedly succeeded".into()),
        Err(error) => println!("Empty export: {error}"),
    }
    Ok(())
}
