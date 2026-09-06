float3 shadow_visibility(
    float3 world_position,
    float3 world_normal,
    float3 direction,
    float distance)
{
    float3 origin = world_position + world_normal * RAY_MINIMUM;
    float3 object_origin = mul(scene.model_inverse, float4(origin, 1.0)).xyz;
    float3 object_target = mul(
        scene.model_inverse,
        float4(origin + direction * distance, 1.0)).xyz;
    float3 object_delta = object_target - object_origin;

    RayDesc ray;
    ray.Origin = object_origin;
    ray.Direction = normalize(object_delta);
    ray.TMin = RAY_MINIMUM;
    ray.TMax = max(length(object_delta) - RAY_MINIMUM, RAY_MINIMUM);
    ShadowPayload shadow = {
        float3(1.0),
        0
    };
    TraceRay(
        scene_acceleration,
        RAY_FLAG_ACCEPT_FIRST_HIT_AND_END_SEARCH
            | RAY_FLAG_SKIP_CLOSEST_HIT_SHADER
            | RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES,
        0xff,
        1,
        0,
        1,
        ray,
        shadow);
    return shadow.visible != 0 ? shadow.transmittance : float3(0.0);
}

struct ToonShadowTracer : IToonShadowTracer
{
    float visibility(
        float3 world_position,
        float3 normal,
        float3 direction,
        float maximum)
    {
        return saturate(luminance(shadow_visibility(
            world_position,
            normal,
            direction,
            maximum)));
    }
};

float3 sample_pbr_sun(
    SunLight source,
    float3 world_position,
    float3 normal,
    float3 view,
    float3 base_color,
    float metallic,
    float roughness,
    PbrSettings settings,
    uint sample_count,
    inout uint state)
{
    float3 color = float3(0.0);
    float3 center_direction = normalize(-source.direction_angular_radius.xyz);
    float3 radiance = source.color_intensity.rgb * source.color_intensity.w;
    for (uint sample_index = 0; sample_index < sample_count; ++sample_index)
    {
        float3 light = sample_cone(
            center_direction,
            source.direction_angular_radius.w,
            state);
        float3 visibility = shadow_visibility(
            world_position,
            normal,
            light,
            RAY_MAXIMUM);
        color += pbr_light(
            normal,
            view,
            light,
            radiance,
            base_color,
            metallic,
            roughness,
            visibility,
            settings);
    }
    return color / float(sample_count);
}

float3 sample_pbr_point(
    PointLight source,
    float3 world_position,
    float3 normal,
    float3 view,
    float3 base_color,
    float metallic,
    float roughness,
    PbrSettings settings,
    uint sample_count,
    inout uint state)
{
    float3 color = float3(0.0);
    for (uint sample_index = 0; sample_index < sample_count; ++sample_index)
    {
        float3 sampled_position = source.position_radius.xyz
            + sample_unit_sphere(state) * source.position_radius.w;
        float3 delta = sampled_position - world_position;
        float distance_squared = dot(delta, delta);
        float distance = sqrt(distance_squared);
        if (distance >= source.range_padding.x)
            continue;
        float3 light = delta / max(distance, RAY_MINIMUM);
        float3 visibility = shadow_visibility(
            world_position,
            normal,
            light,
            distance);
        float falloff = pow(1.0 - distance / source.range_padding.x, 2.0)
            / max(distance_squared, 0.01);
        float3 radiance = source.color_intensity.rgb
            * source.color_intensity.w * falloff;
        color += pbr_light(
            normal,
            view,
            light,
            radiance,
            base_color,
            metallic,
            roughness,
            visibility,
            settings);
    }
    return color / float(sample_count);
}

struct ObjPbrEnvironmentTracer : IPbrEnvironmentTracer
{
    float3 radiance(float3 world_origin, float3 direction, out bool composed_hit)
    {
        return sample_environment(world_origin, direction, 0.0, composed_hit);
    }

    float3 visibility(float3 world_position, float3 normal, float3 direction)
    {
        return shadow_visibility(
            world_position,
            normal,
            direction,
            RAY_MAXIMUM);
    }
};

float3 sample_pbr_environment(
    float3 world_position,
    float3 normal,
    float3 view,
    float3 base_color,
    float metallic,
    float roughness,
    PbrSettings settings,
    inout uint state)
{
    ObjPbrEnvironmentTracer tracer;
    return integrate_pbr_environment(
        scene,
        world_position,
        normal,
        view,
        base_color,
        metallic,
        roughness,
        settings,
        state,
        tracer);
}
