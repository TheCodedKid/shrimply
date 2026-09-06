use crate::gpu::modifiers::{CanvasRgbaFrame, GpuModifier, ModifierContext};
use crate::layer::Visual;
use shrimply_cuda::LaunchConfig;
use shrimply_project::project::{VisualClipTransition, VisualTransition};

struct Origami {
    visibility: f32,
    depth: f32,
    direction_degrees: f32,
    grid: u32,
}

impl GpuModifier for Origami {
    fn name(&self) -> &'static str {
        "Origami transition"
    }

    fn apply(
        &self,
        context: &mut ModifierContext<'_>,
        input: CanvasRgbaFrame,
    ) -> Result<CanvasRgbaFrame, String> {
        let width = input.width();
        let height = input.height();
        let count = width as usize * height as usize;
        let vertices = crate::math::origami_mesh_vertices(
            width,
            height,
            self.grid,
            self.visibility,
            self.depth,
            self.direction_degrees,
        );
        let vertices = context.upload(&vertices)?;
        let vertices_ptr = vertices.cu_deviceptr() as usize as *const f32;
        let mut pass = input.into_pass(context)?;
        let module = context.modifier_module(crate::gpu::modifiers::ModifierModule::Matte)?;
        unsafe {
            shrimply_cuda::cuda_launch! {
                kernel: origami_transition,
                stream: context.stream(), module: &module,
                config: LaunchConfig::for_num_elems(u32::try_from(count).map_err(|_| "canvas is too large")?),
                args: [
                    pass.input_ptr(),
                    width,
                    height,
                    slice_mut(pass.output_buffer()),
                    vertices_ptr,
                    self.grid,
                    self.visibility
                ]
            }
        }
        .map_err(|error| format!("launch origami transition CUDA kernel: {error:?}"))?;
        Ok(pass.finish(context))
    }
}

pub(crate) fn apply(
    visual: &mut Visual,
    transition: &VisualTransition,
    visibility: f32,
    center: glam::Vec2,
) {
    match shrimply_video_core::transition::raster_plan(transition, visibility, center) {
        Some(shrimply_video_core::transition::RasterTransition::Origami(effect)) => {
            visual.push_pixel(Box::new(Origami {
                visibility: effect.visibility,
                depth: effect.depth,
                direction_degrees: effect.direction_degrees,
                grid: effect.grid,
            }));
        }
        Some(shrimply_video_core::transition::RasterTransition::Pixel(effect)) => {
            visual.push_pixel(Box::new(effect));
        }
        None => {}
    }
}

pub(crate) fn apply_clip_mask(
    visual: &mut Visual,
    transition: &VisualClipTransition,
    progress: f32,
) {
    if let Some(effect) = shrimply_video_core::transition::clip_mask(transition, progress) {
        visual.push_pixel(Box::new(effect));
    }
}
