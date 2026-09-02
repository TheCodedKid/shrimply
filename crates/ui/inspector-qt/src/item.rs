use serde_json::Value;
use shrimply_inspector_core::item::{
    HeaderAction as SharedHeaderAction, HeaderButtonToggle as SharedHeaderButtonToggle,
    HeaderToggle as SharedHeaderToggle, InspectorItemPresentation,
};
use shrimply_preview_core::{PreviewFacetKey, PreviewTarget};
use uuid::Uuid;

use crate::section::InspectorSection;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InspectorAction {
    Reset {
        path: String,
        value: Value,
    },
    ResetFields {
        values: Vec<(String, Value)>,
    },
    ResetVideo {
        reset: shrimply_inspector_core::VideoReset,
    },
    SetBoolean {
        path: String,
        value: bool,
    },
    SetOptional {
        path: String,
        value: Option<Value>,
    },
    CopyArrayItem {
        path: String,
        index: usize,
    },
    MoveArrayItem {
        path: String,
        index: usize,
        offset: isize,
    },
    RemoveArrayItem {
        path: String,
        index: usize,
    },
    ResetAudioModifier {
        id: Uuid,
        effect: Value,
    },
    SetAudioModifierEnabled {
        id: Uuid,
        enabled: bool,
    },
    CopyAudioModifier {
        id: Uuid,
    },
    MoveAudioModifier {
        id: Uuid,
        offset: isize,
    },
    RemoveAudioModifier {
        id: Uuid,
    },
    ResetVisualModifier {
        id: Uuid,
        effect: Value,
    },
    SetVisualModifierEnabled {
        id: Uuid,
        enabled: bool,
    },
    CopyVisualModifier {
        id: Uuid,
    },
    MoveVisualModifier {
        id: Uuid,
        offset: isize,
    },
    RemoveVisualModifier {
        id: Uuid,
    },
    SetVisualModifierAlphaMask {
        id: Uuid,
        enabled: bool,
    },
    ToggleAudioCache {
        id: Uuid,
    },
    ReloadAsset {
        asset: String,
        kind: ReloadKind,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReloadKind {
    Blender,
    Manim,
}

pub(crate) type HeaderAction = SharedHeaderAction<InspectorAction>;
pub(crate) type HeaderToggle = SharedHeaderToggle<InspectorAction>;
pub(crate) type HeaderButtonToggle = SharedHeaderButtonToggle<InspectorAction>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InspectorItem {
    pub(crate) presentation: InspectorItemPresentation,
    pub(crate) section: InspectorSection,
    pub(crate) reset: Option<InspectorAction>,
    pub(crate) actions: Vec<HeaderAction>,
    pub(crate) toggle: Option<HeaderToggle>,
    pub(crate) button_toggle: Option<HeaderButtonToggle>,
}

impl InspectorItem {
    pub(crate) fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        section: InspectorSection,
    ) -> Self {
        Self {
            presentation: InspectorItemPresentation::new(key, title),
            section,
            reset: None,
            actions: Vec::new(),
            toggle: None,
            button_toggle: None,
        }
    }

    pub(crate) fn reset(mut self, reset: InspectorAction) -> Self {
        self.reset = Some(reset);
        self
    }

    pub(crate) fn actions(mut self, actions: Vec<HeaderAction>) -> Self {
        self.actions = actions;
        self
    }

    pub(crate) fn toggle(mut self, toggle: HeaderToggle) -> Self {
        self.toggle = Some(toggle);
        self
    }

    pub(crate) fn button_toggle(mut self, toggle: HeaderButtonToggle) -> Self {
        self.button_toggle = Some(toggle);
        self
    }

    pub(crate) fn preview_facet(mut self, facet: PreviewFacetKey) -> Self {
        self.presentation = self.presentation.preview_facet(facet);
        self
    }

    pub(crate) fn preview_target(mut self, target: PreviewTarget) -> Self {
        self.presentation = self.presentation.preview_target(target);
        self
    }

    pub(crate) fn boxed(self) -> InspectorListItem {
        InspectorListItem::Item(Box::new(self))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InspectorListItem {
    Item(Box<InspectorItem>),
    Flat(InspectorSection),
}
