use shrimply_cuda::LaunchConfig;
use shrimply_render_core::DitheringParams;

use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext};

impl GpuModifier for shrimply_video_core::raster_modifiers::Dithering {
    fn name(&self) -> &'static str {
        "Dithering"
    }

    fn apply(
        &self,
        context: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        let width = input.width();
        let count = width as usize * input.height() as usize;
        let launch =
            LaunchConfig::for_num_elems(u32::try_from(count).map_err(|_| "canvas is too large")?);
        let mut pass = input.into_pass(context)?;
        let module = context.modifier_module(crate::gpu::modifiers::ModifierModule::General)?;
        let palette = context.upload(&self.palette)?;
        let palette_ptr = palette.cu_deviceptr() as usize as *const u32;
        let palette_len = u32::try_from(self.palette.len()).map_err(|_| "palette is too large")?;
        let params = DitheringParams {
            pattern: self.pattern,
            color_mode: self.color_mode,
            levels: self.levels,
            amount: self.amount,
            palette: palette_ptr,
            palette_len,
            _padding_0: [0; 4],
        };
        unsafe {
            shrimply_cuda::cuda_launch! {
                kernel: dithering,
                stream: context.stream(),
                module: &module,
                config: launch,
                args: [
                    pass.input_ptr(),
                    width,
                    slice_mut(pass.output_buffer()),
                    params
                ]
            }
        }
        .map_err(|error| format!("launch dithering CUDA kernel: {error:?}"))?;
        Ok(pass.finish(context))
    }
}
