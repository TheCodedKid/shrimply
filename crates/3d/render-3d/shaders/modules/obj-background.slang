struct ObjCompositeSampler : ICompositeSampler
{
    uint2 composite_dimensions()
    {
        uint width;
        uint height;
        transmission_background_texture.GetDimensions(width, height);
        return uint2(width, height);
    }

    float4 composite_sample(float2 uv, float lod)
    {
        return transmission_background_texture.SampleLevel(
            environment_sampler,
            uv,
            lod);
    }

    float3 environment_sample(float3 direction, float lod)
    {
        return environment_texture.SampleLevel(
            environment_sampler,
            environment_coordinates(scene.environment_inverse_rotation, direction),
            lod).rgb;
    }
};

bool environment_enabled()
{
    return shared_environment_enabled(scene);
}

bool sample_composed_planes(
    float3 world_origin,
    float3 world_direction,
    float lod,
    out float4 color,
    out float distance,
    out int ground_index)
{
    ObjCompositeSampler sampler;
    return shared_sample_composed_planes(
        scene,
        sampler,
        world_origin,
        world_direction,
        lod,
        color,
        distance,
        ground_index);
}

float3 sample_environment(
    float3 world_origin,
    float3 direction,
    float lod,
    out bool composed_hit)
{
    ObjCompositeSampler sampler;
    return shared_sample_environment(
        scene,
        sampler,
        world_origin,
        direction,
        lod,
        composed_hit);
}

bool sample_transmission_background(out float4 background)
{
    float3 world_origin = mul(scene.model, float4(WorldRayOrigin(), 1.0)).xyz;
    float3 world_direction = normalize(mul(
        scene.model,
        float4(WorldRayDirection(), 0.0)).xyz);
    ObjCompositeSampler sampler;
    return shared_sample_transmission_background(
        scene,
        sampler,
        world_origin,
        world_direction,
        background);
}
