use shrimply_video::modifier_cache::{self, Status};
use shrimply_video_modifiers::cache::CacheModifier;

use crate::{ControlKind, InspectorControl, InspectorRuntime, InspectorSection};

pub(super) fn presentation(
    value: &CacheModifier,
    index: usize,
    id: uuid::Uuid,
    _runtime: InspectorRuntime,
) -> InspectorSection {
    let base = format!("/modifiers/{index}/effect/effect/config");
    let status = modifier_cache::status(id);
    let baking = matches!(status, Status::Baking { .. });
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            format!("{base}/quality"),
            "Format",
            serde_json::to_value(value.quality)
                .expect("cache quality must serialize")
                .as_str()
                .expect("cache quality must serialize as text"),
            [
                ("compact".into(), "H.265 · Compact".into()),
                ("balanced".into(), "H.265 · Balanced".into()),
                ("high".into(), "H.265 · High".into()),
                ("lossless".into(), "H.265 · Lossless".into()),
            ],
        )
        .sensitive(!baking)
        .immediate_commit("visual-cache-format"),
    );
    let (label, progress, tooltip) = match status {
        Status::Missing => ("Bake", 0.0, String::new()),
        Status::Baking { completed, total } => (
            "Baking…",
            if total == 0 {
                -1.0
            } else {
                completed as f64 / total as f64
            },
            String::new(),
        ),
        Status::Ready => ("Rebake", 0.0, String::new()),
        Status::Failed(error) => ("Bake", 0.0, error),
    };
    section.add(
        InspectorControl::new(ControlKind::AudioCache, format!("{base}/bake"), "")
            .value(label)
            .components(vec![progress.to_string()])
            .tooltip(tooltip)
            .target(id),
    );
    section
}
