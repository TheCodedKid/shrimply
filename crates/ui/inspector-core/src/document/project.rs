use crate::{ControlKind, InspectorControl, InspectorSection, ProjectPresentation};

use super::{CategoryIcon, InspectorCategory, InspectorListItem, detail_item};

pub(super) fn categories(project: &ProjectPresentation) -> Vec<InspectorCategory> {
    let config = InspectorSection {
        controls: vec![
            InspectorControl::new(ControlKind::Text, "/name", "Name").value(&project.name),
            InspectorControl::new(ControlKind::ProjectSettings, "", "Project Settings").components(
                vec![
                    project.canvas_size.width.to_string(),
                    project.canvas_size.height.to_string(),
                    shrimply_math_core::fraction_numerator(project.frame_rate).to_string(),
                    shrimply_math_core::fraction_denominator(project.frame_rate).to_string(),
                ],
            ),
        ],
    };
    vec![
        InspectorCategory {
            key: "project",
            label: "Project",
            icon: CategoryIcon::Project,
            items: vec![InspectorListItem::Flat(config)],
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: CategoryIcon::Info,
            items: vec![detail_item(&project.details())],
        },
        InspectorCategory {
            key: "performance",
            label: "Performance",
            icon: CategoryIcon::Performance,
            items: vec![InspectorListItem::Flat(crate::benchmarking::section())],
        },
    ]
}
