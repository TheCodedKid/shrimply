import render_3d_abi;
import render_3d_normals;
import render_3d_pbr;
import render_3d_toon;

static const float RAY_MINIMUM = 0.001;
static const float RAY_MAXIMUM = 100000.0;

struct OutputSize
{
    uint width;
    uint height;
};

struct ComputeMaterial
{
    float4 base_color;
    float4 metallic_roughness_normal_alpha;
    uint4 flags;
};

ConstantBuffer<SceneUniforms> scene;
ConstantBuffer<OutputSize> output_size;
RaytracingAccelerationStructure scene_acceleration;
StructuredBuffer<float4> positions;
StructuredBuffer<float4> normals;
StructuredBuffer<ComputeMaterial> materials;
StructuredBuffer<MeshInstance> mesh_instances;
RWStructuredBuffer<uint> output_pixels;

uint pack_color(float4 color)
{
    uint4 value = uint4(round(saturate(color) * 255.0));
    return value.r | (value.g << 8) | (value.b << 16) | (value.a << 24);
}

[shader("compute")]
[numthreads(8, 8, 1)]
void obj_compute(uint3 thread_id : SV_DispatchThreadID)
{
    uint2 pixel = thread_id.xy;
    uint2 dimensions = uint2(output_size.width, output_size.height);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y)
        return;

    float2 uv = (float2(pixel) + 0.5) / float2(dimensions);
    float2 ndc = float2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    float4 near_point = mul(scene.inverse_view_projection, float4(ndc, 0.0, 1.0));
    float4 far_point = mul(scene.inverse_view_projection, float4(ndc, 1.0, 1.0));
    float3 world_origin = near_point.xyz / near_point.w;
    float3 world_direction = normalize(far_point.xyz / far_point.w - world_origin);

    RayDesc ray;
    ray.Origin = mul(scene.model_inverse, float4(world_origin, 1.0)).xyz;
    ray.Direction = normalize(mul(scene.model_inverse, float4(world_direction, 0.0)).xyz);
    ray.TMin = RAY_MINIMUM;
    ray.TMax = RAY_MAXIMUM;

    RayQuery<RAY_FLAG_FORCE_NON_OPAQUE | RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES> query;
    query.TraceRayInline(scene_acceleration, RAY_FLAG_NONE, 0xff, ray);
    while (query.Proceed())
    {
        if (query.CandidateType() == CANDIDATE_NON_OPAQUE_TRIANGLE)
            query.CommitNonOpaqueTriangleHit();
    }

    float4 color = float4(scene.environment_color.rgb, 0.0);
    if (query.CommittedStatus() == COMMITTED_TRIANGLE_HIT)
    {
        uint instance_index = query.CommittedInstanceIndex();
        uint geometry_index = query.CommittedGeometryIndex();
        uint primitive_index = query.CommittedPrimitiveIndex();
        float2 committed = query.CommittedTriangleBarycentrics();
        float3 barycentric = float3(
            1.0 - committed.x - committed.y,
            committed.x,
            committed.y);
        MeshInstance instance = mesh_instances[instance_index];
        uint2 offsets = geometry_index == 0
            ? instance.geometry_offsets.xy
            : instance.geometry_offsets.zw;
        uint first = offsets.x + primitive_index * 3;
        uint material_index = offsets.y + primitive_index;
        float3 object_normal = normalize(
            normals[first].xyz * barycentric.x
            + normals[first + 1].xyz * barycentric.y
            + normals[first + 2].xyz * barycentric.z);
        float3 normal = normalize(mul(
            mul(scene.normal, instance.normal),
            float4(object_normal, 0.0)).xyz);
        ComputeMaterial material = materials[material_index];
        float3 base_color = material.base_color.rgb;
        float3 object_position = ray.Origin + ray.Direction * query.CommittedRayT();
        float3 world_position = mul(scene.model, float4(object_position, 1.0)).xyz;
        float3 view = normalize(scene.camera_position.xyz - world_position);
        float metallic = material.metallic_roughness_normal_alpha.x;
        float roughness = material.metallic_roughness_normal_alpha.y;
        float3 shaded = base_color * 0.03;
        for (uint light_index = 0; light_index < scene.sun_light_count; ++light_index)
        {
            SunLight light = scene.sun_lights[light_index];
            shaded += pbr_light(
                normal,
                view,
                normalize(-light.direction_angular_radius.xyz),
                light.color_intensity.rgb * light.color_intensity.w,
                base_color,
                metallic,
                roughness,
                float3(1.0),
                scene.pbr);
        }
        for (uint light_index = 0; light_index < scene.point_light_count; ++light_index)
        {
            PointLight light = scene.point_lights[light_index];
            float3 delta = light.position_radius.xyz - world_position;
            float distance_squared = dot(delta, delta);
            float distance = sqrt(distance_squared);
            if (distance >= light.range_padding.x)
                continue;
            float falloff = pow(1.0 - distance / light.range_padding.x, 2.0)
                / max(distance_squared, 0.01);
            shaded += pbr_light(
                normal,
                view,
                delta / max(distance, RAY_MINIMUM),
                light.color_intensity.rgb * light.color_intensity.w * falloff,
                base_color,
                metallic,
                roughness,
                float3(1.0),
                scene.pbr);
        }
        if (scene.toon.shading_model == ShadingModel::Toon)
            shaded = toon_palette_quantize(shaded, scene.toon.color_levels);
        shaded *= scene.material.z;
        shaded = float3(
            linear_to_srgb(aces_tone_map(shaded.r)),
            linear_to_srgb(aces_tone_map(shaded.g)),
            linear_to_srgb(aces_tone_map(shaded.b)));
        color = float4(shaded, material.base_color.a);
    }
    output_pixels[pixel.y * dimensions.x + pixel.x] = pack_color(color);
}
