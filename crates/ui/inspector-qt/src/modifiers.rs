use shrimply_inspector_core::VisualModifierPresentation;
use shrimply_preview_core::PreviewTarget;
use shrimply_video_modifiers::MODIFIER_PREVIEW_FACET;

use crate::item::{
    HeaderAction, HeaderButtonToggle, HeaderToggle, InspectorAction, InspectorItem,
    InspectorListItem,
};
use crate::section::InspectorSection;

mod opacity;
mod transform;

pub(crate) fn items(
    modifiers: &[VisualModifierPresentation],
) -> impl Iterator<Item = InspectorListItem> + '_ {
    modifiers.iter().map(item)
}

fn item(modifier: &VisualModifierPresentation) -> InspectorListItem {
    let id = modifier.id;
    let mut section = modifier
        .body
        .as_ref()
        .map_or_else(InspectorSection::default, |body| match body {
            shrimply_inspector_core::VisualModifierBodyPresentation::Opacity(value) => {
                opacity::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Transform(value) => {
                transform::section(value)
            }
        });
    if let Some(mask) = &modifier.alpha_mask {
        section
            .controls
            .extend(mask.section.controls.iter().cloned());
    }
    let mut item = InspectorItem::new(format!("modifier:{id}"), modifier.title, section)
        .reset(InspectorAction::ResetVisualModifier {
            id,
            effect: modifier.default_effect.clone(),
        })
        .toggle(HeaderToggle {
            active: modifier.enabled,
            tooltip: if modifier.enabled {
                "Disable modifier"
            } else {
                "Enable modifier"
            },
            activate: InspectorAction::SetVisualModifierEnabled {
                id,
                enabled: !modifier.enabled,
            },
        })
        .actions(vec![
            HeaderAction {
                icon: "edit-copy-symbolic",
                tooltip: "Copy",
                sensitive: true,
                activate: InspectorAction::CopyVisualModifier { id },
            },
            HeaderAction {
                icon: "go-up-symbolic",
                tooltip: "Move up",
                sensitive: modifier.can_move_up,
                activate: InspectorAction::MoveVisualModifier { id, offset: -1 },
            },
            HeaderAction {
                icon: "go-down-symbolic",
                tooltip: "Move down",
                sensitive: modifier.can_move_down,
                activate: InspectorAction::MoveVisualModifier { id, offset: 1 },
            },
            HeaderAction {
                icon: "user-trash-symbolic",
                tooltip: "Remove",
                sensitive: modifier.can_remove,
                activate: InspectorAction::RemoveVisualModifier { id },
            },
        ])
        .preview_target(PreviewTarget::new(id, MODIFIER_PREVIEW_FACET));
    if let Some(mask) = &modifier.alpha_mask {
        item = item.button_toggle(HeaderButtonToggle {
            icon: "select-symbolic",
            active: mask.active,
            tooltip: "Mask",
            activate: InspectorAction::SetVisualModifierAlphaMask {
                id,
                enabled: !mask.active,
            },
        });
    }
    item.boxed()
}
