use shrimply_inspector_core::InspectorTarget;
use shrimply_project::project::ItemAddress;

use crate::item::InspectorListItem;

pub(crate) use shrimply_inspector_core::list::InspectorListState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InspectorDocument {
    pub(crate) target: InspectorTarget,
    pub(crate) title: String,
    pub(crate) categories: Vec<InspectorCategory>,
    pub(crate) preview_item: Option<PreviewItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreviewItem {
    pub(crate) address: ItemAddress,
    pub(crate) id: uuid::Uuid,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InspectorCategory {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) items: Vec<InspectorListItem>,
}
