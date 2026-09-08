RayPayload trace_surface(float2 sample_position, uint2 dimensions, bool probe, uint random_seed)
{
    RayPayload payload = {
        float4(0.0),
        float4(0.0),
        float3(0.0),
        RAY_MAXIMUM,
        RAY_MAXIMUM,
        0.0,
        0,
        probe ? 1u : 0u,
        0,
        random_seed,
        0,
        0xffffffffu
    };
    CameraRay camera = camera_ray(
        scene,
        sample_position,
        dimensions,
        !probe,
        random_seed);
    if (!camera.valid)
        return payload;
    TraceRay(
        scene_acceleration,
        RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES,
        0xff,
        0,
        0,
        0,
        camera.object_ray,
        payload);
    if (!probe)
        composite_virtual_surfaces(
            payload,
            camera.ground_origin,
            camera.world_direction,
            random_seed);
    return payload;
}
