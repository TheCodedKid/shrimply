use crate::InspectorContext;
use gtk::prelude::*;
use shrimply_core::timeline_value::TimelineStep;
use shrimply_inspector_core::{ControlKind, InspectorTarget, NumberSpec};
use shrimply_video_modifiers::repeat::{RepeatModifier, RepeatOffsetAxis};
use uuid::Uuid;

use super::vec_row;

pub fn add_rows(value: &RepeatModifier, out: &gtk::Box, id: Uuid, context: &InspectorContext) {
    let key = context
        .selected_item
        .clone()
        .expect("Repeat inspector must have a selected item");
    let section = context
        .inspector_core
        .repeat_presentation(&InspectorTarget::Item(key), id)
        .expect("live Repeat modifier must have a shared presentation");
    let [copies_x, copies_y, step, row_offset, offset_axis] = section
        .controls
        .try_into()
        .expect("Repeat presentation must contain exactly five controls");

    for (control, field, timeline) in [
        (copies_x, "copies_x", &value.copies_x),
        (copies_y, "copies_y", &value.copies_y),
    ] {
        assert_eq!(
            control.number,
            NumberSpec {
                minimum: 1.0,
                drag_step: 1.0,
                digits: 0,
                ..NumberSpec::default()
            },
        );
        assert!(control.integer);
        assert!(!control.lock);
        let widget = super::shared_scalar_row(&control, field, timeline, id, context);
        widget.set_sensitive(control.sensitive);
        if control.visible {
            out.append(&widget);
        }
    }

    assert_eq!(step.kind, ControlKind::LayeredVector2);
    assert!(step.path.ends_with("/effect/effect/config/step"));
    assert_eq!(step.target_id, Some(id));
    assert_eq!(step.timeline_id, Some(value.step.id));
    assert_eq!(
        step.number,
        NumberSpec {
            drag_step: 1.0,
            digits: 0,
            unit: "px",
            ..NumberSpec::default()
        },
    );
    assert_eq!(step.width_characters, 7);
    assert_eq!(step.prefixes, ["X", "Y"]);
    assert!(!step.lock);
    assert_eq!(step.commit_name, "visual-modifier-vector");
    let step_row = vec_row(&step.label, &value.step, id, false, None, context);
    step_row.set_sensitive(step.sensitive);
    if step.visible {
        out.append(&step_row);
    }

    assert_eq!(
        row_offset.number,
        NumberSpec {
            drag_step: 0.01,
            digits: 2,
            unit: "px",
            ..NumberSpec::default()
        },
    );
    assert!(!row_offset.integer);
    assert!(!row_offset.lock);
    let row_offset_row =
        super::shared_scalar_row(&row_offset, "row_offset", &value.row_offset, id, context);
    row_offset_row.set_sensitive(row_offset.sensitive);
    if row_offset.visible {
        out.append(&row_offset_row);
    }

    assert_eq!(offset_axis.kind, ControlKind::LayeredSelector);
    assert!(
        offset_axis
            .path
            .ends_with("/effect/effect/config/row_offset_axis"),
    );
    assert_eq!(offset_axis.target_id, Some(id));
    assert_eq!(offset_axis.timeline_id, Some(value.row_offset_axis.id));
    assert_eq!(
        offset_axis.commit_name,
        shrimply_inspector_core::visual_modifiers::REPEAT_OFFSET_AXIS_COMMIT,
    );
    assert_eq!(
        offset_axis.values,
        RepeatOffsetAxis::variants()
            .iter()
            .map(|variant| variant.key.to_string())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        offset_axis.labels,
        RepeatOffsetAxis::variants()
            .iter()
            .map(|variant| variant.label.to_string())
            .collect::<Vec<_>>(),
    );
    assert!(offset_axis.icons.is_empty());
    let offset_axis_row = super::shared_step_row(
        &offset_axis,
        "row_offset_axis",
        &value.row_offset_axis,
        id,
        context,
        shrimply_inspector_core::visual_modifiers::REPEAT_OFFSET_AXIS_COMMIT,
        shrimply_inspector_core::visual_modifiers::repeat_offset_axis,
        shrimply_inspector_core::visual_modifiers::repeat_offset_axis_mut,
    );
    offset_axis_row.set_sensitive(offset_axis.sensitive);
    if offset_axis.visible {
        out.append(&offset_axis_row);
    }
}
