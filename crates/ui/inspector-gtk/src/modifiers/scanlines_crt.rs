use gtk::prelude::*;
use shrimply_inspector_core::InspectorTarget;
use shrimply_video_modifiers::scanlines_crt::ScanlinesCrtModifier;
use uuid::Uuid;

use super::InspectorContext;

pub fn add_rows(
    value: &ScanlinesCrtModifier,
    out: &gtk::Box,
    id: Uuid,
    context: &InspectorContext,
) {
    let key = context
        .selected_item
        .clone()
        .expect("Scanlines/CRT inspector must have a selected item");
    let section = context
        .inspector_core
        .scanlines_crt_presentation(&InspectorTarget::Item(key), id)
        .expect("live Scanlines/CRT modifier must have a shared presentation");
    let controls: [_; 4] = section
        .controls
        .try_into()
        .expect("Scanlines/CRT presentation must contain exactly four controls");
    for (control, (field, timeline)) in controls.into_iter().zip([
        ("spacing", &value.spacing),
        ("intensity", &value.intensity),
        ("curvature", &value.curvature),
        ("mask_strength", &value.mask_strength),
    ]) {
        let row = super::shared_scalar_row(&control, field, timeline, id, context);
        row.set_sensitive(control.sensitive);
        if control.visible {
            out.append(&row);
        }
    }
}
