use shrimply_inspector_core::TrackPresentation;
use shrimply_project::project::ItemKind;

use crate::item::InspectorListItem;
use crate::list::InspectorCategory;
use crate::section::{ControlKind, InspectorControl, InspectorSection};

pub(crate) fn categories(track: &TrackPresentation) -> Vec<InspectorCategory> {
    let mut controls = vec![
        InspectorControl::new(ControlKind::Boolean, "/enabled", "Enabled")
            .value(track.enabled.to_string())
            .subtitle("Include this track in playback and export"),
    ];

    if track.kind == ItemKind::Caption {
        controls.push(shrimply_inspector_core::selector::optional_selector(
            "/language",
            "Language",
            track.language.as_deref(),
            shrimply_project::project::caption_languages()
                .iter()
                .cloned()
                .map(|language| (language.clone(), language)),
        ));
    }

    vec![
        InspectorCategory {
            key: "track",
            label: "Track",
            icon: "sliders-horizontal-symbolic",
            items: vec![InspectorListItem::Flat(InspectorSection { controls })],
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: "info-outline-symbolic",
            items: vec![crate::info::item(&track.details())],
        },
    ]
}
