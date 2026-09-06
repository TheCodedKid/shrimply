module render_3d_environment;

static const float ENVIRONMENT_PI = 3.14159265358979323846;

public float2 environment_coordinates(
    float4x4 inverse_rotation,
    float3 direction)
{
    float3 rotated = normalize(mul(
        inverse_rotation,
        float4(direction, 0.0)).xyz);
    float longitude = atan2(rotated.z, rotated.x);
    return float2(
        frac(longitude / (2.0 * ENVIRONMENT_PI) + 0.5),
        acos(clamp(rotated.y, -1.0, 1.0)) / ENVIRONMENT_PI);
}
