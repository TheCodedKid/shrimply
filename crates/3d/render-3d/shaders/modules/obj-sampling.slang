static const uint INTERACTIVE_MINIMUM_SAMPLES = 2u;
static const uint INTERACTIVE_SAMPLE_DIVISOR = 8u;

void adaptive_path_budget(
    out uint minimum_samples,
    out uint maximum_samples,
    out float relative_error)
{
    uint requested = max(uint(scene.pbr.path_tracing), 1u);
    if (scene.pbr.render_quality == RenderQuality::Interactive)
    {
        minimum_samples = INTERACTIVE_MINIMUM_SAMPLES;
        maximum_samples = max(requested / INTERACTIVE_SAMPLE_DIVISOR, minimum_samples);
        relative_error = INTERACTIVE_RELATIVE_ERROR;
        return;
    }
    minimum_samples = requested;
    maximum_samples = requested * 2u;
    relative_error = FINAL_RELATIVE_ERROR;
}

bool adaptive_converged(float4 mean, float4 squared_error, uint sample_count, float relative_error)
{
    if (sample_count < MINIMUM_VARIANCE_SAMPLES)
        return false;
    float4 variance = squared_error / float(sample_count - 1u);
    float color_standard_error = sqrt(
        max(max(variance.r, max(variance.g, variance.b)), 0.0) / float(sample_count));
    float alpha_standard_error = sqrt(max(variance.a, 0.0) / float(sample_count));
    float color_scale = max(
        max(abs(mean.r), max(abs(mean.g), abs(mean.b))),
        MINIMUM_ADAPTIVE_MEAN);
    return color_standard_error <= relative_error * color_scale
        && alpha_standard_error <= relative_error;
}

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
