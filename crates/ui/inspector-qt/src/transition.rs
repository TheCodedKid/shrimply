use shrimply_inspector_core::TransitionPresentation;

use crate::item::InspectorItem;
use crate::list::InspectorCategory;

pub(crate) fn categories(transition: &TransitionPresentation) -> Vec<InspectorCategory> {
    vec![InspectorCategory {
        key: "transition",
        label: transition.title,
        icon: "media-playback-start",
        items: vec![
            InspectorItem::new("transition", transition.title, transition.section()).boxed(),
        ],
    }]
}
