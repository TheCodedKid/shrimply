module render_3d_outline;

import render_3d_abi;
import render_3d_ray;

static const uint OUTLINE_NOISE_X_CHANNEL = 0x68bc21ebu;
static const uint OUTLINE_NOISE_Y_CHANNEL = 0x02e5be93u;
static const uint OUTLINE_NOISE_WIDTH_CHANNEL = 0x967a889bu;
static const uint OUTLINE_NOISE_HASH_X = 0x9e3779b9u;
static const uint OUTLINE_NOISE_HASH_Y = 0x85ebca6bu;
static const uint OUTLINE_NOISE_HASH_Z = 0xc2b2ae35u;
static const uint OUTLINE_NOISE_HASH_A = 0x7feb352du;
static const uint OUTLINE_NOISE_HASH_B = 0x846ca68bu;
static const float OUTLINE_UINT_TO_UNIT_FLOAT = 1.0 / 4294967296.0;
static const float OUTLINE_DETECTION_THRESHOLD = 0.1;
static const float OUTLINE_PI = 3.14159265358979323846;
static const uint OUTLINE_DOG_RING_COUNT = 3u;

public static const float OUTLINE_MINIMUM_WIDTH = 0.25;
public static const int OUTLINE_MAXIMUM_WIDTH = 16;

float outline_noise_value(int3 cell, uint channel, ToonSettings settings)
{
    uint state = asuint(cell.x) * OUTLINE_NOISE_HASH_X
        ^ asuint(cell.y) * OUTLINE_NOISE_HASH_Y
        ^ asuint(cell.z) * OUTLINE_NOISE_HASH_Z
        ^ settings.outline_noise_seed
        ^ channel;
    state ^= state >> 16;
    state *= OUTLINE_NOISE_HASH_A;
    state ^= state >> 15;
    state *= OUTLINE_NOISE_HASH_B;
    state ^= state >> 16;
    return float(state) * OUTLINE_UINT_TO_UNIT_FLOAT;
}

float outline_noise(
    float2 position,
    float frequency,
    uint channel,
    ToonSettings settings)
{
    float3 point = float3(position * frequency, settings.outline_noise_evolution);
    int3 cell = int3(floor(point));
    float3 blend = frac(point);
    blend = blend * blend * (3.0 - 2.0 * blend);
    float lower = lerp(
        lerp(
            outline_noise_value(cell, channel, settings),
            outline_noise_value(cell + int3(1, 0, 0), channel, settings),
            blend.x),
        lerp(
            outline_noise_value(cell + int3(0, 1, 0), channel, settings),
            outline_noise_value(cell + int3(1, 1, 0), channel, settings),
            blend.x),
        blend.y);
    float upper = lerp(
        lerp(
            outline_noise_value(cell + int3(0, 0, 1), channel, settings),
            outline_noise_value(cell + int3(1, 0, 1), channel, settings),
            blend.x),
        lerp(
            outline_noise_value(cell + int3(0, 1, 1), channel, settings),
            outline_noise_value(cell + int3(1, 1, 1), channel, settings),
            blend.x),
        blend.y);
    return lerp(lower, upper, blend.z) * 2.0 - 1.0;
}

public void outline_variation(
    float2 position,
    ToonSettings settings,
    out float2 offset,
    out float width)
{
    offset = float2(0.0);
    width = 0.0;
    if (settings.outline_offset_variation > 0.0)
    {
        offset = settings.outline_offset_variation * float2(
            outline_noise(
                position,
                settings.outline_frequency.x,
                OUTLINE_NOISE_X_CHANNEL,
                settings),
            outline_noise(
                position,
                settings.outline_frequency.x,
                OUTLINE_NOISE_Y_CHANNEL,
                settings));
    }
    if (settings.outline_width_variation > 0.0)
    {
        width = settings.outline_width_variation
            * outline_noise(
                position,
                settings.outline_frequency.y,
                OUTLINE_NOISE_WIDTH_CHANNEL,
                settings);
    }
}

public float outline_clamp_width(float width)
{
    return clamp(
        width,
        OUTLINE_MINIMUM_WIDTH,
        float(OUTLINE_MAXIMUM_WIDTH));
}

public float varied_outline_width(float variation, ToonSettings settings)
{
    return outline_clamp_width(settings.outline_width + variation);
}

public float outline_binary_coverage(float coverage, ToonSettings settings)
{
    return coverage >= OUTLINE_DETECTION_THRESHOLD / settings.outline_aggressiveness
        ? 1.0
        : 0.0;
}

public float4 composite_outline(
    float4 color,
    float coverage,
    ToonSettings settings)
{
    float alpha = settings.outline_color.a * settings.outline_opacity * coverage;
    color.rgb = lerp(color.rgb, settings.outline_color.rgb, alpha);
    color.a = max(color.a, alpha);
    return color;
}

public struct OutlineSurface
{
    public uint hit;
    public float3 normal;
    public float depth;
    public float facing;
};

public interface IOutlineTracer
{
    OutlineSurface trace(float2 position, uint2 dimensions, uint sample_index);
}

public interface IRegionOutlineStorage
{
    float4 guide(float2 position, uint2 dimensions);
    float2 distance(float2 position, uint2 dimensions);
}

static const int OUTLINE_CONTINUATION_STEPS = 3;
static const float OUTLINE_EDGE_PEAK_RATIO = 2.0;

bool outline_guide_hit(float4 guide)
{
    return guide.w < RAY_MAXIMUM;
}

bool directional_outline_crease<T : IRegionOutlineStorage>(
    float2 position,
    float2 direction,
    uint2 dimensions,
    float depth_threshold,
    float normal_threshold,
    T storage)
{
    float4 center = storage.guide(position, dimensions);
    float4 neighbor = storage.guide(position + direction, dimensions);
    if (!outline_guide_hit(center) || !outline_guide_hit(neighbor))
        return false;

    float depth_change = abs(center.w - neighbor.w)
        / max(min(center.w, neighbor.w), RAY_MINIMUM);
    float normal_change = 1.0 - dot(center.xyz, neighbor.xyz);
    if (depth_change <= depth_threshold && normal_change <= normal_threshold)
        return false;

    float continuation_depth = 0.0;
    float continuation_normal = 0.0;
    float4 previous = neighbor;
    for (int step = 2; step <= OUTLINE_CONTINUATION_STEPS + 1; ++step)
    {
        float4 following = storage.guide(
            position + direction * step,
            dimensions);
        if (!outline_guide_hit(following))
            break;
        continuation_depth = max(
            continuation_depth,
            abs(previous.w - following.w)
                / max(min(previous.w, following.w), RAY_MINIMUM));
        continuation_normal = max(
            continuation_normal,
            1.0 - dot(previous.xyz, following.xyz));
        previous = following;
    }

    return (depth_change > depth_threshold
            && depth_change > continuation_depth * OUTLINE_EDGE_PEAK_RATIO)
        || (normal_change > normal_threshold
            && normal_change > continuation_normal * OUTLINE_EDGE_PEAK_RATIO);
}

bool outline_crease_seed<T : IRegionOutlineStorage>(
    float2 position,
    uint2 dimensions,
    ToonSettings settings,
    T storage)
{
    float depth_threshold = settings.outline_depth_threshold
        / settings.outline_aggressiveness;
    float normal_threshold = (1.0 - settings.outline_normal_cosine)
        / settings.outline_aggressiveness;
    for (int axis = 0; axis < 2; ++axis)
    {
        for (int direction = -1; direction <= 1; direction += 2)
        {
            float2 offset = axis == 0
                ? float2(direction, 0.0)
                : float2(0.0, direction);
            if (directional_outline_crease(
                    position,
                    offset,
                    dimensions,
                    depth_threshold,
                    normal_threshold,
                    storage))
                return true;
        }
    }
    return false;
}

public float2 region_horizontal_outline_distances<T : IRegionOutlineStorage>(
    float2 position,
    uint2 dimensions,
    OutlineMode mode,
    ToonSettings settings,
    T storage)
{
    float region_distance = RAY_MAXIMUM;
    float crease_distance = RAY_MAXIMUM;
    int maximum_width = min(
        OUTLINE_MAXIMUM_WIDTH,
        int(ceil(settings.outline_width + settings.outline_width_variation)));
    for (int x = -maximum_width; x <= maximum_width; ++x)
    {
        float2 candidate_position = position + float2(x, 0.0);
        float4 candidate = storage.guide(candidate_position, dimensions);
        if (!outline_guide_hit(candidate))
            continue;
        float distance = abs(float(x));
        region_distance = min(region_distance, distance);
        if (mode == OutlineMode::SilhouetteAndCreases
            && outline_crease_seed(candidate_position, dimensions, settings, storage))
            crease_distance = min(crease_distance, distance);
    }
    return float2(region_distance, crease_distance);
}

public float region_boundary_outline<T : IRegionOutlineStorage>(
    float2 center_position,
    uint2 dimensions,
    OutlineMode mode,
    float width,
    T storage)
{
    bool center_hit = outline_guide_hit(storage.guide(center_position, dimensions));
    float region_distance = RAY_MAXIMUM;
    float crease_distance = RAY_MAXIMUM;
    float crease_radius = width * 0.5;
    int vertical_extent = min(OUTLINE_MAXIMUM_WIDTH, int(ceil(width)));
    for (int y = -vertical_extent; y <= vertical_extent; ++y)
    {
        float vertical_distance = abs(float(y));
        float2 horizontal = storage.distance(
            center_position + float2(0.0, y),
            dimensions);
        if (!center_hit && vertical_distance <= width)
            region_distance = min(
                region_distance,
                length(float2(horizontal.x, vertical_distance)));
        if (center_hit
            && mode == OutlineMode::SilhouetteAndCreases
            && vertical_distance <= crease_radius)
            crease_distance = min(
                crease_distance,
                length(float2(horizontal.y, vertical_distance)));
    }

    bool silhouette = !center_hit && region_distance <= width + 0.5;
    bool crease = center_hit
        && mode == OutlineMode::SilhouetteAndCreases
        && crease_distance <= crease_radius;
    return silhouette || crease ? 1.0 : 0.0;
}

float outline_geometry_edge(
    OutlineSurface center,
    OutlineSurface neighbor,
    OutlineMode mode,
    ToonSettings settings)
{
    if (center.hit != neighbor.hit)
        return 1.0;
    if (center.hit == 0 || mode != OutlineMode::SilhouetteAndCreases)
        return 0.0;
    float relative_depth = abs(center.depth - neighbor.depth)
        / max(min(center.depth, neighbor.depth), RAY_MINIMUM);
    return relative_depth > settings.outline_depth_threshold
        || dot(center.normal, neighbor.normal) < settings.outline_normal_cosine
        ? 1.0
        : 0.0;
}

public float radial_outline<T : IOutlineTracer>(
    float2 center_position,
    uint2 dimensions,
    OutlineSurface center,
    OutlineMode mode,
    float radius,
    uint probe_count,
    ToonSettings settings,
    T tracer)
{
    float edges = 0.0;
    for (uint probe_index = 0; probe_index < probe_count; ++probe_index)
    {
        float angle = 2.0 * OUTLINE_PI * float(probe_index) / float(probe_count);
        float2 offset = float2(cos(angle), sin(angle)) * radius;
        OutlineSurface neighbor = tracer.trace(
            center_position + offset,
            dimensions,
            probe_index);
        edges += outline_geometry_edge(center, neighbor, mode, settings);
    }
    return saturate(edges * 2.0 / float(probe_count));
}

float outline_relative_depth(OutlineSurface center, OutlineSurface sample)
{
    return center.hit != 0 && sample.hit != 0
        ? (sample.depth - center.depth) / max(center.depth, RAY_MINIMUM)
        : 0.0;
}

float3 outline_sampled_normal(OutlineSurface center, OutlineSurface sample)
{
    return center.hit != 0 && sample.hit != 0 ? sample.normal : center.normal;
}

float outline_threshold_response(float value, float threshold)
{
    return saturate((value - threshold) / max(threshold, RAY_MINIMUM));
}

public float sobel_outline<T : IOutlineTracer>(
    float2 center_position,
    uint2 dimensions,
    OutlineSurface center,
    OutlineMode mode,
    float radius,
    ToonSettings settings,
    T tracer)
{
    static const float2 offsets[8] = {
        float2(-1.0, -1.0), float2(0.0, -1.0), float2(1.0, -1.0),
        float2(-1.0, 0.0), float2(1.0, 0.0),
        float2(-1.0, 1.0), float2(0.0, 1.0), float2(1.0, 1.0)
    };
    static const float kernel_x[8] = {
        -1.0, 0.0, 1.0, -2.0, 2.0, -1.0, 0.0, 1.0
    };
    static const float kernel_y[8] = {
        -1.0, -2.0, -1.0, 0.0, 0.0, 1.0, 2.0, 1.0
    };
    float2 hit_gradient = float2(0.0);
    float2 depth_gradient = float2(0.0);
    float3 normal_gradient_x = float3(0.0);
    float3 normal_gradient_y = float3(0.0);
    for (uint index = 0; index < 8; ++index)
    {
        OutlineSurface neighbor = tracer.trace(
            center_position + offsets[index] * radius,
            dimensions,
            index);
        float hit = neighbor.hit != 0 ? 1.0 : 0.0;
        float depth = outline_relative_depth(center, neighbor);
        float3 normal = outline_sampled_normal(center, neighbor);
        hit_gradient += float2(kernel_x[index], kernel_y[index]) * hit;
        depth_gradient += float2(kernel_x[index], kernel_y[index]) * depth;
        normal_gradient_x += normal * kernel_x[index];
        normal_gradient_y += normal * kernel_y[index];
    }
    float response = saturate(length(hit_gradient) / 4.0);
    if (center.hit != 0 && mode == OutlineMode::SilhouetteAndCreases)
    {
        float depth_response = outline_threshold_response(
            length(depth_gradient) / 4.0,
            settings.outline_depth_threshold);
        float normal_threshold = sqrt(max(
            2.0 - 2.0 * settings.outline_normal_cosine,
            RAY_MINIMUM));
        float normal_response = outline_threshold_response(
            sqrt(
                dot(normal_gradient_x, normal_gradient_x)
                + dot(normal_gradient_y, normal_gradient_y)) / 4.0,
            normal_threshold);
        response = max(response, max(depth_response, normal_response));
    }
    return response;
}

public float roberts_cross_outline<T : IOutlineTracer>(
    float2 center_position,
    uint2 dimensions,
    OutlineSurface center,
    OutlineMode mode,
    float radius,
    ToonSettings settings,
    T tracer)
{
    OutlineSurface right = tracer.trace(
        center_position + float2(radius, 0.0), dimensions, 0u);
    OutlineSurface down = tracer.trace(
        center_position + float2(0.0, radius), dimensions, 1u);
    OutlineSurface diagonal = tracer.trace(
        center_position + float2(radius, radius), dimensions, 2u);
    float hit_x = (center.hit != 0 ? 1.0 : 0.0)
        - (diagonal.hit != 0 ? 1.0 : 0.0);
    float hit_y = (right.hit != 0 ? 1.0 : 0.0)
        - (down.hit != 0 ? 1.0 : 0.0);
    float response = saturate(length(float2(hit_x, hit_y)));
    if (center.hit != 0 && mode == OutlineMode::SilhouetteAndCreases)
    {
        float depth_x = -outline_relative_depth(center, diagonal);
        float depth_y = outline_relative_depth(center, right)
            - outline_relative_depth(center, down);
        float depth_response = outline_threshold_response(
            length(float2(depth_x, depth_y)),
            settings.outline_depth_threshold);
        float3 normal_x = center.normal - outline_sampled_normal(center, diagonal);
        float3 normal_y = outline_sampled_normal(center, right)
            - outline_sampled_normal(center, down);
        float normal_threshold = sqrt(max(
            2.0 - 2.0 * settings.outline_normal_cosine,
            RAY_MINIMUM));
        float normal_response = outline_threshold_response(
            sqrt(dot(normal_x, normal_x) + dot(normal_y, normal_y)),
            normal_threshold);
        response = max(response, max(depth_response, normal_response));
    }
    return response;
}

public float difference_of_gaussians_outline<T : IOutlineTracer>(
    float2 center_position,
    uint2 dimensions,
    OutlineSurface center,
    OutlineMode mode,
    float inner_sigma,
    uint probe_count,
    ToonSettings settings,
    T tracer)
{
    float inner_hit = 0.0;
    float outer_hit = 0.0;
    float inner_depth = 0.0;
    float outer_depth = 0.0;
    float3 inner_normal = float3(0.0);
    float3 outer_normal = float3(0.0);
    float inner_weight_sum = 0.0;
    float outer_weight_sum = 0.0;
    float outer_sigma = inner_sigma * settings.dog_radius_ratio;
    for (uint ring = 0; ring < OUTLINE_DOG_RING_COUNT; ++ring)
    {
        float radius = 3.0 * outer_sigma
            * (float(ring) + 0.5) / float(OUTLINE_DOG_RING_COUNT);
        float inner_weight = radius
            * exp(-0.5 * radius * radius / (inner_sigma * inner_sigma));
        float outer_weight = radius
            * exp(-0.5 * radius * radius / (outer_sigma * outer_sigma));
        for (uint probe_index = 0; probe_index < probe_count; ++probe_index)
        {
            float angle = 2.0 * OUTLINE_PI * (
                float(probe_index) / float(probe_count)
                + float(ring) / float(OUTLINE_DOG_RING_COUNT * probe_count));
            float2 direction = float2(cos(angle), sin(angle));
            OutlineSurface neighbor = tracer.trace(
                center_position + direction * radius,
                dimensions,
                ring * probe_count + probe_index);
            float hit = neighbor.hit != 0 ? 1.0 : 0.0;
            float depth = outline_relative_depth(center, neighbor);
            float3 normal = outline_sampled_normal(center, neighbor);
            inner_hit += hit * inner_weight;
            outer_hit += hit * outer_weight;
            inner_depth += depth * inner_weight;
            outer_depth += depth * outer_weight;
            inner_normal += normal * inner_weight;
            outer_normal += normal * outer_weight;
            inner_weight_sum += inner_weight;
            outer_weight_sum += outer_weight;
        }
    }
    float response = center.hit != 0
        ? inner_hit / inner_weight_sum - outer_hit / outer_weight_sum
        : 0.0;
    if (center.hit != 0 && mode == OutlineMode::SilhouetteAndCreases)
    {
        float depth_response = outline_threshold_response(
            abs(inner_depth / inner_weight_sum - outer_depth / outer_weight_sum),
            settings.outline_depth_threshold);
        float normal_threshold = sqrt(max(
            2.0 - 2.0 * settings.outline_normal_cosine,
            RAY_MINIMUM));
        float normal_response = outline_threshold_response(
            length(
                inner_normal / inner_weight_sum
                - outer_normal / outer_weight_sum),
            normal_threshold);
        response = max(response, max(depth_response, normal_response));
    }
    return 1.0 - exp(
        -settings.dog_sharpness
        * max(response - settings.dog_threshold, 0.0));
}
