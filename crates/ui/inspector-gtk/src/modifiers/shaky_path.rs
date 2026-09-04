use gtk::prelude::*;
use shrimply_inspector_core::InspectorTarget;
use uuid::Uuid;

use super::InspectorContext;
use shrimply_video_modifiers::shaky_path::ShakyPathModifier;

pub fn add_rows(
    value: &ShakyPathModifier,
    out: &gtk::Box,
    modifier_id: Uuid,
    context: &InspectorContext,
) {
    let key = context
        .selected_item
        .clone()
        .expect("Shaky path inspector must have a selected item");
    let section = context
        .inspector_core
        .shaky_path_presentation(&InspectorTarget::Item(key), modifier_id)
        .expect("live Shaky path modifier must have a shared presentation");
    let controls: [_; 4] = section
        .controls
        .try_into()
        .expect("Shaky path presentation must contain exactly four controls");
    for (control, (field, timeline)) in controls.into_iter().zip([
        ("amplitude", &value.amplitude),
        ("step_size", &value.step_size),
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
