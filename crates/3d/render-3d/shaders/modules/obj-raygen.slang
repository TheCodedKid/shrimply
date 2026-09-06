[shader("raygeneration")]
void raygen_main()
{
    uint2 pixel = DispatchRaysIndex().xy;
    uint2 dimensions = DispatchRaysDimensions().xy;
    AntiAliasing anti_aliasing = scene.toon.anti_aliasing;
    uint anti_aliasing_samples = anti_aliasing_sample_count(anti_aliasing);
    uint minimum_samples = 1u;
    uint maximum_samples = 1u;
    float relative_error = 0.0;
    bool adaptive = scene.pbr.path_tracing != PathTracingMode::Off;
    if (adaptive)
        adaptive_path_budget(
            scene.pbr,
            minimum_samples,
            maximum_samples,
            relative_error);
    uint transmission_samples = scene.pbr.transmission > 0.0
        ? ground_light_sample_count(scene.pbr)
        : 1u;
    uint depth_of_field_samples = scene.camera_lens.y > 0.0
        ? (scene.pbr.render_quality == RenderQuality::Interactive ? 2u : 8u)
        : 1u;
    minimum_samples = max(
        minimum_samples,
        max(anti_aliasing_samples, max(transmission_samples, depth_of_field_samples)));
    maximum_samples = max(maximum_samples, minimum_samples);
    float4 mean = float4(0.0);
    float4 mean_background = float4(0.0);
    float4 squared_error = float4(0.0);
    uint sample_count = 0u;
    for (uint sample_index = 0; sample_index < maximum_samples; ++sample_index)
    {
        float2 offset = maximum_samples == anti_aliasing_samples
            ? anti_aliasing_offset(sample_index, anti_aliasing, pixel)
            : stochastic_pixel_offset(sample_index, maximum_samples, pixel);
        RayPayload sample = trace_surface(
            float2(pixel) + 0.5 + offset,
            dimensions,
            false,
            random_seed(pixel, sample_index));
        float4 sample_color = sample.color;
        sample_count += 1u;
        float4 delta = sample_color - mean;
        mean += delta / float(sample_count);
        mean_background += (sample.background - mean_background) / float(sample_count);
        squared_error += delta * (sample_color - mean);
        if (adaptive
            && sample_count >= minimum_samples
            && adaptive_converged(mean, squared_error, sample_count, relative_error))
            break;
    }
    float4 color = mean;

    OutlineMode outline_mode = scene.toon.outline_mode;
    bool outlines = outline_mode != OutlineMode::Off;
    OutlineMethod outline_method = scene.toon.outline_method;
    if (outlines && outline_method == OutlineMethod::RegionBoundary)
    {
        RayPayload guide = trace_surface(float2(pixel) + 0.5, dimensions, true, 0);
        outline_guide[pixel] = guide.hit != 0
            ? float4(guide.normal, guide.depth)
            : float4(0.0, 0.0, 0.0, RAY_MAXIMUM);
    }
    if (outlines && outline_method != OutlineMethod::RegionBoundary)
    {
        float2 center_position = float2(pixel) + 0.5;
        float2 outline_offset;
        float outline_width_offset;
        outline_variation(
            center_position,
            scene.toon,
            outline_offset,
            outline_width_offset);
        float2 outline_position = center_position + outline_offset;
        float outline_width = varied_outline_width(
            outline_width_offset,
            scene.toon);
        RayPayload center = trace_surface(outline_position, dimensions, true, 0);
        OutlineSurface outline_center = outline_surface(center);
        ObjOutlineTracer outline_tracer;
        float coverage = 0.0;
        uint probe_count = uint(scene.toon.outline_quality);
        switch (outline_method)
        {
        case OutlineMethod::RayTraced:
            coverage = radial_outline(
                outline_position,
                dimensions,
                outline_center,
                outline_mode,
                outline_width,
                probe_count,
                scene.toon,
                outline_tracer);
            break;
        case OutlineMethod::Fresnel:
            break;
        case OutlineMethod::Hybrid:
            coverage = radial_outline(
                outline_position,
                dimensions,
                outline_center,
                outline_mode,
                outline_width,
                probe_count,
                scene.toon,
                outline_tracer);
            break;
        case OutlineMethod::Sobel:
            coverage = sobel_outline(
                outline_position,
                dimensions,
                outline_center,
                outline_mode,
                outline_width,
                scene.toon,
                outline_tracer);
            break;
        case OutlineMethod::RobertsCross:
            coverage = roberts_cross_outline(
                outline_position,
                dimensions,
                outline_center,
                outline_mode,
                outline_width,
                scene.toon,
                outline_tracer);
            break;
        case OutlineMethod::DifferenceOfGaussians:
            coverage = difference_of_gaussians_outline(
                outline_position,
                dimensions,
                outline_center,
                outline_mode,
                outline_clamp_width(
                    scene.toon.dog_inner_radius + outline_width_offset),
                probe_count,
                scene.toon,
                outline_tracer);
            break;
        default:
            break;
        }
        if ((outline_method == OutlineMethod::Fresnel
                || outline_method == OutlineMethod::Hybrid)
            && center.hit != 0)
        {
            float fresnel_coverage = toon_outline_fresnel(
                center.facing,
                outline_width);
            coverage = max(coverage, fresnel_coverage);
        }
        coverage = outline_binary_coverage(coverage, scene.toon);
        color = composite_outline(color, coverage, scene.toon);
    }
    if (scene.toon.shading_model == ShadingModel::Toon)
        color.rgb = toon_palette_quantize(color.rgb, scene.toon.color_levels);
    if (scene.pbr.denoising != 0)
    {
        RayPayload guide = trace_surface(float2(pixel) + 0.5, dimensions, true, 0);
        denoiser_beauty[pixel] = color;
        denoiser_background[pixel] = mean_background;
        denoiser_albedo[pixel] = guide.hit != 0
            ? float4(guide.color.rgb, 1.0)
            : float4(0.0);
        denoiser_normal[pixel] = guide.hit != 0
            ? float4(guide.normal, 1.0)
            : float4(0.0);
    }
    else
        output_image[pixel] = color;
}

[shader("raygeneration")]
void outline_distance_main()
{
    uint2 pixel = DispatchRaysIndex().xy;
    uint2 dimensions = DispatchRaysDimensions().xy;
    if (scene.toon.outline_mode == OutlineMode::Off
        || scene.toon.outline_method != OutlineMethod::RegionBoundary)
        return;
    ObjRegionOutlineStorage storage;
    outline_distance[pixel] = region_horizontal_outline_distances(
        float2(pixel) + 0.5,
        dimensions,
        scene.toon.outline_mode,
        scene.toon,
        storage);
}

[shader("raygeneration")]
void outline_main()
{
    uint2 pixel = DispatchRaysIndex().xy;
    uint2 dimensions = DispatchRaysDimensions().xy;
    if (scene.toon.outline_mode == OutlineMode::Off
        || scene.toon.outline_method != OutlineMethod::RegionBoundary)
        return;

    float2 center_position = float2(pixel) + 0.5;
    float2 outline_offset;
    float outline_width_offset;
    outline_variation(
        center_position,
        scene.toon,
        outline_offset,
        outline_width_offset);
    float2 outline_position = center_position + outline_offset;
    float outline_width = varied_outline_width(
        outline_width_offset,
        scene.toon);
    ObjRegionOutlineStorage storage;
    float coverage = region_boundary_outline(
        outline_position,
        dimensions,
        scene.toon.outline_mode,
        outline_width,
        storage);
    if (coverage == 0.0)
        return;

    bool denoising = scene.pbr.denoising != 0;
    float4 color = denoising
        ? denoiser_beauty[pixel]
        : output_image[pixel];
    color = composite_outline(color, 1.0, scene.toon);
    if (scene.toon.shading_model == ShadingModel::Toon)
        color.rgb = toon_palette_quantize(color.rgb, scene.toon.color_levels);
    if (denoising)
        denoiser_beauty[pixel] = color;
    else
        output_image[pixel] = color;
}
