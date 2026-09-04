use gtk::prelude::*;
use shrimply_inspector_core::InspectorTarget;
use shrimply_video_modifiers::texture_bounds::TextureBoundsModifier;
use uuid::Uuid;

use crate::InspectorContext;

pub fn add_rows(
    value: &TextureBoundsModifier,
    out: &gtk::Box,
    modifier_id: Uuid,
    context: &InspectorContext,
) {
    let key = context
        .selected_item
        .clone()
        .expect("Texture bounds inspector must have a selected item");
    let section = context
        .inspector_core
        .texture_bounds_presentation(&InspectorTarget::Item(key), modifier_id)
        .expect("live Texture bounds modifier must have a shared presentation");
    let [top, right, bottom, left, address_mode] = section
        .controls
        .try_into()
        .expect("Texture bounds presentation must contain exactly five controls");

    for (control, field, timeline) in [
        (top, "edges/top", &value.edges.top),
        (right, "edges/right", &value.edges.right),
        (bottom, "edges/bottom", &value.edges.bottom),
        (left, "edges/left", &value.edges.left),
    ] {
        let row = super::shared_scalar_row(&control, field, timeline, modifier_id, context);
        row.set_sensitive(control.sensitive);
        if control.visible {
            out.append(&row);
        }
    }

    let row = super::shared_step_row(
        &address_mode,
        "address_mode",
        &value.address_mode,
        modifier_id,
        context,
        shrimply_inspector_core::visual_modifiers::TEXTURE_BOUNDS_ADDRESS_MODE_COMMIT,
        shrimply_inspector_core::visual_modifiers::texture_bounds_address_mode,
        shrimply_inspector_core::visual_modifiers::texture_bounds_address_mode_mut,
    );
    row.set_sensitive(address_mode.sensitive);
    if address_mode.visible {
        out.append(&row);
    }
}
