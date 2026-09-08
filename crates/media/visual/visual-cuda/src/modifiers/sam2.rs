use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext, ModifierModule};

pub(crate) use shrimply_visual_core::sam2::{
    MASK_LOGIT_QUANTIZATION_SCALE, MASK_SIZE, MODEL_SIZE, validate_cache,
};

impl GpuModifier for shrimply_visual_core::sam2::ResolvedMask {
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
            shrimply_gpu_cuda::cuda_launch! {
                kernel: sam2_apply_mask,
                stream: context.stream(), module: &module,
                config: shrimply_gpu_cuda::LaunchConfig::for_num_elems(u32::try_from(count).map_err(|_| "canvas is too large")?),
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
