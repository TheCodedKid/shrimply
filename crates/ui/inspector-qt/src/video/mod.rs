use shrimply_inspector_core::{VideoCard, VideoPresentation};

use crate::item::{HeaderButtonToggle, InspectorAction, InspectorItem, InspectorListItem};
use crate::list::InspectorCategory;
use crate::section::InspectorSection;

mod playback;

pub(crate) fn categories(
    video: &VideoPresentation,
    details: &[shrimply_inspector_core::InspectorDetail],
    metadata: Option<crate::MediaMetadataState>,
    can_paste_modifiers: bool,
) -> Vec<InspectorCategory> {
    let mut visual_items = video.visual.iter().cloned().map(item).collect::<Vec<_>>();
    visual_items.extend(crate::modifiers::items(&video.modifiers));
    let mut modifier_menu = InspectorSection::default();
    modifier_menu.add(
        shrimply_inspector_core::InspectorControl::new(
            shrimply_inspector_core::ControlKind::VisualModifierMenu,
            "",
            "",
        )
        .value(can_paste_modifiers.to_string())
        .choices(
            video
                .modifier_choices
                .iter()
                .map(|choice| choice.key.clone())
                .collect(),
            video
                .modifier_choices
                .iter()
                .map(|choice| choice.label.to_string())
                .collect(),
        )
        .choice_search_terms(
            video
                .modifier_choices
                .iter()
                .map(|choice| choice.search_text.clone())
                .collect(),
        ),
    );
    visual_items.push(InspectorListItem::Flat(modifier_menu));
    vec![
        InspectorCategory {
            key: "visual",
            label: "Visual",
            icon: "blend-tool-symbolic",
            items: visual_items,
        },
        InspectorCategory {
            key: "playback",
            label: "Playback",
            icon: "playback-speed-symbolic",
            items: video.playback.iter().cloned().map(playback::item).collect(),
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: "info-outline-symbolic",
            items: vec![info_item(video, details, metadata.as_ref())],
        },
    ]
}

pub(super) fn item(card: VideoCard) -> InspectorListItem {
    let mut section = card.section;
    if let Some(mask) = &card.alpha_mask {
        section
            .controls
            .extend(mask.section.controls.iter().cloned());
    }
    let mut item = InspectorItem::new(card.key, card.title, section);
    if let Some(reset) = card.reset {
        item = item.reset(InspectorAction::ResetVideo { reset });
    }
    if let Some(mask) = card.alpha_mask {
        item = item.button_toggle(HeaderButtonToggle {
            icon: "select-symbolic",
            active: mask.active,
            tooltip: "Mask",
            activate: InspectorAction::SetAlphaMask {
                target: shrimply_project::project::VisualAlphaMaskTarget::Compositing,
                enabled: !mask.active,
            },
        });
    }
    if let Some(facet) = card.preview_facet {
        item = item.preview_facet(facet);
    }
    item.boxed()
}

fn video_stream_choice(stream: u32) -> (String, String) {
    (
        stream.to_string(),
        shrimply_i18n_qt::text_args(
            "Video stream %{number}",
            &[("number", (stream + 1).to_string())],
        )
        .to_string(),
    )
}

fn info_item(
    video: &VideoPresentation,
    details: &[shrimply_inspector_core::InspectorDetail],
    metadata: Option<&crate::MediaMetadataState>,
) -> InspectorListItem {
    let mut section = InspectorSection::default();
    let video_stream_count = match metadata {
        Some(crate::MediaMetadataState::Ready(metadata)) => metadata.video_stream_count,
        _ => 0,
    };
    if let Some(stream) = video.stream
        && video_stream_count > 1
    {
        section.add(shrimply_inspector_core::selector::selector(
            "/track_id",
            "Video Stream",
            stream.selected.min(video_stream_count - 1).to_string(),
            (0..video_stream_count).map(video_stream_choice),
        ));
        section.add(shrimply_inspector_core::selector::optional_number_selector(
            "/alpha_mask_video",
            "Alpha Mask Stream",
            stream.alpha_mask.filter(|value| *value != stream.selected),
            (0..video_stream_count)
                .filter(|value| *value != stream.selected)
                .map(video_stream_choice),
        ));
    }
    crate::info::append(&mut section, details, metadata, video.source_metadata);
    InspectorListItem::Flat(section)
}

pub(crate) fn reload_blender(asset: &str) -> Result<(), String> {
    let asset = shrimply_project::project::Asset::from(std::path::Path::new(asset));
    shrimply_blender::invalidate_metadata(asset.path());
    asset
        .mark_dirty()
        .map_err(|error| format!("could not mark Blender source dirty: {error}"))
}

pub(crate) fn reload_manim(asset: &str) -> Result<(), String> {
    let asset = shrimply_project::project::Asset::from(std::path::Path::new(asset));
    shrimply_manim_parser::invalidate_ir_cache(&asset)
        .map_err(|error| format!("could not invalidate Manim source: {error}"))?;
    asset
        .mark_dirty()
        .map_err(|error| format!("could not mark Manim source dirty: {error}"))
}
