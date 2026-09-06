import render_3d_abi;
import render_3d_environment;
import render_3d_material;
import render_3d_normals;
import render_3d_pbr;
import render_3d_sampling;
import render_3d_toon;

static const float RAY_MINIMUM = 0.001;
static const float RAY_MAXIMUM = 100000.0;

struct OutputSize
{
    uint width;
    uint height;
};

struct ComputePbr
{
    uint4 modes;
    float4 surface;
    float ior;
    float3 padding;
};

struct ComputeMaterial
{
    float4 base_color;
    float4 metallic_roughness_normal_alpha;
    uint4 flags;
    TextureMapping base_color_texture;
    TextureMapping metallic_roughness_texture;
    TextureMapping normal_texture;
    uint4 surface;
    ComputePbr pbr;
};

PbrSettings material_pbr(ComputeMaterial material)
{
    if (material.flags.w == 0)
        return scene.pbr;
    PbrSettings result;
    result.path_tracing = PathTracingMode(material.pbr.modes.x);
    result.light_sampling_quality = LightSamplingQuality(material.pbr.modes.y);
    result.render_quality = RenderQuality(material.pbr.modes.z);
    result.optix_denoising = material.pbr.modes.w;
    result.subsurface = material.pbr.surface.x;
    result.clearcoat = material.pbr.surface.y;
    result.sheen = material.pbr.surface.z;
    result.transmission = material.pbr.surface.w;
    result.ior = material.pbr.ior;
    result._padding = material.pbr.padding;
    return result;
}

ConstantBuffer<SceneUniforms> scene;
ConstantBuffer<OutputSize> output_size;
RaytracingAccelerationStructure scene_acceleration;
StructuredBuffer<float4> positions;
StructuredBuffer<float4> normals;
StructuredBuffer<float4> tangents;
StructuredBuffer<float4> tex_coords_0;
StructuredBuffer<float4> tex_coords_1;
StructuredBuffer<float4> vertex_colors;
StructuredBuffer<ComputeMaterial> materials;
Texture2D<float4> material_texture;
SamplerState material_sampler;
Texture2D<float4> environment_texture;
SamplerState environment_sampler;
StructuredBuffer<MeshInstance> mesh_instances;
RWStructuredBuffer<uint> output_pixels;

float2 vertex_uv(uint vertex, uint set)
{
    return set == 0 ? tex_coords_0[vertex].xy : tex_coords_1[vertex].xy;
}

float4 sample_material_texture(
    TextureMapping mapping,
    uint first,
    float3 barycentric)
{
    if (mapping.settings.x == 0)
        return float4(1.0);
    float2 atlas_uv = material_texture_coordinates(
        mapping,
        vertex_uv(first, mapping.settings.y),
        vertex_uv(first + 1, mapping.settings.y),
        vertex_uv(first + 2, mapping.settings.y),
        barycentric);
    if (mapping.rotation_filter.y == 0.0)
        return material_texture.SampleLevel(material_sampler, atlas_uv, 0.0);
    uint width;
    uint height;
    material_texture.GetDimensions(width, height);
    int2 texture_pixel = int2(clamp(
        floor(atlas_uv * float2(width, height)),
        float2(0.0),
        float2(width - 1, height - 1)));
    return material_texture.Load(int3(texture_pixel, 0));
}

float4 surface_base_color(
    uint first,
    float3 barycentric,
    ComputeMaterial material)
{
    float4 vertex_color = material_vertex_color(
        vertex_colors[first],
        vertex_colors[first + 1],
        vertex_colors[first + 2],
        barycentric);
    return material_base_color(
        material.base_color,
        vertex_color,
        sample_material_texture(material.base_color_texture, first, barycentric));
}

uint2 hit_offsets(uint instance_index, uint geometry_index)
{
    MeshInstance instance = mesh_instances[instance_index];
    return geometry_index == 0
        ? instance.geometry_offsets.xy
        : instance.geometry_offsets.zw;
}

float candidate_surface_alpha(
    uint instance_index,
    uint geometry_index,
    uint primitive_index,
    float2 coordinates,
    out uint material_index,
    out uint alpha_mode,
    out float alpha_cutoff)
{
    uint2 offsets = hit_offsets(instance_index, geometry_index);
    uint first = offsets.x + primitive_index * 3;
    material_index = offsets.y + primitive_index;
    ComputeMaterial material = materials[material_index];
    alpha_mode = material.flags.x != 0 ? material.flags.y : 0;
    alpha_cutoff = material.metallic_roughness_normal_alpha.w;
    if (material.flags.x == 0)
        return scene.base_color.a;
    float3 barycentric = float3(
        1.0 - coordinates.x - coordinates.y,
        coordinates.x,
        coordinates.y);
    return surface_base_color(first, barycentric, material).a;
}

bool candidate_hit_accepted(
    uint instance_index,
    uint geometry_index,
    uint primitive_index,
    float2 coordinates)
{
    uint material_index;
    uint alpha_mode;
    float alpha_cutoff;
    float alpha = candidate_surface_alpha(
        instance_index,
        geometry_index,
        primitive_index,
        coordinates,
        material_index,
        alpha_mode,
        alpha_cutoff);
    return material_hit_accepted(
        material_index,
        coordinates,
        alpha_mode,
        alpha,
        alpha_cutoff);
}

float shadow_visibility(
    float3 world_position,
    float3 world_normal,
    float3 world_direction,
    float maximum)
{
    float3 world_origin = world_position + world_normal * RAY_MINIMUM;
    float3 object_origin = mul(
        scene.model_inverse,
        float4(world_origin, 1.0)).xyz;
    float3 object_target = mul(
        scene.model_inverse,
        float4(world_origin + world_direction * maximum, 1.0)).xyz;
    float3 object_delta = object_target - object_origin;
    RayDesc ray;
    ray.Origin = object_origin;
    ray.Direction = normalize(object_delta);
    ray.TMin = RAY_MINIMUM;
    ray.TMax = max(length(object_delta) - RAY_MINIMUM, RAY_MINIMUM);
    RayQuery<RAY_FLAG_FORCE_NON_OPAQUE | RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES> query;
    query.TraceRayInline(scene_acceleration, RAY_FLAG_NONE, 0xff, ray);
    float visibility = 1.0;
    while (query.Proceed())
    {
        if (query.CandidateType() != CANDIDATE_NON_OPAQUE_TRIANGLE)
            continue;
        uint material_index;
        uint alpha_mode;
        float alpha_cutoff;
        float alpha = candidate_surface_alpha(
                query.CandidateInstanceIndex(),
                query.CandidateGeometryIndex(),
                query.CandidatePrimitiveIndex(),
                query.CandidateTriangleBarycentrics(),
                material_index,
                alpha_mode,
                alpha_cutoff);
        float coverage = material_shadow_coverage(alpha_mode, alpha, alpha_cutoff);
        visibility *= 1.0 - coverage;
        if (coverage >= 1.0)
            query.CommitNonOpaqueTriangleHit();
    }
    return query.CommittedStatus() == COMMITTED_NOTHING ? visibility : 0.0;
}

float3 sample_environment(float3 direction, float lod)
{
    if (scene.environment.source == EnvironmentSource::Image)
        return environment_texture.SampleLevel(
            environment_sampler,
            environment_coordinates(scene.environment_inverse_rotation, direction),
            lod).rgb;
    if (scene.environment.source == EnvironmentSource::Black)
        return scene.environment_color.rgb;
    return float3(0.0);
}

uint pack_color(float4 color)
{
    uint4 value = uint4(round(saturate(color) * 255.0));
    return value.r | (value.g << 8) | (value.b << 16) | (value.a << 24);
}

float4 render_sample(
    float2 sample_position,
    uint2 pixel,
    uint2 dimensions,
    uint seed)
{
    CameraRay camera = camera_ray(
        scene,
        sample_position,
        dimensions,
        true,
        seed);
    if (!camera.valid)
        return float4(0.0);
    RayDesc ray = camera.object_ray;

    RayQuery<RAY_FLAG_FORCE_NON_OPAQUE | RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES> query;
    query.TraceRayInline(scene_acceleration, RAY_FLAG_NONE, 0xff, ray);
    while (query.Proceed())
    {
        if (query.CandidateType() == CANDIDATE_NON_OPAQUE_TRIANGLE
            && candidate_hit_accepted(
                query.CandidateInstanceIndex(),
                query.CandidateGeometryIndex(),
                query.CandidatePrimitiveIndex(),
                query.CandidateTriangleBarycentrics()))
            query.CommitNonOpaqueTriangleHit();
    }

    float4 color = float4(0.0);
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
        uint2 offsets = hit_offsets(instance_index, geometry_index);
        uint first = offsets.x + primitive_index * 3;
        uint material_index = offsets.y + primitive_index;
        float3 triangle_positions[3] = {
            positions[first].xyz,
            positions[first + 1].xyz,
            positions[first + 2].xyz
        };
        float3 triangle_normals[3] = {
            normals[first].xyz,
            normals[first + 1].xyz,
            normals[first + 2].xyz
        };
        ComputeMaterial material = materials[material_index];
        NormalMode normal_mode = material.flags.w != 0
            ? NormalMode(material.surface.x)
            : scene.normal_mode;
        float3 object_normal = surface_normal(
            normal_mode,
            triangle_positions,
            triangle_normals,
            barycentric);
        float3 normal = normalize(mul(
            mul(scene.normal, instance.normal),
            float4(object_normal, 0.0)).xyz);
        float4 sampled_normal = sample_material_texture(
            material.normal_texture,
            first,
            barycentric);
        if (material.normal_texture.settings.x != 0)
        {
            float4 object_tangent = tangents[first] * barycentric.x
                + tangents[first + 1] * barycentric.y
                + tangents[first + 2] * barycentric.z;
            float3 tangent = normalize(mul(
                mul(scene.model, instance.model),
                float4(object_tangent.xyz, 0.0)).xyz);
            float handedness = object_tangent.w < 0.0 ? -1.0 : 1.0;
            normal = apply_normal_map(
                normal,
                tangent,
                handedness,
                sampled_normal.xyz,
                material.metallic_roughness_normal_alpha.z);
        }
        float3 object_position = triangle_positions[0] * barycentric.x
            + triangle_positions[1] * barycentric.y
            + triangle_positions[2] * barycentric.z;
        float3 scene_position = mul(instance.model, float4(object_position, 1.0)).xyz;
        float3 world_position = mul(scene.model, float4(scene_position, 1.0)).xyz;
        float3 view = -camera.world_direction;
        if (dot(normal, view) < 0.0)
            normal = -normal;
        float4 mesh_base_color = surface_base_color(first, barycentric, material);
        float3 base_color = material.flags.x != 0
            ? mesh_base_color.rgb
            : scene.base_color.rgb;
        float4 metallic_roughness = sample_material_texture(
            material.metallic_roughness_texture,
            first,
            barycentric);
        float metallic = material_metallic(
            material.flags.x != 0,
            material.metallic_roughness_normal_alpha.x,
            metallic_roughness,
            scene.material.x);
        float roughness = material_roughness(
            material.flags.x != 0,
            material.metallic_roughness_normal_alpha.y,
            metallic_roughness,
            scene.material.y);
        PbrSettings surface_pbr = material_pbr(material);
        if (scene.toon.shading_model == ShadingModel::Depth)
        {
            float4 clip_position = mul(
                scene.view_projection,
                float4(world_position, 1.0));
            float depth = saturate(
                clip_position.z / max(abs(clip_position.w), RAY_MINIMUM));
            color = float4(
                float3(depth),
                material.flags.x != 0 ? mesh_base_color.a : scene.base_color.a);
            return color;
        }
        float3 shaded = base_color * 0.02;
        bool toon = scene.toon.shading_model == ShadingModel::Toon;
        uint light_samples = toon ? 1u : uint(scene.pbr.light_sampling_quality);
        for (uint light_index = 0; light_index < scene.sun_light_count; ++light_index)
        {
            SunLight light = scene.sun_lights[light_index];
            float3 light_color = float3(0.0);
            for (uint light_sample = 0; light_sample < light_samples; ++light_sample)
            {
                float3 world_light = sample_cone(
                    normalize(-light.direction_angular_radius.xyz),
                    light.direction_angular_radius.w,
                    seed);
                float visibility = shadow_visibility(
                    world_position,
                    normal,
                    world_light,
                    RAY_MAXIMUM);
                light_color += pbr_light(
                    normal,
                    view,
                    world_light,
                    light.color_intensity.rgb * light.color_intensity.w * visibility,
                    base_color,
                    metallic,
                    roughness,
                    float3(1.0),
                    surface_pbr);
            }
            shaded += light_color / float(light_samples);
        }
        for (uint light_index = 0; light_index < scene.point_light_count; ++light_index)
        {
            PointLight light = scene.point_lights[light_index];
            float3 light_color = float3(0.0);
            for (uint light_sample = 0; light_sample < light_samples; ++light_sample)
            {
                float3 sampled_position = light.position_radius.xyz
                    + sample_unit_sphere(seed) * light.position_radius.w;
                float3 delta = sampled_position - world_position;
                float distance_squared = dot(delta, delta);
                float distance = sqrt(distance_squared);
                if (distance >= light.range_padding.x)
                    continue;
                float visibility = shadow_visibility(
                    world_position,
                    normal,
                    delta / max(distance, RAY_MINIMUM),
                    distance);
                float falloff = pow(1.0 - distance / light.range_padding.x, 2.0)
                    / max(distance_squared, 0.01);
                light_color += pbr_light(
                    normal,
                    view,
                    delta / max(distance, RAY_MINIMUM),
                    light.color_intensity.rgb * light.color_intensity.w * falloff * visibility,
                    base_color,
                    metallic,
                    roughness,
                    float3(1.0),
                    surface_pbr);
            }
            shaded += light_color / float(light_samples);
        }
        uint environment_width;
        uint environment_height;
        uint environment_levels;
        environment_texture.GetDimensions(
            0,
            environment_width,
            environment_height,
            environment_levels);
        float max_environment_lod = float(max(environment_levels, 1u) - 1u);
        float3 environment_diffuse = sample_environment(normal, max_environment_lod)
            * scene.environment.intensity;
        float3 reflection = reflect(-view, normal);
        float3 environment_reflection = sample_environment(
            reflection,
            roughness * max_environment_lod)
            * scene.environment.intensity;
        float3 fresnel = pbr_fresnel(
            base_color,
            metallic,
            saturate(dot(normal, view)),
            surface_pbr.ior);
        shaded += environment_diffuse * base_color * (1.0 - metallic)
            + environment_reflection * fresnel;
        if (scene.toon.shading_model == ShadingModel::Toon)
            shaded = toon_palette_quantize(shaded, scene.toon.color_levels);
        shaded *= scene.material.z;
        shaded = float3(
            linear_to_srgb(aces_tone_map(shaded.r)),
            linear_to_srgb(aces_tone_map(shaded.g)),
            linear_to_srgb(aces_tone_map(shaded.b)));
        color = float4(
            shaded,
            material.flags.x != 0 ? mesh_base_color.a : scene.base_color.a);
    }
    return color;
}

[shader("compute")]
[numthreads(8, 8, 1)]
void obj_compute(uint3 thread_id : SV_DispatchThreadID)
{
    uint2 pixel = thread_id.xy;
    uint2 dimensions = uint2(output_size.width, output_size.height);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y)
        return;

    AntiAliasing anti_aliasing = scene.toon.anti_aliasing;
    uint anti_aliasing_samples = anti_aliasing_sample_count(anti_aliasing);
    uint depth_of_field_samples = scene.camera_lens.y > 0.0
        ? (scene.pbr.render_quality == RenderQuality::Interactive ? 2u : 8u)
        : 1u;
    uint sample_count = max(anti_aliasing_samples, depth_of_field_samples);
    float4 color = float4(0.0);
    for (uint sample_index = 0; sample_index < sample_count; ++sample_index)
    {
        float2 offset = sample_count == anti_aliasing_samples
            ? anti_aliasing_offset(sample_index, anti_aliasing, pixel)
            : stochastic_pixel_offset(sample_index, sample_count, pixel);
        color += render_sample(
            float2(pixel) + 0.5 + offset,
            pixel,
            dimensions,
            random_seed(pixel, sample_index));
    }
    output_pixels[pixel.y * dimensions.x + pixel.x] = pack_color(
        color / float(sample_count));
}
