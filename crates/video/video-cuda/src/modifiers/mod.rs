mod dithering;
mod mask;
pub(crate) mod sam2;
mod shared;
mod stabilization_warp;
pub(crate) mod transparent_fill;

use crate::layer::Visual;
use crate::visual_source::VisualModifierContext;
use shrimply_video_modifiers::ModifierEffect;

pub(crate) fn stabilization_warp(
    source_transform: glam::Mat3,
) -> Box<dyn crate::layer::PreservingRasterModifier> {
    Box::new(stabilization_warp::Source { source_transform })
}

pub(crate) fn apply_source(
    effect: &ModifierEffect,
    input: Visual,
    context: &mut VisualModifierContext<'_>,
) -> Result<Visual, String> {
    match effect {
        ModifierEffect::Scene3d(_) => Ok(input),
        // Still-image sources perform the expensive trace before the modifier chain so the
        // remaining vector modifiers never see an intermediate raster surface.
        ModifierEffect::Vectorize(_) => {
            let Visual::Vector(_) = input else {
                unreachable!("validated Vectorize modifier received untraced raster input")
            };
            Ok(input)
        }
        ModifierEffect::Vector(effect) => {
            let Visual::Vector(mut input) = input else {
                unreachable!("validated vector modifier chain received raster input")
            };
            if let Some(operation) = shrimply_video_core::vector_modifiers::operation(
                effect,
                context.evaluation,
                context.expressions,
            ) {
                input.push(operation);
            }
            Ok(Visual::Vector(input))
        }
        ModifierEffect::Rasterize(effect) => {
            let configured = effect
                .sample_method
                .value_at(context.evaluation.local_time());
            let sample_method = shrimply_video_core::generated::sampling(
                configured,
                context.accuracy.content_accurate(),
            );
            Ok(Visual::Raster(match input {
                Visual::Vector(input) => {
                    input.rasterize(context.item.skia_drawing_strategy, sample_method)
                }
                // Scene3D sources are already canvas-sized CUDA RGBA. Rasterize is their
                // semantic kind boundary and intentionally performs no copy.
                Visual::Raster(input) => input,
            }))
        }
        ModifierEffect::Raster(_) => Err("raster modifier bypassed shared planning".to_string()),
    }
}

pub(crate) fn apply_resolved(
    operation: shrimply_video_core::raster_modifiers::Operation,
    input: Visual,
    mask_source: Option<std::rc::Rc<crate::gpu::VisualFrame>>,
) -> Result<Visual, String> {
    use shrimply_video_core::raster_modifiers::Operation;
    let Visual::Raster(mut input) = input else {
        return Err("shared raster operation received vector input".to_string());
    };
    match operation {
        Operation::Pixel(effect) => input.push_pixel(Box::new(effect)),
        Operation::Dithering(effect) => input.push_pixel(Box::new(effect)),
        Operation::Sam2Mask(effect) => input.push_pixel(Box::new(effect)),
        Operation::TransparentFillMask(effect) => input.push_pixel(Box::new(effect)),
        Operation::Mask(effect) => input = mask::apply(input, &effect, mask_source),
        Operation::Transform(transform) => input.push_spatial(move |state| {
            state.transform = transform.compose(state.transform);
        }),
        Operation::Opacity(opacity) => input.push_spatial(move |state| {
            state.compositing.opacity *= opacity;
        }),
        Operation::Sampling(method) => input.push_spatial(move |state| state.sampling = method),
        Operation::TextureBounds {
            edges,
            address_mode,
        } => input.push_spatial(move |state| {
            for (current, added) in state.bounds.edges.iter_mut().zip(edges) {
                *current += added;
            }
            state.bounds.address_mode = match address_mode {
                shrimply_render_core::TextureAddressMode::Transparent => {
                    shrimply_core::TextureAddressMode::Transparent
                }
                shrimply_render_core::TextureAddressMode::ClampToEdge => {
                    shrimply_core::TextureAddressMode::ClampToEdge
                }
                shrimply_render_core::TextureAddressMode::Repeat => {
                    shrimply_core::TextureAddressMode::Repeat
                }
                shrimply_render_core::TextureAddressMode::MirrorRepeat => {
                    shrimply_core::TextureAddressMode::MirrorRepeat
                }
                shrimply_render_core::TextureAddressMode::BlurredMirror => {
                    shrimply_core::TextureAddressMode::BlurredMirror
                }
                shrimply_render_core::TextureAddressMode::Stochastic => {
                    shrimply_core::TextureAddressMode::Stochastic
                }
            };
        }),
        Operation::CropPercentage(crop) => input.push_spatial(move |state| {
            (
                state.bounds.modifier_crop,
                state.bounds.modifier_crop_pixels,
            ) = crate::math::compose_fractional_crop(
                state.bounds.modifier_crop,
                state.bounds.modifier_crop_pixels,
                crate::math::normalized_crop(crop),
            );
        }),
        Operation::CropPixels(crop) => input.push_spatial(move |state| {
            for (current, added) in state.bounds.modifier_crop_pixels.iter_mut().zip(crop) {
                *current += added.max(0.0);
            }
        }),
        Operation::RasterBoundary => {}
    }
    Ok(Visual::Raster(input))
}
