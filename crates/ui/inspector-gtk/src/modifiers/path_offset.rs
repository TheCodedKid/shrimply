use gtk::prelude::*;
use shrimply_inspector_core::InspectorTarget;
use uuid::Uuid;

use super::InspectorContext;
use shrimply_video_modifiers::path_offset::PathOffsetModifier;

pub fn add_rows(
    value: &PathOffsetModifier,
    out: &gtk::Box,
    modifier_id: Uuid,
    context: &InspectorContext,
) {
    let key = context
        .selected_item
        .clone()
        .expect("Path offset inspector must have a selected item");
    let section = context
        .inspector_core
        .path_offset_presentation(&InspectorTarget::Item(key), modifier_id)
        .expect("live Path offset modifier must have a shared presentation");
    let controls: [_; 4] = section
        .controls
        .try_into()
        .expect("Path offset presentation must contain exactly four controls");
    for (control, (field, timeline)) in controls.into_iter().zip([
        ("amplitude", &value.amplitude),
        ("spacing", &value.spacing),
        ("seed", &value.seed),
        ("evolution", &value.evolution),
    ]) {
        out.append(&super::shared_scalar_row(
            &control,
            field,
            timeline,
            modifier_id,
            context,
        ));
    }
}
