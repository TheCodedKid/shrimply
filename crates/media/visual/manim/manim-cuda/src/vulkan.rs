use hashbrown::HashMap;
pub use shrimply_gpu_vulkan::ExportedFrame;
use shrimply_gpu_vulkan::ExternalImage as Target;
use shrimply_manim_wgpu::{ExternalFrameDescriptor, PreparedAnimation, Renderer as SharedRenderer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderedExternalFrame {
    pub descriptor: ExternalFrameDescriptor,
    pub semaphore_value: u64,
}

pub struct Renderer {
    shared: SharedRenderer,
    targets: HashMap<usize, Target>,
}
impl Renderer {
    pub fn new(
        cuda_device_uuid: [u8; shrimply_gpu_cuda::DEVICE_UUID_BYTES],
    ) -> Result<Self, String> {
        let (device, queue) = shrimply_gpu_vulkan::device_for_cuda(cuda_device_uuid)?;
        Ok(Self {
            shared: SharedRenderer::from_device(device, queue),
            targets: HashMap::new(),
        })
    }

    pub fn target_descriptor(&self, slot: usize) -> Option<ExternalFrameDescriptor> {
        self.shared.target_descriptor(slot)
    }
    pub fn release_render_surfaces(&mut self) -> bool {
        let released = self.shared.release_render_surfaces() || !self.targets.is_empty();
        self.targets.clear();
        released
    }
    pub fn release_gpu_animation_resources(&mut self) -> bool {
        self.shared.release_gpu_animation_resources()
    }
    pub fn remove_target(&mut self, slot: usize) -> bool {
        self.shared.remove_target(slot) | self.targets.remove(&slot).is_some()
    }
    pub fn render_external(
        &mut self,
        slot: usize,
        animation: &PreparedAnimation,
        frame_index: usize,
    ) -> Result<RenderedExternalFrame, String> {
        let descriptor = SharedRenderer::external_frame_descriptor(animation);
        if !matches!(descriptor.samples, 1 | 2 | 4 | 8 | 16) {
            return Err(format!(
                "unsupported Manim sample count {}",
                descriptor.samples
            ));
        }
        if self.target_descriptor(slot) != Some(descriptor) {
            self.remove_target(slot);
            self.targets.insert(
                slot,
                Target::new(&self.shared.device, [descriptor.width, descriptor.height])?,
            );
        }
        let target = self
            .targets
            .get(&slot)
            .expect("initialized Manim Vulkan target");
        let mut encoder =
            self.shared
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Manim Vulkan frame"),
                });
        self.shared.encode(
            slot,
            animation,
            frame_index,
            target.output.clone(),
            &mut encoder,
        )?;
        let semaphore_value = target.prepare_submit(&self.shared.queue, &mut encoder)?;
        self.shared.queue.submit([encoder.finish()]);
        Ok(RenderedExternalFrame {
            descriptor,
            semaphore_value,
        })
    }
    pub fn export_frame(&self, slot: usize) -> Result<ExportedFrame, String> {
        self.targets
            .get(&slot)
            .ok_or("Manim target is not initialized")?
            .export(&self.shared.device)
    }
}
