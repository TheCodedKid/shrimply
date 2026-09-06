module render_3d_material;

import render_3d_abi;
import render_3d_pbr;

public float wrap_texture_coordinate(float value, uint mode)
{
    if (mode == 0)
        return saturate(value);
    if (mode == 1)
        return 1.0 - abs(frac(value * 0.5) * 2.0 - 1.0);
    return frac(value);
}

public float2 mapped_texture_coordinates(TextureMapping mapping, float2 uv)
{
    float sine = sin(mapping.rotation_filter.x);
    float cosine = cos(mapping.rotation_filter.x);
    float2 scaled = uv * mapping.transform.zw;
    float2 transformed = mapping.transform.xy + float2(
        cosine * scaled.x - sine * scaled.y,
        sine * scaled.x + cosine * scaled.y);
    return float2(
        wrap_texture_coordinate(transformed.x, mapping.settings.z),
        wrap_texture_coordinate(transformed.y, mapping.settings.w));
}

public float4 material_base_color(
    float4 factor,
    float4 vertex_color,
    float4 texture_color)
{
    texture_color.rgb = float3(
        srgb_to_linear(texture_color.r),
        srgb_to_linear(texture_color.g),
        srgb_to_linear(texture_color.b));
    return factor * vertex_color * texture_color;
}

public float2 material_texture_coordinates(
    TextureMapping mapping,
    float2 first,
    float2 second,
    float2 third,
    float3 barycentric)
{
    float2 uv = first * barycentric.x
        + second * barycentric.y
        + third * barycentric.z;
    return mapping.atlas.xy
        + mapped_texture_coordinates(mapping, uv) * mapping.atlas.zw;
}

public float4 material_vertex_color(
    float4 first,
    float4 second,
    float4 third,
    float3 barycentric)
{
    return first * barycentric.x
        + second * barycentric.y
        + third * barycentric.z;
}

public float material_metallic(bool has_material, float factor, float4 sampled, float fallback)
{
    return has_material ? factor * sampled.b : fallback;
}

public float material_roughness(bool has_material, float factor, float4 sampled, float fallback)
{
    return has_material ? factor * sampled.g : fallback;
}

public float material_shadow_coverage(
    uint alpha_mode,
    float alpha,
    float alpha_cutoff)
{
    if (alpha_mode == 0)
        return 1.0;
    if (alpha_mode == 1)
        return alpha >= alpha_cutoff ? 1.0 : 0.0;
    return saturate(alpha);
}

public float3 material_shadow_transmittance(
    float4 base_color,
    float coverage,
    float metallic,
    float cosine,
    PbrSettings settings)
{
    float fresnel = pbr_fresnel(
        float3(1.0),
        0.0,
        abs(cosine),
        settings.ior).r;
    float transmission = saturate(
        settings.transmission * (1.0 - metallic) * (1.0 - fresnel));
    return lerp(
        float3(1.0),
        saturate(base_color.rgb) * transmission,
        coverage);
}

public bool material_hit_accepted(
    uint material_index,
    float2 barycentrics,
    uint alpha_mode,
    float alpha,
    float alpha_cutoff)
{
    if (alpha_mode == 0)
        return true;
    if (alpha_mode == 1)
        return alpha >= alpha_cutoff;

    const uint HASH_MULTIPLIER = 0x9e3779b9u;
    uint hash = material_index * HASH_MULTIPLIER
        ^ asuint(barycentrics.x)
        ^ (asuint(barycentrics.y) << 1);
    hash ^= hash >> 16;
    return float(hash & 0x00ffffffu) / float(0x01000000u) < saturate(alpha);
}
