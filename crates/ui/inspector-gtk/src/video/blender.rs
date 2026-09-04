use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use shrimply_gtk_components::ui::StringSelector;
use shrimply_project::project::{Asset, BlenderItem};

use crate::InspectorContext;
use crate::item::{DefaultInspectorItem, InspectorListItem};
use crate::section::InspectorSection;
use crate::selector::{StringChoice, labeled_string_selector};

use shrimply_inspector_core::video::VideoReset;
use shrimply_inspector_core::video::blender::{self as shared, MetadataState};

pub(super) fn item(
    blender: &BlenderItem,
    source: Asset,
    _context: &InspectorContext,
) -> InspectorListItem {
    let card = shared::card(
        blender,
        &source.path().to_string_lossy(),
        &MetadataState::Loading,
    );
    let reset = card
        .reset
        .expect("shared Blender card must provide reset metadata");
    let actions = super::header_actions(card.actions);
    let controls_source = source.clone();
    DefaultInspectorItem::new(
        card.key,
        card.title,
        blender.clone(),
        move |blender, context| controls(blender, &controls_source, context),
        move |context, _| reset_blender(context, &reset),
    )
    .actions(actions)
    .boxed()
}

fn reset_blender(context: &InspectorContext, reset: &VideoReset) {
    let Some(address) = context.selected_item.as_ref() else {
        return;
    };
    if let Err(error) = context.inspector_core.reset_video(
        &shrimply_inspector_core::InspectorTarget::Item(address.clone()),
        reset,
    ) {
        tracing::warn!("Could not reset Blender inspector: {error}");
    }
}

fn controls(blender: &BlenderItem, source: &Asset, context: &InspectorContext) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    let Some(address) = context.selected_item.clone() else {
        return vec![section.into_widget()];
    };
    let binary = shrimply_state::preferences::snapshot(&context.preferences).blender_binary;
    let metadata = shared::metadata(source, binary.as_deref());
    let metadata_controls = shared::metadata_controls(blender, &metadata);
    let selectors = metadata_controls
        .iter()
        .map(|control| shared_selector(control, address.clone(), context))
        .collect::<Vec<_>>();
    for selector in &selectors {
        section.add_wide_control(selector.widget());
    }
    for control in shared::settings_controls(blender) {
        section.add_wide_control(shared_selector(&control, address.clone(), context).widget());
    }

    let current = blender.clone();
    let source = source.clone();
    let listener_scope = Rc::downgrade(&context.listener_scope);
    let inspector = context.inspector_core.clone();
    let target = shrimply_inspector_core::InspectorTarget::Item(address);
    glib::spawn_future_local(async move {
        let metadata = if matches!(metadata, MetadataState::Loading) {
            loop {
                glib::timeout_future(std::time::Duration::from_millis(50)).await;
                let metadata = shared::metadata(&source, binary.as_deref());
                if !matches!(metadata, MetadataState::Loading) {
                    break metadata;
                }
                if listener_scope.upgrade().is_none() {
                    return;
                }
            }
        } else {
            metadata
        };
        if listener_scope.upgrade().is_none() {
            return;
        }
        for (selector, control) in selectors
            .iter()
            .zip(shared::metadata_controls(&current, &metadata))
        {
            selector.set_choices(
                &control.value,
                control
                    .values
                    .iter()
                    .cloned()
                    .zip(control.labels.iter().cloned())
                    .map(|(value, label)| StringChoice { value, label })
                    .collect(),
            );
            selector.set_sensitive(control.sensitive);
            selector.widget().set_tooltip_text(
                (!control.tooltip.is_empty()).then_some(control.tooltip.as_str()),
            );
        }
        if let MetadataState::Ready(metadata) = metadata
            && let Err(error) = inspector.sync_blender_metadata(&target, &metadata)
        {
            tracing::warn!("Could not synchronize Blender metadata: {error}");
        }
    });

    vec![section.into_widget()]
}

fn shared_selector(
    control: &shrimply_inspector_core::InspectorControl,
    address: shrimply_project::project::ItemAddress,
    context: &InspectorContext,
) -> StringSelector {
    let choices = control
        .values
        .iter()
        .cloned()
        .zip(control.labels.iter().cloned())
        .map(|(value, label)| StringChoice { value, label })
        .collect();
    let path = control.path.clone();
    let commit_name = control.commit_name.clone();
    let commit_immediately = control.commit_immediately;
    let inspector = context.inspector_core.clone();
    let selector = labeled_string_selector(&control.label, &control.value, choices, move |value| {
        if let Err(error) = inspector.set_video_field(
            &shrimply_inspector_core::InspectorTarget::Item(address.clone()),
            &path,
            &value,
            &commit_name,
            commit_immediately,
        ) {
            tracing::warn!("Could not edit Blender inspector: {error}");
        }
    });
    selector.set_sensitive(control.sensitive);
    selector
}
