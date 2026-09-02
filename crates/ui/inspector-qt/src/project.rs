use shrimply_inspector_core::{InspectorDetail, ProjectPresentation};
use shrimply_math_core::{fraction_denominator, fraction_numerator};

use crate::item::InspectorListItem;
use crate::list::InspectorCategory;
use crate::section::{ControlKind, InspectorControl, InspectorSection};

pub(crate) fn categories(project: &ProjectPresentation) -> Vec<InspectorCategory> {
    let config = InspectorSection {
        controls: vec![
            InspectorControl::new(ControlKind::Text, "/name", "Name").value(&project.name),
            InspectorControl::new(ControlKind::ProjectSettings, "", "Project Settings").components(
                vec![
                    project.canvas_size.width.to_string(),
                    project.canvas_size.height.to_string(),
                    fraction_numerator(project.frame_rate).to_string(),
                    fraction_denominator(project.frame_rate).to_string(),
                ],
            ),
        ],
    };
    let details = [
        InspectorDetail {
            label: "Tracks",
            value: [
                track_count(
                    project.video_track_count,
                    "1 video track",
                    "%{count} video tracks",
                ),
                track_count(
                    project.audio_track_count,
                    "1 audio track",
                    "%{count} audio tracks",
                ),
                track_count(
                    project.caption_track_count,
                    "1 caption track",
                    "%{count} caption tracks",
                ),
            ]
            .join(", "),
        },
        InspectorDetail {
            label: "Duration",
            value: shrimply_project::time_format::project_duration(project.duration),
        },
        InspectorDetail {
            label: "Project File",
            value: project.file.to_string_lossy().into_owned(),
        },
    ];
    let performance = InspectorSection {
        controls: vec![
            InspectorControl::new(ControlKind::Performance, "", "Performance").read_only(),
        ],
    };
    vec![
        InspectorCategory {
            key: "config",
            label: "Project",
            icon: "sliders-horizontal-symbolic",
            items: vec![InspectorListItem::Flat(config)],
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: "info-outline-symbolic",
            items: vec![crate::info::item(&details)],
        },
        InspectorCategory {
            key: "performance",
            label: "Performance",
            icon: "speedometer-symbolic",
            items: vec![InspectorListItem::Flat(performance)],
        },
    ]
}

fn track_count(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        shrimply_i18n_qt::text(singular).to_string()
    } else {
        shrimply_i18n_qt::text_args(plural, &[("count", count.to_string())]).to_string()
    }
}
