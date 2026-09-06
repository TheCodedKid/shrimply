#![cfg(target_os = "macos")]

use std::mem::size_of;

use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_foundation::{NSArray, NSString};
use objc2_metal::{
    MTLAccelerationStructure, MTLAccelerationStructureCommandEncoder,
    MTLAccelerationStructureInstanceDescriptor, MTLAccelerationStructureInstanceOptions,
    MTLAccelerationStructureTriangleGeometryDescriptor, MTLAttributeFormat, MTLCommandBuffer,
    MTLCommandBufferStatus, MTLCommandEncoder, MTLCommandQueue, MTLComputeCommandEncoder,
    MTLComputePipelineState, MTLDevice, MTLInstanceAccelerationStructureDescriptor, MTLLibrary,
    MTLPackedFloat3, MTLPackedFloat4x3, MTLPrimitiveAccelerationStructureDescriptor, MTLResource,
    MTLResourceOptions, MTLResourceUsage, MTLSize,
};

include!(concat!(env!("OUT_DIR"), "/obj_metal.rs"));

pub struct Rendered {
    pub buffer: shrimply_render_metal::Buffer,
    pub row_bytes: usize,
}

struct UploadedGeometry {
    identity: shrimply_render_3d::GeometryIdentity,
    positions: shrimply_render_metal::Buffer,
    normals: shrimply_render_metal::Buffer,
    blases: Vec<Retained<ProtocolObject<dyn MTLAccelerationStructure>>>,
}

pub struct Renderer {
    _library: Retained<ProtocolObject<dyn MTLLibrary>>,
    pipeline: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    uploaded: Option<UploadedGeometry>,
}

#[repr(C)]
struct OutputSize {
    width: u32,
    height: u32,
}

#[repr(C, align(16))]
struct ComputeMaterial {
    base_color: [f32; 4],
    metallic_roughness_normal_alpha: [f32; 4],
    flags: [u32; 4],
}

impl Renderer {
    pub fn new(compute: &shrimply_render_metal::Renderer) -> Result<Self, String> {
        if !compute.supports_ray_tracing() {
            return Err("OBJ rendering requires Metal ray tracing".to_string());
        }
        let device = compute.device();
        let library = device
            .newLibraryWithSource_options_error(&NSString::from_str(OBJ_METAL_SOURCE), None)
            .map_err(|error| format!("Compile shared OBJ Metal shader: {error}"))?;
        let function = library
            .newFunctionWithName(&NSString::from_str("obj_compute"))
            .ok_or("Shared OBJ Metal shader omitted obj_compute")?;
        let pipeline = device
            .newComputePipelineStateWithFunction_error(&function)
            .map_err(|error| format!("Create shared OBJ Metal pipeline: {error}"))?;
        Ok(Self {
            _library: library,
            pipeline,
            uploaded: None,
        })
    }

    pub fn render(
        &mut self,
        metal: &shrimply_render_metal::Renderer,
        plan: &shrimply_video_core::obj::Prepared,
    ) -> Result<Rendered, String> {
        if plan.params.path_tracing != shrimply_render_3d::obj::PathTracingMode::Off {
            return Err("Path-traced OBJ rendering is not yet connected to Metal".into());
        }
        self.ensure_geometry(metal, &plan.session)?;
        let width = plan.width.max(1);
        let height = plan.height.max(1);
        let row_bytes = usize::try_from(width)
            .map_err(|_| "OBJ width exceeds Metal limits")?
            .checked_mul(size_of::<u32>())
            .ok_or("OBJ output row size overflow")?;
        let output_bytes = row_bytes
            .checked_mul(usize::try_from(height).map_err(|_| "OBJ height exceeds Metal limits")?)
            .ok_or("OBJ output size overflow")?;
        let scene = metal.upload(bytes_of(&plan.uniforms))?;
        let output_size = metal.upload(bytes_of(&OutputSize { width, height }))?;
        let compute_materials = plan
            .session
            .materials()
            .iter()
            .map(|material| ComputeMaterial {
                base_color: material.base_color_factor,
                metallic_roughness_normal_alpha: material.metallic_roughness_normal_alpha,
                flags: material.flags,
            })
            .collect::<Vec<_>>();
        let materials = metal.upload(bytes_slice(&compute_materials))?;
        let mesh_instances = metal.upload(bytes_slice(plan.session.mesh_instances()))?;
        let output = metal.allocate(output_bytes)?;
        let instance_descriptors = plan
            .session
            .acceleration_instances()
            .iter()
            .map(|instance| MTLAccelerationStructureInstanceDescriptor {
                transformationMatrix: packed_transform(instance.transform),
                options: MTLAccelerationStructureInstanceOptions::None,
                mask: u32::from(u8::MAX),
                intersectionFunctionTableOffset: 0,
                accelerationStructureIndex: instance.geometry_index,
            })
            .collect::<Vec<_>>();
        let instance_buffer = metal.upload(bytes_slice(&instance_descriptors))?;
        let uploaded = self.uploaded.as_ref().expect("OBJ geometry uploaded");
        let tlas_descriptor = MTLInstanceAccelerationStructureDescriptor::new();
        tlas_descriptor.setInstanceDescriptorBuffer(Some(&instance_buffer.metal()));
        tlas_descriptor.setInstanceCount(instance_descriptors.len());
        let blases = NSArray::from_retained_slice(&uploaded.blases);
        tlas_descriptor.setInstancedAccelerationStructures(Some(&blases));
        let tlas = allocate_acceleration(metal.device(), &tlas_descriptor)?;
        let tlas_scratch = scratch_buffer(metal.device(), tlas.scratch_size)?;
        let command = metal
            .queue()
            .commandBuffer()
            .ok_or("Could not create OBJ Metal command buffer")?;
        let acceleration_encoder = command
            .accelerationStructureCommandEncoder()
            .ok_or("Could not create OBJ Metal acceleration encoder")?;
        acceleration_encoder
            .buildAccelerationStructure_descriptor_scratchBuffer_scratchBufferOffset(
                &tlas.structure,
                &tlas_descriptor,
                &tlas_scratch,
                0,
            );
        acceleration_encoder.endEncoding();
        let encoder = command
            .computeCommandEncoder()
            .ok_or("Could not create OBJ Metal compute encoder")?;
        encoder.setComputePipelineState(&self.pipeline);
        unsafe {
            encoder.setBuffer_offset_atIndex(Some(&scene.metal()), 0, OBJ_SCENE_BUFFER);
            encoder.setBuffer_offset_atIndex(Some(&output_size.metal()), 0, OBJ_OUTPUT_SIZE_BUFFER);
            encoder.setAccelerationStructure_atBufferIndex(
                Some(&tlas.structure),
                OBJ_ACCELERATION_BUFFER,
            );
            encoder.setBuffer_offset_atIndex(
                Some(&uploaded.positions.metal()),
                0,
                OBJ_POSITIONS_BUFFER,
            );
            encoder.setBuffer_offset_atIndex(
                Some(&uploaded.normals.metal()),
                0,
                OBJ_NORMALS_BUFFER,
            );
            encoder.setBuffer_offset_atIndex(Some(&materials.metal()), 0, OBJ_MATERIALS_BUFFER);
            encoder.setBuffer_offset_atIndex(
                Some(&mesh_instances.metal()),
                0,
                OBJ_INSTANCES_BUFFER,
            );
            encoder.setBuffer_offset_atIndex(Some(&output.metal()), 0, OBJ_OUTPUT_BUFFER);
        }
        for buffer in [
            &scene,
            &output_size,
            &uploaded.positions,
            &uploaded.normals,
            &materials,
            &mesh_instances,
            &output,
        ] {
            let resource: &ProtocolObject<dyn MTLResource> =
                ProtocolObject::from_ref(&**buffer.metal());
            encoder.useResource_usage(resource, MTLResourceUsage::Read | MTLResourceUsage::Write);
        }
        let tlas_resource: &ProtocolObject<dyn MTLResource> =
            ProtocolObject::from_ref(&*tlas.structure);
        encoder.useResource_usage(tlas_resource, MTLResourceUsage::Read);
        for blas in &uploaded.blases {
            let resource: &ProtocolObject<dyn MTLResource> = ProtocolObject::from_ref(&**blas);
            encoder.useResource_usage(resource, MTLResourceUsage::Read);
        }
        encoder.dispatchThreads_threadsPerThreadgroup(
            MTLSize {
                width: width as usize,
                height: height as usize,
                depth: 1,
            },
            MTLSize {
                width: OBJ_THREADS[0],
                height: OBJ_THREADS[1],
                depth: OBJ_THREADS[2],
            },
        );
        encoder.endEncoding();
        command.commit();
        command.waitUntilCompleted();
        if command.status() == MTLCommandBufferStatus::Error {
            return Err(command.error().map_or_else(
                || "OBJ Metal command failed".to_string(),
                |error| format!("OBJ Metal command failed: {error}"),
            ));
        }
        Ok(Rendered {
            buffer: output,
            row_bytes,
        })
    }

    fn ensure_geometry(
        &mut self,
        metal: &shrimply_render_metal::Renderer,
        session: &shrimply_render_3d::ObjRenderSession,
    ) -> Result<(), String> {
        if self
            .uploaded
            .as_ref()
            .is_some_and(|uploaded| &uploaded.identity == session.geometry_identity())
        {
            return Ok(());
        }
        let positions = metal.upload(bytes_slice(session.positions()))?;
        let normals = metal.upload(bytes_slice(session.normals()))?;
        let mut descriptors = Vec::with_capacity(session.geometries().len());
        for geometry in session.geometries() {
            let mut triangles = Vec::with_capacity(geometry.geometry_count as usize);
            for slot in 0..geometry.geometry_count as usize {
                let descriptor = MTLAccelerationStructureTriangleGeometryDescriptor::new();
                descriptor.setVertexBuffer(Some(&positions.metal()));
                unsafe {
                    descriptor.setVertexBufferOffset(
                        geometry.vertex_offsets[slot] as usize * size_of::<[f32; 4]>(),
                    );
                }
                descriptor.setVertexStride(size_of::<[f32; 4]>());
                descriptor.setVertexFormat(MTLAttributeFormat::Float3);
                descriptor.setTriangleCount(geometry.primitive_counts[slot] as usize);
                descriptor.setOpaque(geometry.opaque[slot]);
                triangles.push(descriptor.into_super());
            }
            let descriptor = MTLPrimitiveAccelerationStructureDescriptor::new();
            descriptor.setGeometryDescriptors(Some(&NSArray::from_retained_slice(&triangles)));
            descriptors.push(descriptor);
        }
        let command = metal
            .queue()
            .commandBuffer()
            .ok_or("Could not create OBJ Metal BLAS command buffer")?;
        let encoder = command
            .accelerationStructureCommandEncoder()
            .ok_or("Could not create OBJ Metal BLAS encoder")?;
        let mut blases = Vec::with_capacity(descriptors.len());
        let mut scratch = Vec::with_capacity(descriptors.len());
        for descriptor in &descriptors {
            let allocation = allocate_acceleration(metal.device(), descriptor)?;
            let buffer = scratch_buffer(metal.device(), allocation.scratch_size)?;
            encoder.buildAccelerationStructure_descriptor_scratchBuffer_scratchBufferOffset(
                &allocation.structure,
                descriptor,
                &buffer,
                0,
            );
            blases.push(allocation.structure);
            scratch.push(buffer);
        }
        encoder.endEncoding();
        command.commit();
        command.waitUntilCompleted();
        if command.status() == MTLCommandBufferStatus::Error {
            return Err(command.error().map_or_else(
                || "OBJ Metal BLAS build failed".to_string(),
                |error| format!("OBJ Metal BLAS build failed: {error}"),
            ));
        }
        self.uploaded = Some(UploadedGeometry {
            identity: session.geometry_identity().clone(),
            positions,
            normals,
            blases,
        });
        Ok(())
    }
}

struct AccelerationAllocation {
    structure: Retained<ProtocolObject<dyn MTLAccelerationStructure>>,
    scratch_size: usize,
}

fn allocate_acceleration(
    device: &ProtocolObject<dyn MTLDevice>,
    descriptor: &objc2_metal::MTLAccelerationStructureDescriptor,
) -> Result<AccelerationAllocation, String> {
    let sizes = device.accelerationStructureSizesWithDescriptor(descriptor);
    let structure = device
        .newAccelerationStructureWithSize(sizes.accelerationStructureSize)
        .ok_or("Could not allocate OBJ Metal acceleration structure")?;
    Ok(AccelerationAllocation {
        structure,
        scratch_size: sizes.buildScratchBufferSize,
    })
}

fn scratch_buffer(
    device: &ProtocolObject<dyn MTLDevice>,
    length: usize,
) -> Result<Retained<ProtocolObject<dyn objc2_metal::MTLBuffer>>, String> {
    device
        .newBufferWithLength_options(length, MTLResourceOptions::StorageModePrivate)
        .ok_or_else(|| "Could not allocate OBJ Metal acceleration scratch buffer".into())
}

fn packed_transform(matrix: [f32; 16]) -> MTLPackedFloat4x3 {
    MTLPackedFloat4x3 {
        columns: std::array::from_fn(|column| MTLPackedFloat3 {
            x: matrix[column * 4],
            y: matrix[column * 4 + 1],
            z: matrix[column * 4 + 2],
        }),
    }
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(std::ptr::from_ref(value).cast(), size_of::<T>()) }
}

fn bytes_slice<T>(values: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}
