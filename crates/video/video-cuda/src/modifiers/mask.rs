use std::rc::Rc;

use shrimply_cuda::LaunchConfig;

use crate::gpu::VisualFrame;
use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext};
use crate::layer::{PreservingRasterModifier, RasterVisual, VisualState};
use shrimply_render_core::MaskParams;

struct Resolved {
    mask: Option<Rc<VisualFrame>>,
    transform: glam::Mat3,
    luminance: bool,
    invert: bool,
}

struct Pending {
    mask: Option<Rc<VisualFrame>>,
    luminance: bool,
    invert: bool,
}

pub(super) fn apply(
    mut input: RasterVisual,
    effect: &shrimply_video_core::raster_modifiers::ExternalMask,
    mask: Option<Rc<VisualFrame>>,
) -> RasterVisual {
    input.push_preserving_pixel(Box::new(Pending {
        mask,
        luminance: effect.luminance,
        invert: effect.invert,
    }));
    input
}

impl PreservingRasterModifier for Pending {
    fn resolve(&self, state: VisualState) -> Box<dyn GpuModifier> {
        Box::new(Resolved {
            mask: self.mask.clone(),
            transform: state.transform.matrix,
            luminance: self.luminance,
            invert: self.invert,
        })
    }
}

impl GpuModifier for Resolved {
    fn name(&self) -> &'static str {
        "Mask"
    }

    fn apply(
        &self,
        context: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        let width = input.width();
        let height = input.height();
        let count = width as usize * height as usize;
        let (mask, mask_width, mask_height) =
            self.mask.as_ref().map_or((std::ptr::null(), 1, 1), |mask| {
                let plane = mask.plane(0).expect("RGBA mask has no plane");
                (
                    plane.device_ptr as usize as *const u32,
                    mask.width().max(1),
                    mask.height().max(1),
                )
            });
        let mut pass = input.into_pass(context)?;
        let module = context.modifier_module(crate::gpu::modifiers::ModifierModule::Matte)?;
        unsafe {
            shrimply_cuda::cuda_launch! {
                kernel: mask,
                stream: context.stream(),
                module: &module,
                config: LaunchConfig::for_num_elems(
                    u32::try_from(count).map_err(|_| "canvas is too large")?
                ),
                args: [
                    pass.input_ptr(),
                    MaskParams {
                        mask,
                        input_width: width,
                        mask_width,
                        mask_height,
                        transform: self.transform,
                        luminance: self.luminance,
                        invert: self.invert,
                        _padding_0: [0; 6],
                    },
                    slice_mut(pass.output_buffer())
                ]
            }
        }
        .map_err(|error| format!("launch mask CUDA kernel: {error:?}"))?;
        Ok(pass.finish(context))
    }
}
