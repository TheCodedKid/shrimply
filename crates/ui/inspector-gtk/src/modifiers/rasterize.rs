use crate::InspectorContext;
use gtk::prelude::*;
use shrimply_core::{VideoSampleMethod, timeline_value::TimelineStep};
use shrimply_inspector_core::{ControlKind, InspectorTarget, NumberSpec};
use shrimply_video_modifiers::rasterize::RasterizeModifier;
use uuid::Uuid;

use super::vec_row;

pub fn add_rows(value: &RasterizeModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    let key = context
        .selected_item
        .clone()
        .expect("Rasterize inspector must have a selected item");
    let section = context
        .inspector_core
        .rasterize_presentation(&InspectorTarget::Item(key), id)
        .expect("live Rasterize modifier must have a shared presentation");
    let [size, sample_method] = section
        .controls
        .try_into()
        .expect("Rasterize presentation must contain exactly two controls");

    assert_eq!(size.kind, ControlKind::LayeredVector2);
    assert!(size.path.ends_with("/effect/effect/size"));
    assert_eq!(size.target_id, Some(id));
    assert_eq!(size.timeline_id, Some(value.size().id));
    assert_eq!(
        size.number,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        }
    );
    assert_eq!(size.width_characters, 7);
    assert_eq!(size.prefixes, ["X", "Y"]);
    assert!(!size.lock);
    assert_eq!(size.commit_name, "visual-modifier-vector");
    let size_row = vec_row(&size.label, value.size(), id, false, None, context);
    size_row.set_sensitive(size.sensitive);
    if size.visible {
        out.append(&size_row);
    }

    assert_eq!(sample_method.kind, ControlKind::LayeredSelector);
    assert!(sample_method.path.ends_with("/effect/effect/sample_method"));
    assert_eq!(sample_method.target_id, Some(id));
    assert_eq!(sample_method.timeline_id, Some(value.sample_method.id));
    assert_eq!(
        sample_method.commit_name,
        shrimply_inspector_core::visual_modifiers::RASTERIZE_SAMPLE_METHOD_COMMIT,
    );
    assert_eq!(
        sample_method.values,
        VideoSampleMethod::variants()
            .iter()
            .map(|variant| variant.key.to_string())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        sample_method.labels,
        VideoSampleMethod::variants()
            .iter()
            .map(|variant| variant.label.to_string())
            .collect::<Vec<_>>(),
    );
    assert!(sample_method.icons.is_empty());
    let sample_method_row = super::step_row(
        &sample_method.label,
        &value.sample_method,
        context,
        super::ModifierStepTarget {
            modifier_id: id,
            commit_name: shrimply_inspector_core::visual_modifiers::RASTERIZE_SAMPLE_METHOD_COMMIT,
            get: shrimply_inspector_core::visual_modifiers::rasterize_sample_method,
            get_mut: shrimply_inspector_core::visual_modifiers::rasterize_sample_method_mut,
        },
    );
    sample_method_row.set_sensitive(sample_method.sensitive);
    if sample_method.visible {
        out.append(&sample_method_row);
    }
}
