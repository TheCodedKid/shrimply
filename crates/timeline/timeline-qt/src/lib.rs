pub use shrimply_cross_ui_tl::{
    ContextMenu, ContextMenuAction, ContextMenuControl, ContextMenuEntry, ContextMenuItem,
    ContextMenuRequest, CursorTool, DragCollisionMode, TIMELINE_CLIPBOARD_MARKER,
    VideoFrameSelection,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuEntry {
    Separator,
    Action(ContextMenuItem),
    Control(ContextMenuControl),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuModel {
    entries: Vec<MenuEntry>,
}

impl MenuModel {
    pub fn new(contract: &ContextMenu) -> Self {
        let mut entries = Vec::new();
        for section in &contract.sections {
            if !entries.is_empty() && !section.is_empty() {
                entries.push(MenuEntry::Separator);
            }
            entries.extend(section.iter().map(|entry| match entry {
                ContextMenuEntry::Action(item) => MenuEntry::Action(*item),
                ContextMenuEntry::Control(control) => MenuEntry::Control(*control),
            }));
        }
        Self { entries }
    }

    pub fn entries(&self) -> &[MenuEntry] {
        &self.entries
    }

    pub fn action(&self, index: usize) -> Option<ContextMenuAction> {
        match self.entries.get(index) {
            Some(MenuEntry::Action(item)) if item.enabled => Some(item.action),
            _ => None,
        }
    }

    pub fn control(&self, index: usize) -> Option<ContextMenuControl> {
        match self.entries.get(index) {
            Some(MenuEntry::Control(control)) => Some(*control),
            _ => None,
        }
    }
}
