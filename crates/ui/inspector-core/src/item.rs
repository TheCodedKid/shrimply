use shrimply_preview_core::{PreviewFacetKey, PreviewTarget};
use shrimply_project::project::{ITEM_PREVIEW_FACET, ItemAddress, VideoItem};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderAction<A> {
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub sensitive: bool,
    pub activate: A,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderToggle<A> {
    pub active: bool,
    pub tooltip: &'static str,
    pub activate: A,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderButtonToggle<A> {
    pub icon: &'static str,
    pub active: bool,
    pub tooltip: &'static str,
    pub activate: A,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorItemPresentation {
    pub key: String,
    pub title: String,
    pub preview_target: PreviewFocusTarget,
}

impl InspectorItemPresentation {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            preview_target: PreviewFocusTarget::facet(ITEM_PREVIEW_FACET),
        }
    }

    pub fn preview_facet(mut self, facet: PreviewFacetKey) -> Self {
        self.preview_target = PreviewFocusTarget::facet(facet);
        self
    }

    pub fn preview_target(mut self, target: PreviewTarget) -> Self {
        self.preview_target = PreviewFocusTarget::target(target);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewFocusTarget {
    owner_id: Option<Uuid>,
    facet: PreviewFacetKey,
}

impl PreviewFocusTarget {
    pub const fn facet(facet: PreviewFacetKey) -> Self {
        Self {
            owner_id: None,
            facet,
        }
    }

    pub const fn target(target: PreviewTarget) -> Self {
        Self {
            owner_id: Some(target.owner_id()),
            facet: target.facet(),
        }
    }

    pub fn resolve(self, item_id: Uuid) -> PreviewTarget {
        PreviewTarget::new(self.owner_id.unwrap_or(item_id), self.facet)
    }
}

pub fn valid_preview_focus(
    focused_item: &ItemAddress,
    focused_target: PreviewTarget,
    current_item: &ItemAddress,
    video: &VideoItem,
) -> bool {
    focused_item == current_item && video.owns_preview_target(focused_target)
}
