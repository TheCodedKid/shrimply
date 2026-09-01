use gtk::prelude::*;
use shrimply_gtk_components::{
    tr,
    ui::{SearchMenuItem, search_rank, searchable_menu},
};

use crate::{InspectorContext, player_state};
use shrimply_video_modifiers::{
    ModifierEffect, ModifierModel, RasterModifierEffect, VectorModifierEffect,
};

pub(super) fn button(context: &InspectorContext) -> gtk::Widget {
    let context = context.detached();
    searchable_menu(
        tr!("Add modifier").as_ref(),
        tr!("Search modifiers").as_ref(),
        move |query| items(query, &context),
    )
    .upcast()
}

fn items(query: &str, context: &InspectorContext) -> Vec<SearchMenuItem> {
    let state = context.selected_item.clone().and_then(|key| {
        context
            .project
            .borrow()
            .video_item(&key)?
            .modifier_output_state()
            .ok()
    });
    let Some(state) = state else {
        return Vec::new();
    };
    let canvas = context.project.borrow().canvas_size;
    let canvas_size = glam::Vec2::new(canvas.width.max(1) as f32, canvas.height.max(1) as f32);
    let transform_center = transform_center(context, canvas_size * 0.5);
    let mut effects = ModifierEffect::catalog()
        .filter_map(|effect| effect.adapted_for(state))
        .filter_map(|mut effect| {
            match &mut effect {
                ModifierEffect::Vector(effect) => {
                    if let VectorModifierEffect::Transform(transform) = &mut **effect {
                        **transform =
                            shrimply_video_modifiers::transform::TransformModifier::centered_at(
                                transform_center,
                            );
                    }
                }
                ModifierEffect::Rasterize(rasterize) => {
                    *rasterize =
                        shrimply_video_modifiers::rasterize::RasterizeModifier::new(canvas_size);
                }
                ModifierEffect::Raster(effect) => {
                    if let RasterModifierEffect::Transform(transform) = &mut **effect {
                        **transform =
                            shrimply_video_modifiers::transform::TransformModifier::centered_at(
                                transform_center,
                            );
                    }
                }
                ModifierEffect::Scene3d(_) | ModifierEffect::Vectorize(_) => {}
            }
            let rank = search_rank(
                effect.display_name(),
                effect.keywords().iter().copied(),
                query,
            )?;
            Some((rank, effect))
        })
        .collect::<Vec<_>>();
    effects.sort_by_key(|(rank, _)| *rank);
    effects
        .into_iter()
        .map(|(_, effect)| {
            let label = tr!(effect.display_name()).into_owned();
            let context = context.detached();
            SearchMenuItem::new(label, move || super::add_effect(effect.clone(), &context))
        })
        .collect()
}

fn transform_center(context: &InspectorContext, fallback: glam::Vec2) -> glam::Vec2 {
    let Some(key) = context.selected_item.clone() else {
        return fallback;
    };
    let position = player_state::snapshot(&context.player_state).position;
    let audio = context.audio_analysis_at(position);
    let project = context.project.borrow();
    let Some(item) = project.video_item(&key) else {
        return fallback;
    };
    let center = shrimply_evaluation::resolve_item_transform_with_audio(
        &project,
        item,
        position,
        &audio,
        &mut Default::default(),
    )
    .position;
    if center.is_finite() { center } else { fallback }
}
