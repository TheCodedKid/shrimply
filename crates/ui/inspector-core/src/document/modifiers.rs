use super::{BasicInspectorAction as Action, InspectorItem, InspectorListItem};
use crate::item::{HeaderAction, HeaderToggle};
use crate::{ControlKind, InspectorControl, InspectorSection, VisualModifierPresentation};

pub(super) fn visual(modifier: &VisualModifierPresentation) -> InspectorListItem {
    let id = modifier.id;
    let mut section = modifier
        .body
        .as_ref()
        .map_or_else(InspectorSection::default, |body| body.section());
    if let Some(mask) = &modifier.alpha_mask {
        section
            .controls
            .extend(mask.section.controls.iter().cloned());
    }
    let mut item = InspectorItem::new(format!("modifier:{id}"), modifier.title, section)
        .reset(Action::ResetModifier { id, audio: false })
        .toggle(HeaderToggle {
            active: modifier.enabled,
            tooltip: "Enable modifier",
            activate: Action::SetModifierEnabled {
                id,
                enabled: !modifier.enabled,
                audio: false,
            },
        });
    item.presentation =
        item.presentation
            .preview_target(shrimply_preview_core::PreviewTarget::new(
                id,
                shrimply_video_modifiers::MODIFIER_PREVIEW_FACET,
            ));
    item.actions = chain_actions(
        id,
        false,
        modifier.can_move_up,
        modifier.can_move_down,
        modifier.can_remove,
    );
    if let Some(mask) = &modifier.alpha_mask {
        item.actions.push(HeaderAction {
            icon: "select-symbolic",
            tooltip: "Mask",
            sensitive: true,
            activate: Action::SetAlphaMask {
                target: shrimply_project::project::VisualAlphaMaskTarget::Modifier(id),
                enabled: !mask.active,
            },
        });
    }
    item.boxed()
}

pub(super) fn chain_actions(
    id: uuid::Uuid,
    audio: bool,
    up: bool,
    down: bool,
    remove: bool,
) -> Vec<HeaderAction<Action>> {
    vec![
        HeaderAction {
            icon: "edit-copy-symbolic",
            tooltip: "Copy",
            sensitive: true,
            activate: Action::CopyModifier { id, audio },
        },
        HeaderAction {
            icon: "go-up-symbolic",
            tooltip: "Move up",
            sensitive: up,
            activate: Action::MoveModifier {
                id,
                offset: -1,
                audio,
            },
        },
        HeaderAction {
            icon: "go-down-symbolic",
            tooltip: "Move down",
            sensitive: down,
            activate: Action::MoveModifier {
                id,
                offset: 1,
                audio,
            },
        },
        HeaderAction {
            icon: "user-trash-symbolic",
            tooltip: "Remove",
            sensitive: remove,
            activate: Action::RemoveModifier { id, audio },
        },
    ]
}

pub(super) fn menu(
    kind: ControlKind,
    choices: impl Iterator<Item = (String, String, String)>,
) -> InspectorListItem {
    let choices = choices.collect::<Vec<_>>();
    InspectorListItem::Flat(InspectorSection {
        controls: vec![
            InspectorControl::new(kind, "", "")
                .choices(
                    choices.iter().map(|c| c.0.clone()).collect(),
                    choices.iter().map(|c| c.1.clone()).collect(),
                )
                .choice_search_terms(choices.into_iter().map(|c| c.2).collect()),
        ],
    })
}
