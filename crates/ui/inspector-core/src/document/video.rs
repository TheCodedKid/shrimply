use crate::{InspectorDetail, VideoPresentation};

use super::{
    BasicInspectorAction, CategoryIcon, InspectorCategory, InspectorItem, InspectorListItem,
    detail_item,
};

pub(super) fn categories(
    video: &VideoPresentation,
    details: &[InspectorDetail],
) -> Vec<InspectorCategory> {
    vec![
        InspectorCategory {
            key: "visual",
            label: "Visual",
            icon: CategoryIcon::Visual,
            items: video
                .visual
                .iter()
                .cloned()
                .map(item)
                .chain(video.modifiers.iter().map(super::modifiers::visual))
                .chain(std::iter::once(super::modifiers::menu(
                    crate::ControlKind::VisualModifierMenu,
                    video
                        .modifier_choices
                        .iter()
                        .map(|c| (c.key.clone(), c.label.to_string(), c.search_text.clone())),
                )))
                .collect(),
        },
        InspectorCategory {
            key: "playback",
            label: "Playback",
            icon: CategoryIcon::Playback,
            items: video.playback.iter().cloned().map(item).collect(),
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: CategoryIcon::Info,
            items: vec![detail_item(details)],
        },
    ]
}

fn item(card: crate::VideoCard) -> InspectorListItem {
    let mut item = InspectorItem::new(card.key, card.title, card.section);
    item.actions = card
        .actions
        .into_iter()
        .map(|action| crate::item::HeaderAction {
            icon: action.icon,
            tooltip: action.tooltip,
            sensitive: action.sensitive,
            activate: BasicInspectorAction::Video(action.activate),
        })
        .collect();
    if let Some(mask) = &card.alpha_mask {
        item = item.alpha_mask(
            shrimply_project::project::VisualAlphaMaskTarget::Compositing,
            mask,
        );
    }
    if let Some(reset) = card.reset {
        item = item.reset(BasicInspectorAction::ResetVideo(reset));
    }
    if let Some(facet) = card.preview_facet {
        item = item.preview_facet(facet);
    }
    item.boxed()
}
