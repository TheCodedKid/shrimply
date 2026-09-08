#![cfg(target_os = "macos")]

use objc2::{AnyThread, rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTLCommandBuffer, MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor,
    MTLTextureType, MTLTextureUsage,
};
use objc2_metal_performance_shaders::MPSSVGFDenoiser;

pub struct Targets {
    pub beauty: Retained<ProtocolObject<dyn MTLTexture>>,
    pub background: Retained<ProtocolObject<dyn MTLTexture>>,
    pub alpha: Retained<ProtocolObject<dyn MTLTexture>>,
    pub depth_normal: Retained<ProtocolObject<dyn MTLTexture>>,
}

impl Targets {
    pub fn new(device: &ProtocolObject<dyn MTLDevice>, size: [u32; 2]) -> Result<Self, String> {
        let [width, height] = size;
        Ok(Self {
            beauty: allocate_denoiser_texture(device, width, height)?,
            background: allocate_denoiser_texture(device, width, height)?,
            alpha: allocate_denoiser_texture(device, width, height)?,
            depth_normal: allocate_denoiser_texture(device, width, height)?,
        })
    }
}

pub struct Output {
    pub beauty: Retained<ProtocolObject<dyn MTLTexture>>,
    pub background: Retained<ProtocolObject<dyn MTLTexture>>,
    pub alpha: Retained<ProtocolObject<dyn MTLTexture>>,
}

pub struct Denoiser {
    beauty: Retained<MPSSVGFDenoiser>,
    background: Retained<MPSSVGFDenoiser>,
    alpha: Retained<MPSSVGFDenoiser>,
}

impl Denoiser {
    pub fn new(device: &ProtocolObject<dyn MTLDevice>) -> Self {
        Self {
            beauty: unsafe { MPSSVGFDenoiser::initWithDevice(MPSSVGFDenoiser::alloc(), device) },
            background: unsafe {
                MPSSVGFDenoiser::initWithDevice(MPSSVGFDenoiser::alloc(), device)
            },
            alpha: unsafe { MPSSVGFDenoiser::initWithDevice(MPSSVGFDenoiser::alloc(), device) },
        }
    }

    /// Encode independent frames without motion vectors or previous depth history.
    /// The caller must finish the previous submission before reusing this denoiser.
    pub fn encode(
        &mut self,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        targets: &Targets,
    ) -> Output {
        let encode = |denoiser: &MPSSVGFDenoiser, source: &ProtocolObject<dyn MTLTexture>| unsafe {
            denoiser.releaseTemporaryTextures();
            denoiser.encodeToCommandBuffer_sourceTexture_motionVectorTexture_depthNormalTexture_previousDepthNormalTexture(command, source, None, &targets.depth_normal, None)
        };
        Output {
            beauty: encode(&self.beauty, &targets.beauty),
            background: encode(&self.background, &targets.background),
            alpha: encode(&self.alpha, &targets.alpha),
        }
    }
}

fn allocate_denoiser_texture(
    device: &ProtocolObject<dyn MTLDevice>,
    width: u32,
    height: u32,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, String> {
    let descriptor = MTLTextureDescriptor::new();
    descriptor.setTextureType(MTLTextureType::Type2D);
    descriptor.setPixelFormat(MTLPixelFormat::RGBA32Float);
    unsafe {
        descriptor.setWidth(width as usize);
        descriptor.setHeight(height as usize);
    }
    descriptor.setStorageMode(MTLStorageMode::Private);
    descriptor.setUsage(MTLTextureUsage::ShaderRead | MTLTextureUsage::ShaderWrite);
    device
        .newTextureWithDescriptor(&descriptor)
        .ok_or_else(|| "Could not allocate Metal denoiser texture".into())
}
