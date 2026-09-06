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
    int2 pixel = int2(clamp(
        floor(atlas_uv * float2(width, height)),
        float2(0.0),
        float2(width - 1, height - 1)));
    return material_texture.Load(int3(pixel, 0));
}

float4 surface_base_color(uint first, float3 barycentric, MeshMaterial material)
{
    float4 vertex_color = material_vertex_color(
        vertex_colors[first],
        vertex_colors[first + 1],
        vertex_colors[first + 2],
        barycentric);
    float4 sampled = sample_material_texture(
        material.base_color_texture,
        first,
        barycentric);
    return material_base_color(material.base_color_factor, vertex_color, sampled);
}
