struct GroundShadowTracer : IGroundShadowTracer
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

float ground_shadow_alpha(float3 world_position, float3 normal, inout uint state)
{
    GroundShadowTracer tracer;
    return integrate_ground_shadow(scene, world_position, normal, state, tracer);
}

struct GroundReflectionTracer : IGroundReflectionTracer
{
    float4 radiance(
        float3 world_origin,
        float3 world_direction,
        uint path_bounce,
        uint random_seed)
    {
        RayDesc ray = {
            mul(scene.model_inverse, float4(world_origin, 1.0)).xyz,
            RAY_MINIMUM,
            normalize(mul(
                scene.model_inverse,
                float4(world_direction, 0.0)).xyz),
            RAY_MAXIMUM
        };
        RayPayload reflected = {
            float4(0.0),
            float4(0.0),
            float3(0.0),
            RAY_MAXIMUM,
            RAY_MAXIMUM,
            0.0,
            0,
            0,
            path_bounce,
            random_seed,
            0,
            0xffffffffu
        };
        TraceRay(
            scene_acceleration,
            RAY_FLAG_SKIP_PROCEDURAL_PRIMITIVES,
            0xff,
            0,
            0,
            0,
            ray,
            reflected);
        return reflected.hit != 0 ? reflected.color : float4(0.0);
    }
};

void composite_virtual_surfaces(
    inout RayPayload payload,
    float3 world_origin,
    float3 world_direction,
    uint random_seed)
{
    float visible_distance = payload.hit != 0 ? payload.ray_distance : RAY_MAXIMUM;
    int ground_index = -1;
    float ground_distance = visible_distance + RAY_MINIMUM;
    float3 world_position = float3(0.0);
    float3 ground_normal = float3(0.0);
    if (payload.path_bounce != 0)
    {
        float4 virtual_color = float4(0.0);
        float virtual_distance = RAY_MAXIMUM;
        int virtual_ground = -1;
        if (!sample_composed_planes(
            world_origin,
            world_direction,
            0.0,
            virtual_color,
            virtual_distance,
            virtual_ground)
            || virtual_distance >= visible_distance)
            return;

        float3 virtual_position = world_origin + world_direction * virtual_distance;
        float3 camera_forward = normalize(mul(
            scene.camera_world,
            float4(0.0, 0.0, -1.0, 0.0)).xyz);
        payload.hit = 1;
        payload.material_index = 0xffffffffu;
        payload.normal = virtual_ground >= 0
            ? normalize(scene.grounds[virtual_ground].normal_composite.xyz)
            : -camera_forward;
        payload.depth = length(scene.camera_position.xyz - virtual_position);
        payload.ray_distance = virtual_distance;
        if (payload.transmitted != 0)
        {
            payload.color = float4(0.0);
            payload.background = virtual_color;
        }
        else
        {
            payload.color = virtual_color;
            payload.background = float4(0.0);
        }
        if (virtual_ground < 0)
            return;
        ground_index = virtual_ground;
        ground_distance = virtual_distance;
        world_position = virtual_position;
        ground_normal = payload.normal;
    }
    else
    {
        for (uint index = 0; index < scene.ground_count; ++index)
        {
            float candidate_distance;
            float3 candidate_position;
            float3 candidate_normal;
            if (intersect_ground(
                scene.grounds[index],
                world_origin,
                world_direction,
                candidate_distance,
                candidate_position,
                candidate_normal)
                && candidate_distance < ground_distance)
            {
                ground_index = int(index);
                ground_distance = candidate_distance;
                world_position = candidate_position;
                ground_normal = candidate_normal;
            }
        }
    }
    if (ground_index < 0 || ground_distance >= visible_distance + RAY_MINIMUM)
        return;
    Ground ground = scene.grounds[ground_index];

    if (scene.ground_shading_model == ShadingModel::Depth)
    {
        float depth = depth_buffer_value(world_position);
        payload.color = float4(float3(depth), ground.position_opacity.w);
        payload.background = float4(0.0);
        payload.normal = ground_normal;
        payload.depth = length(scene.camera_position.xyz - world_position);
        payload.ray_distance = ground_distance;
        payload.hit = 1;
        payload.material_index = 0xffffffffu;
        return;
    }

    uint state = random_seed;
    float shadow_occlusion = ground_shadow_alpha(world_position, ground_normal, state);
    float receiver_strength = ground.settings.y * ground.position_opacity.w;
    float shadow_alpha = shadow_occlusion * receiver_strength;
    bool toon = scene.ground_shading_model == ShadingModel::Toon;
    float4 toon_shadow = float4(0.0);
    if (toon)
        toon_shadow = toon_ground_shadow(
            scene,
            shadow_occlusion,
            receiver_strength,
            float2(DispatchRaysIndex().xy) + 0.5);
    GroundReflectionTracer reflection_tracer;
    float4 reflection = ground_reflection(
        scene,
        ground,
        world_position,
        ground_normal,
        world_direction,
        payload.path_bounce,
        state,
        reflection_tracer);
    if (payload.transmitted != 0)
    {
        composite_over(payload.background, reflection);
        if (toon)
        {
            payload.background.rgb = lerp(
                payload.background.rgb,
                toon_shadow.rgb,
                toon_shadow.a);
            payload.color.rgb = lerp(
                payload.color.rgb,
                toon_shadow.rgb,
                toon_shadow.a);
        }
        else
        {
            payload.background.rgb *= 1.0 - shadow_alpha;
            payload.color.rgb *= 1.0 - shadow_alpha;
        }
        return;
    }
    composite_over(payload.color, reflection);
    if (toon)
    {
        composite_over(payload.color, toon_shadow);
        return;
    }
    composite_ground_shadow(payload.color, shadow_alpha);
}
