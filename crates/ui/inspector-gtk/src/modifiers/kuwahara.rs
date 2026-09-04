use gtk::prelude::*;
use shrimply_inspector_core::InspectorTarget;
use shrimply_video_modifiers::kuwahara::KuwaharaModifier;
use uuid::Uuid;

use super::InspectorContext;

pub fn add_rows(
    value: &KuwaharaModifier,
    out: &gtk::Box,
    modifier_id: Uuid,
    context: &InspectorContext,
) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    let section = context
        .inspector_core
        .kuwahara_presentation(&InspectorTarget::Item(key), modifier_id)
        .expect("live Kuwahara modifier must have a shared presentation");
    let [version, radius] = section
        .controls
        .try_into()
        .expect("Kuwahara presentation must contain exactly two controls");
    out.append(&super::shared_step_row(
        &version,
        "version",
        &value.version,
        modifier_id,
        context,
        shrimply_inspector_core::visual_modifiers::KUWAHARA_VERSION_COMMIT,
        shrimply_inspector_core::visual_modifiers::kuwahara_version,
        shrimply_inspector_core::visual_modifiers::kuwahara_version_mut,
    ));
    out.append(&super::shared_scalar_row(
        &radius,
        "radius",
        &value.radius,
        modifier_id,
        context,
    ));
}
