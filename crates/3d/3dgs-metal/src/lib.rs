#![cfg(target_os = "macos")]

use std::{mem::size_of, ptr::NonNull};

use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBlendFactor, MTLBlitCommandEncoder, MTLClearColor, MTLCommandBuffer, MTLCommandBufferStatus,
    MTLCommandEncoder, MTLCommandQueue, MTLComputeCommandEncoder, MTLComputePipelineState,
    MTLDevice, MTLFunction, MTLLibrary, MTLLoadAction, MTLOrigin, MTLPixelFormat, MTLPrimitiveType,
    MTLRenderCommandEncoder, MTLRenderPassDescriptor, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState, MTLSize, MTLStorageMode, MTLStoreAction, MTLTexture,
    MTLTextureDescriptor, MTLTextureType, MTLTextureUsage, MTLViewport,
};

include!(concat!(env!("OUT_DIR"), "/gaussian_metal.rs"));

const SORT_THREADS: usize = PREPARE_SORT_THREADS[0];
const RADIX_SIZE: usize = RADIX_PREFIX_THREADS[0];
const RADIX_BITS: u32 = RADIX_SIZE.ilog2();
const RADIX_PASSES: u32 = u32::BITS / RADIX_BITS;
const OUTPUT_ROW_ALIGNMENT: usize = 256;

pub struct Rendered {
    pub buffer: shrimply_render_metal::Buffer,
    pub row_bytes: usize,
}

pub struct Renderer {
    _compute_library: Retained<ProtocolObject<dyn MTLLibrary>>,
    _raster_library: Retained<ProtocolObject<dyn MTLLibrary>>,
    prepare_depth_sort: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    radix_histogram: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    radix_prefix: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    radix_scatter: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    accumulate: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    resolve: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    uploaded: Option<UploadedCloud>,
    target: Option<Target>,
}

struct UploadedCloud {
    identity: shrimply_asset::AssetSnapshot,
    gaussians: shrimply_render_metal::Buffer,
    higher_order: shrimply_render_metal::Buffer,
    sort_keys: shrimply_render_metal::Buffer,
    sorted_indices: shrimply_render_metal::Buffer,
    scratch_keys: shrimply_render_metal::Buffer,
    scratch_indices: shrimply_render_metal::Buffer,
    sort_group_offsets: shrimply_render_metal::Buffer,
    draw_indirect: shrimply_render_metal::Buffer,
    count: u32,
    group_count: u32,
}

struct Target {
    width: u32,
    height: u32,
    accumulation: Retained<ProtocolObject<dyn MTLTexture>>,
    color: Retained<ProtocolObject<dyn MTLTexture>>,
}

impl Renderer {
    pub fn new(metal: &shrimply_render_metal::Renderer) -> Result<Self, String> {
        let device = metal.device();
        let compute_library = device
            .newLibraryWithSource_options_error(
                &NSString::from_str(GAUSSIAN_COMPUTE_METAL_SOURCE),
                None,
            )
            .map_err(|error| format!("Compile shared Gaussian Metal compute module: {error}"))?;
        let raster_library = device
            .newLibraryWithSource_options_error(
                &NSString::from_str(GAUSSIAN_RASTER_METAL_SOURCE),
                None,
            )
            .map_err(|error| format!("Compile shared Gaussian Metal raster module: {error}"))?;
        let prepare_depth_sort = compute_pipeline(device, &compute_library, "prepare_depth_sort")?;
        let radix_histogram = compute_pipeline(device, &compute_library, "radix_histogram_pass")?;
        let radix_prefix = compute_pipeline(device, &compute_library, "radix_prefix_pass")?;
        let radix_scatter = compute_pipeline(device, &compute_library, "radix_scatter_pass")?;
        let accumulate = render_pipeline(
            device,
            &raster_library,
            "gaussian_vertex_metal",
            "gaussian_fragment",
            MTLPixelFormat::RGBA16Float,
            true,
        )?;
        let resolve = render_pipeline(
            device,
            &raster_library,
            "resolve_vertex",
            "resolve_fragment",
            MTLPixelFormat::RGBA8Unorm,
            false,
        )?;
        Ok(Self {
            _compute_library: compute_library,
            _raster_library: raster_library,
            prepare_depth_sort,
            radix_histogram,
            radix_prefix,
            radix_scatter,
            accumulate,
            resolve,
            uploaded: None,
            target: None,
        })
    }

    pub fn render(
        &mut self,
        metal: &shrimply_render_metal::Renderer,
        session: &shrimply_3dgs::RenderSession,
        width: u32,
        height: u32,
        params: &shrimply_3dgs::RenderParams,
    ) -> Result<Rendered, String> {
        if !session.identity().is_current() {
            return Err(format!(
                "Gaussian source changed on disk: {}",
                session.path().display()
            ));
        }
        let width = width.max(1);
        let height = height.max(1);
        self.ensure_uploaded(metal, session)?;
        self.ensure_target(metal.device(), width, height)?;
        let uniforms = params
            .uniforms(session.cloud(), width, height)
            .map_err(|error| error.to_string())?;
        let uniform_buffer = metal.upload(bytes_of(&uniforms))?;
        let row_bytes = align_up(
            usize::try_from(width)
                .map_err(|_| "Gaussian output width is too large")?
                .checked_mul(size_of::<u32>())
                .ok_or("Gaussian output row size overflow")?,
            OUTPUT_ROW_ALIGNMENT,
        )?;
        let output_size = row_bytes
            .checked_mul(
                usize::try_from(height).map_err(|_| "Gaussian output height is too large")?,
            )
            .ok_or("Gaussian output size overflow")?;
        let output = metal.allocate(output_size)?;

        let uploaded = self.uploaded.as_ref().expect("Gaussian cloud was uploaded");
        uploaded
            .draw_indirect
            .write_bytes(bytes_slice(&[4_u32, 0, 0, 0]))?;
        let command = metal
            .queue()
            .commandBuffer()
            .ok_or("Could not create Gaussian Metal command buffer")?;
        let target = self.target.as_ref().expect("Gaussian target was allocated");

        self.encode_compute(
            &command,
            &self.prepare_depth_sort,
            shrimply_3dgs::shader::SortConstants {
                count: uploaded.count,
                shift: 0,
                read_scratch: 0,
                group_count: uploaded.group_count,
            },
            uploaded.group_count,
            PREPARE_SORT_THREADS,
            uploaded,
            &uniform_buffer,
        )?;
        for pass in 0..RADIX_PASSES {
            let constants = shrimply_3dgs::shader::SortConstants {
                count: uploaded.count,
                shift: pass * RADIX_BITS,
                read_scratch: pass & 1,
                group_count: uploaded.group_count,
            };
            self.encode_compute(
                &command,
                &self.radix_histogram,
                constants,
                uploaded.group_count,
                RADIX_HISTOGRAM_THREADS,
                uploaded,
                &uniform_buffer,
            )?;
            self.encode_compute(
                &command,
                &self.radix_prefix,
                constants,
                1,
                RADIX_PREFIX_THREADS,
                uploaded,
                &uniform_buffer,
            )?;
            self.encode_compute(
                &command,
                &self.radix_scatter,
                constants,
                uploaded.group_count,
                RADIX_SCATTER_THREADS,
                uploaded,
                &uniform_buffer,
            )?;
        }

        self.encode_accumulation(&command, uploaded, &uniform_buffer, target, width, height)?;
        self.encode_resolve(&command, uploaded, &uniform_buffer, target, width, height)?;
        let blit = command
            .blitCommandEncoder()
            .ok_or("Could not create Gaussian Metal readback encoder")?;
        unsafe {
            blit.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage(
                &target.color,
                0,
                0,
                MTLOrigin { x: 0, y: 0, z: 0 },
                MTLSize { width: width as usize, height: height as usize, depth: 1 },
                &output.metal(),
                0,
                row_bytes,
                output_size,
            );
        }
        blit.endEncoding();
        command.commit();
        command.waitUntilCompleted();
        if command.status() == MTLCommandBufferStatus::Error {
            return Err(command.error().map_or_else(
                || "Gaussian Metal command failed".to_string(),
                |error| format!("Gaussian Metal command failed: {error}"),
            ));
        }
        Ok(Rendered {
            buffer: output,
            row_bytes,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_compute(
        &self,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        pipeline: &ProtocolObject<dyn MTLComputePipelineState>,
        constants: shrimply_3dgs::shader::SortConstants,
        group_count: u32,
        threads: [usize; 3],
        uploaded: &UploadedCloud,
        uniforms: &shrimply_render_metal::Buffer,
    ) -> Result<(), String> {
        let encoder = command
            .computeCommandEncoder()
            .ok_or("Could not create Gaussian Metal compute encoder")?;
        encoder.setComputePipelineState(pipeline);
        bind_compute_resources(&encoder, uploaded, uniforms, &constants);
        encoder.dispatchThreadgroups_threadsPerThreadgroup(
            MTLSize {
                width: group_count as usize,
                height: 1,
                depth: 1,
            },
            MTLSize {
                width: threads[0],
                height: threads[1],
                depth: threads[2],
            },
        );
        encoder.endEncoding();
        Ok(())
    }

    fn encode_accumulation(
        &self,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        uploaded: &UploadedCloud,
        uniforms: &shrimply_render_metal::Buffer,
        target: &Target,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let pass = render_pass(&target.accumulation);
        let encoder = command
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or("Could not create Gaussian Metal accumulation encoder")?;
        encoder.setRenderPipelineState(&self.accumulate);
        set_viewport(&encoder, width, height);
        bind_render_resources(&encoder, uploaded, uniforms, &target.accumulation);
        unsafe {
            encoder.drawPrimitives_indirectBuffer_indirectBufferOffset(
                MTLPrimitiveType::TriangleStrip,
                &uploaded.draw_indirect.metal(),
                0,
            );
        }
        encoder.endEncoding();
        Ok(())
    }

    fn encode_resolve(
        &self,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        uploaded: &UploadedCloud,
        uniforms: &shrimply_render_metal::Buffer,
        target: &Target,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let pass = render_pass(&target.color);
        let encoder = command
            .renderCommandEncoderWithDescriptor(&pass)
            .ok_or("Could not create Gaussian Metal resolve encoder")?;
        encoder.setRenderPipelineState(&self.resolve);
        set_viewport(&encoder, width, height);
        bind_render_resources(&encoder, uploaded, uniforms, &target.accumulation);
        unsafe {
            encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::Triangle, 0, 3);
        }
        encoder.endEncoding();
        Ok(())
    }

    fn ensure_uploaded(
        &mut self,
        metal: &shrimply_render_metal::Renderer,
        session: &shrimply_3dgs::RenderSession,
    ) -> Result<(), String> {
        if self
            .uploaded
            .as_ref()
            .is_some_and(|uploaded| &uploaded.identity == session.identity())
        {
            return Ok(());
        }
        let count = u32::try_from(session.cloud().gaussians.len())
            .map_err(|_| "Gaussian count exceeds Metal draw limits")?;
        if count == 0 {
            return Err("Gaussian source contains no splats".into());
        }
        let gaussians = session
            .cloud()
            .gaussians
            .iter()
            .map(shrimply_3dgs::shader::GaussianSource::from_gaussian)
            .collect::<Vec<_>>();
        let group_count = count.div_ceil(SORT_THREADS as u32);
        let sort_bytes = usize::try_from(count)
            .map_err(|_| "Gaussian count is too large")?
            .checked_mul(size_of::<u32>())
            .ok_or("Gaussian sort buffer size overflow")?;
        let group_offset_bytes = usize::try_from(group_count)
            .map_err(|_| "Gaussian group count is too large")?
            .checked_mul(RADIX_SIZE)
            .and_then(|value| value.checked_mul(size_of::<u32>()))
            .ok_or("Gaussian radix buffer size overflow")?;
        let higher_order = if session.cloud().higher_order_spherical_harmonics.is_empty() {
            metal.upload(&0_f32.to_ne_bytes())?
        } else {
            metal.upload(bytes_slice(
                &session.cloud().higher_order_spherical_harmonics,
            ))?
        };
        self.uploaded = Some(UploadedCloud {
            identity: session.identity().clone(),
            gaussians: metal.upload(bytes_slice(&gaussians))?,
            higher_order,
            sort_keys: metal.allocate(sort_bytes)?,
            sorted_indices: metal.allocate(sort_bytes)?,
            scratch_keys: metal.allocate(sort_bytes)?,
            scratch_indices: metal.allocate(sort_bytes)?,
            sort_group_offsets: metal.allocate(group_offset_bytes)?,
            draw_indirect: metal.upload(bytes_slice(&[4_u32, 0, 0, 0]))?,
            count,
            group_count,
        });
        Ok(())
    }

    fn ensure_target(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        if self
            .target
            .as_ref()
            .is_some_and(|target| target.width == width && target.height == height)
        {
            return Ok(());
        }
        self.target = Some(Target {
            width,
            height,
            accumulation: texture(
                device,
                width,
                height,
                MTLPixelFormat::RGBA16Float,
                MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead,
            )?,
            color: texture(
                device,
                width,
                height,
                MTLPixelFormat::RGBA8Unorm,
                MTLTextureUsage::RenderTarget,
            )?,
        });
        Ok(())
    }
}

fn compute_pipeline(
    device: &ProtocolObject<dyn MTLDevice>,
    library: &ProtocolObject<dyn MTLLibrary>,
    name: &str,
) -> Result<Retained<ProtocolObject<dyn MTLComputePipelineState>>, String> {
    let function = function(library, name)?;
    device
        .newComputePipelineStateWithFunction_error(&function)
        .map_err(|error| format!("Create Gaussian Metal compute pipeline {name}: {error}"))
}

fn render_pipeline(
    device: &ProtocolObject<dyn MTLDevice>,
    library: &ProtocolObject<dyn MTLLibrary>,
    vertex: &str,
    fragment: &str,
    format: MTLPixelFormat,
    additive: bool,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, String> {
    let descriptor = MTLRenderPipelineDescriptor::new();
    let vertex_function = function(library, vertex)?;
    let fragment_function = function(library, fragment)?;
    descriptor.setVertexFunction(Some(&vertex_function));
    descriptor.setFragmentFunction(Some(&fragment_function));
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(format);
    if additive {
        attachment.setBlendingEnabled(true);
        attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
        attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
        attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
        attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    }
    device
        .newRenderPipelineStateWithDescriptor_error(&descriptor)
        .map_err(|error| {
            format!("Create Gaussian Metal render pipeline {vertex}/{fragment}: {error}")
        })
}

fn function(
    library: &ProtocolObject<dyn MTLLibrary>,
    name: &str,
) -> Result<Retained<ProtocolObject<dyn MTLFunction>>, String> {
    library
        .newFunctionWithName(&NSString::from_str(name))
        .ok_or_else(|| format!("Shared Gaussian Metal module has no {name} entry"))
}

fn texture(
    device: &ProtocolObject<dyn MTLDevice>,
    width: u32,
    height: u32,
    format: MTLPixelFormat,
    usage: MTLTextureUsage,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, String> {
    let descriptor = MTLTextureDescriptor::new();
    descriptor.setTextureType(MTLTextureType::Type2D);
    descriptor.setPixelFormat(format);
    unsafe {
        descriptor.setWidth(width as usize);
        descriptor.setHeight(height as usize);
    }
    descriptor.setStorageMode(MTLStorageMode::Private);
    descriptor.setUsage(usage);
    device
        .newTextureWithDescriptor(&descriptor)
        .ok_or("Could not allocate Gaussian Metal render target".into())
}

fn render_pass(texture: &ProtocolObject<dyn MTLTexture>) -> Retained<MTLRenderPassDescriptor> {
    let pass = MTLRenderPassDescriptor::renderPassDescriptor();
    let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setTexture(Some(texture));
    attachment.setLoadAction(MTLLoadAction::Clear);
    attachment.setStoreAction(MTLStoreAction::Store);
    attachment.setClearColor(MTLClearColor {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 0.0,
    });
    pass
}

fn set_viewport(encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>, width: u32, height: u32) {
    encoder.setViewport(MTLViewport {
        originX: 0.0,
        originY: 0.0,
        width: f64::from(width),
        height: f64::from(height),
        znear: 0.0,
        zfar: 1.0,
    });
}

fn compute_buffers<'a>(
    uploaded: &'a UploadedCloud,
    uniforms: &'a shrimply_render_metal::Buffer,
) -> [(usize, &'a shrimply_render_metal::Buffer); 8] {
    [
        (COMPUTE_GAUSSIANS_BUFFER, &uploaded.gaussians),
        (COMPUTE_UNIFORMS_BUFFER, uniforms),
        (COMPUTE_SORT_KEYS_BUFFER, &uploaded.sort_keys),
        (COMPUTE_SORTED_INDICES_BUFFER, &uploaded.sorted_indices),
        (COMPUTE_SCRATCH_KEYS_BUFFER, &uploaded.scratch_keys),
        (COMPUTE_SCRATCH_INDICES_BUFFER, &uploaded.scratch_indices),
        (
            COMPUTE_SORT_GROUP_OFFSETS_BUFFER,
            &uploaded.sort_group_offsets,
        ),
        (COMPUTE_DRAW_INDIRECT_BUFFER, &uploaded.draw_indirect),
    ]
}

fn raster_buffers<'a>(
    uploaded: &'a UploadedCloud,
    uniforms: &'a shrimply_render_metal::Buffer,
) -> [(usize, &'a shrimply_render_metal::Buffer); 4] {
    [
        (RASTER_GAUSSIANS_BUFFER, &uploaded.gaussians),
        (RASTER_HIGHER_ORDER_BUFFER, &uploaded.higher_order),
        (RASTER_UNIFORMS_BUFFER, uniforms),
        (RASTER_SORTED_INDICES_BUFFER, &uploaded.sorted_indices),
    ]
}

fn bind_compute_resources(
    encoder: &ProtocolObject<dyn MTLComputeCommandEncoder>,
    uploaded: &UploadedCloud,
    uniforms: &shrimply_render_metal::Buffer,
    constants: &shrimply_3dgs::shader::SortConstants,
) {
    unsafe {
        for (index, buffer) in compute_buffers(uploaded, uniforms) {
            encoder.setBuffer_offset_atIndex(Some(&buffer.metal()), 0, index);
        }
        encoder.setBytes_length_atIndex(
            NonNull::from(constants).cast(),
            size_of::<shrimply_3dgs::shader::SortConstants>(),
            COMPUTE_SORT_CONSTANTS_BUFFER,
        );
    }
}

fn bind_render_resources(
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
    uploaded: &UploadedCloud,
    uniforms: &shrimply_render_metal::Buffer,
    accumulation: &ProtocolObject<dyn MTLTexture>,
) {
    unsafe {
        for (index, buffer) in raster_buffers(uploaded, uniforms) {
            encoder.setVertexBuffer_offset_atIndex(Some(&buffer.metal()), 0, index);
            encoder.setFragmentBuffer_offset_atIndex(Some(&buffer.metal()), 0, index);
        }
        encoder.setVertexTexture_atIndex(Some(accumulation), RASTER_ACCUMULATION_TEXTURE);
        encoder.setFragmentTexture_atIndex(Some(accumulation), RASTER_ACCUMULATION_TEXTURE);
    }
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    let remainder = value % alignment;
    if remainder == 0 {
        Ok(value)
    } else {
        value
            .checked_add(alignment - remainder)
            .ok_or_else(|| "Gaussian output row alignment overflow".into())
    }
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(std::ptr::from_ref(value).cast(), size_of::<T>()) }
}

fn bytes_slice<T>(values: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}
