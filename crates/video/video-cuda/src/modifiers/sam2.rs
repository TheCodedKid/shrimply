use shrimply_video_modifiers::sam2::Sam2Modifier;

use super::RasterModifierRuntime;
use crate::{
    gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext, ModifierModule},
    layer::RasterVisual,
    visual_source::VisualModifierContext,
};

pub(crate) use shrimply_video_core::sam2::{
    MASK_LOGIT_QUANTIZATION_SCALE, MASK_SIZE, MODEL_SIZE, Sam2MaskCache, cache_key, validate_cache,
};

impl GpuModifier for shrimply_video_core::sam2::ResolvedMask {
    fn name(&self) -> &'static str {
        "Segment Anything 2"
    }

    fn apply(
        &self,
        context: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        if context.capture_sam2(&self.target, &input)? {
            return Ok(input);
        }
        let Some(mask) = &self.mask else {
            return Ok(input);
        };
        let mask = context.upload_sam2_mask(mask)?;
        let width = input.width();
        let height = input.height();
        let count = width as usize * height as usize;
        let mut pass = input.into_pass(context)?;
        let module = context.modifier_module(ModifierModule::Matte)?;
        unsafe {
            shrimply_cuda::cuda_launch! {
                kernel: sam2_apply_mask,
                stream: context.stream(), module: &module,
                config: shrimply_cuda::LaunchConfig::for_num_elems(u32::try_from(count).map_err(|_| "canvas is too large")?),
                args: [pass.input_ptr(), mask, slice_mut(pass.output_buffer()), shrimply_render_core::Sam2MaskParams {
                    output_width: width,
                    output_height: height,
                    mask_size: MASK_SIZE,
                    threshold: self.threshold,
                    softness: self.softness,
                    invert: self.invert,
                    quantization_scale: MASK_LOGIT_QUANTIZATION_SCALE,
                    _padding_0: [0; 3],
                }]
            }
        }
        .map_err(|error| format!("launch SAM2 mask kernel: {error:?}"))?;
        Ok(pass.finish(context))
    }
}

impl RasterModifierRuntime for Sam2Modifier {
    fn apply_raster(
        &self,
        mut input: RasterVisual,
        context: &mut VisualModifierContext<'_>,
    ) -> Result<RasterVisual, String> {
        if self.points.is_empty() && self.box_prompt.is_none() {
            return Ok(input);
        }
        let Some(resolved) = shrimply_video_core::sam2::resolve(
            context.project,
            context.address,
            context.item,
            context.position,
            context.modifier_id,
            context.modifier_index,
            context.require_complete_assets,
        )?
        else {
            return Ok(input);
        };
        input.push_pixel(Box::new(resolved));
        Ok(input)
    }
}
