struct ObjToonTextureTracer : IToonTextureTracer
{
    ToonTextureSample sample(float2 position, uint2 dimensions)
    {
        RayPayload payload = trace_surface(position, dimensions, true, 0);
        ToonTextureSample result = {
            payload.hit != 0,
            payload.color.rgb,
            payload.normal,
            payload.depth
        };
        return result;
    }
};

float3 screen_space_kuwahara(
    float3 center_color,
    float3 center_normal,
    float center_depth,
    ToonSettings settings)
{
    uint2 dimensions = DispatchRaysDimensions().xy;
    float2 center_position = float2(DispatchRaysIndex().xy) + 0.5;
    ObjToonTextureTracer tracer;
    return toon_screen_space_kuwahara(
        center_color,
        center_normal,
        center_depth,
        center_position,
        dimensions,
        settings,
        tracer);
}
