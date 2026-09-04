use gtk::prelude::{BoxExt, WidgetExt};
use shrimply_inspector_core::InspectorTarget;
use shrimply_video_modifiers::sampling::SamplingModifier;
use uuid::Uuid;

use crate::InspectorContext;

pub fn add_rows(
    value: &SamplingModifier,
    out: &gtk::Box,
    modifier_id: Uuid,
    context: &InspectorContext,
) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    let section = context
        .inspector_core
        .sampling_presentation(&InspectorTarget::Item(key), modifier_id)
        .expect("live Sampling modifier must have a shared presentation");
    let [method] = section
        .controls
        .try_into()
        .expect("Sampling presentation must contain exactly one control");
    let method_row = super::shared_step_row(
        &method,
        "method",
        &value.method,
        context,
        super::ModifierStepTarget {
            modifier_id,
            commit_name: shrimply_inspector_core::visual_modifiers::SAMPLING_METHOD_COMMIT,
            get: shrimply_inspector_core::visual_modifiers::sampling_method,
            get_mut: shrimply_inspector_core::visual_modifiers::sampling_method_mut,
        },
    );
    method_row.set_sensitive(method.sensitive);
    if method.visible {
        out.append(&method_row);
    }
}
