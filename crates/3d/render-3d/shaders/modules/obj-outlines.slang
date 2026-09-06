struct ObjRegionOutlineStorage : IRegionOutlineStorage
{
    float4 guide(float2 position, uint2 dimensions)
    {
        if (any(position < float2(0.0)) || any(position >= float2(dimensions)))
            return float4(0.0, 0.0, 0.0, RAY_MAXIMUM);
        return outline_guide[uint2(position)];
    }

    float2 distance(float2 position, uint2 dimensions)
    {
        if (any(position < float2(0.0)) || any(position >= float2(dimensions)))
            return float2(RAY_MAXIMUM);
        return outline_distance[uint2(position)];
    }
};

OutlineSurface outline_surface(RayPayload payload)
{
    OutlineSurface result = {
        payload.hit,
        payload.normal,
        payload.depth,
        payload.facing
    };
    return result;
}

struct ObjOutlineTracer : IOutlineTracer
{
    OutlineSurface trace(float2 position, uint2 dimensions, uint sample_index)
    {
        return outline_surface(trace_surface(
            position,
            dimensions,
            true,
            sample_index));
    }
};
