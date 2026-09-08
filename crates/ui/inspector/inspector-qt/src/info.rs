use std::sync::Arc;

use base64::Engine;
use shrimply_inspector_core::InspectorDetail;

use crate::item::InspectorListItem;
use crate::section::{ControlKind, InspectorControl, InspectorSection};
use crate::{CachedMediaInfo, MediaMetadataState};

#[cxx::bridge]
mod locale {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("inspector_locale.h");
        #[rust_name = "format_local_date_time"]
        fn formatLocalDateTime(seconds: i64) -> QString;
    }
}

pub(crate) fn item(details: &[InspectorDetail]) -> InspectorListItem {
    media_item(
        details,
        None,
        shrimply_inspector_core::info::SourceMetadata::None,
    )
}

pub(crate) fn media_item(
    details: &[InspectorDetail],
    metadata: Option<&MediaMetadataState>,
    selected: shrimply_inspector_core::info::SourceMetadata,
) -> InspectorListItem {
    let mut section = InspectorSection::default();
    append(&mut section, details, metadata, selected);
    InspectorListItem::Flat(section)
}

pub(crate) fn cache_media_info(info: Arc<shrimply_media_info::FileInfo>) -> Arc<CachedMediaInfo> {
    let presentation = shrimply_inspector_core::info::media_info_presentation(&info, format_date);
    let artwork_url = presentation.artwork.as_ref().map(|artwork| {
        format!(
            "data:{};base64,{}",
            artwork
                .mime_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            base64::engine::general_purpose::STANDARD.encode(&artwork.data)
        )
    });
    let audio_stream_count = info
        .streams
        .iter()
        .filter(|stream| stream.kind == "audio")
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    let video_stream_count = info
        .streams
        .iter()
        .filter(|stream| stream.kind == "video")
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    Arc::new(CachedMediaInfo {
        presentation,
        artwork_url,
        audio_stream_count,
        video_stream_count,
    })
}

pub(crate) fn append(
    section: &mut InspectorSection,
    details: &[InspectorDetail],
    metadata: Option<&MediaMetadataState>,
    selected: shrimply_inspector_core::info::SourceMetadata,
) {
    for detail in details {
        let control = InspectorControl::new(
            if matches!(detail.label, "File Location" | "Project File") {
                ControlKind::FileLocation
            } else {
                ControlKind::ReadOnly
            },
            "",
            detail.label,
        )
        .value(&detail.value)
        .read_only();
        section.add(if detail.label == "Project File" {
            control.tooltip("Show project file in folder")
        } else if detail.label == "File Location" {
            control.tooltip("Show in folder")
        } else {
            control
        });
    }
    match metadata {
        None => {}
        Some(MediaMetadataState::Loading) => section.add(
            InspectorControl::new(ControlKind::InfoLoading, "", "File Metadata")
                .value("Loading…")
                .read_only(),
        ),
        Some(MediaMetadataState::Failed(error)) => {
            section.add(InspectorControl::new(
                ControlKind::InfoHeading,
                "",
                "Diagnostics",
            ));
            section.add(
                InspectorControl::new(ControlKind::ReadOnly, "", "Metadata")
                    .value(error)
                    .read_only(),
            );
        }
        Some(MediaMetadataState::Ready(metadata)) => {
            append_presentation(section, metadata, selected)
        }
    }
}

fn append_presentation(
    section: &mut InspectorSection,
    metadata: &CachedMediaInfo,
    selected: shrimply_inspector_core::info::SourceMetadata,
) {
    if let Some(artwork_url) = &metadata.artwork_url {
        section.add(
            InspectorControl::new(ControlKind::InfoArtwork, "", "Artwork")
                .value(artwork_url)
                .read_only(),
        );
    }
    for group in &metadata.presentation.groups {
        let title = group.display_title(selected);
        section.add(InspectorControl::new(ControlKind::InfoHeading, "", title));
        for row in &group.rows {
            section.add(
                InspectorControl::new(ControlKind::ReadOnly, "", &row.label)
                    .value(&row.value)
                    .read_only(),
            );
        }
    }
}

fn format_date(seconds: i64) -> Option<String> {
    let value = locale::format_local_date_time(seconds).to_string();
    (!value.is_empty()).then_some(value)
}
