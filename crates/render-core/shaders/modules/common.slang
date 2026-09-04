import math;

uint linear_index(uint3 dispatch_thread_id) { return dispatch_thread_id.x; }

float rust_round(float value) {
  return value < 0.0f ? ceil(value - 0.5f) : floor(value + 0.5f);
}

float rem_euclid(float value, float modulus) {
  float remainder = fmod(value, modulus);
  return remainder < 0.0f ? remainder + modulus : remainder;
}

float smoothstep_exact(float edge0, float edge1, float value) {
  float t = clamp((value - edge0) / max(edge1 - edge0, 0.00001f), 0.0f, 1.0f);
  return t * t * (3.0f - 2.0f * t);
}

float hash_unit(int x, int y, uint seed) {
  uint hash =
      uint(x) * 0x8da6b343u + uint(y) * 0xd8163841u + seed * 0xcb1ab31fu;
  hash ^= hash >> 16;
  hash *= 0x7feb352du;
  hash ^= hash >> 15;
  hash *= 0x846ca68bu;
  hash ^= hash >> 16;
  return float(hash) / 4294967296.0f;
}

float streak_wipe_alpha(float2 point, float2 canvas_size, float visibility,
                        float direction_degrees, float line_width,
                        float variation, float softness) {
  visibility = clamp(visibility, 0.0f, 1.0f);
  if (visibility <= 0.0f)
    return 0.0f;
  if (visibility >= 1.0f)
    return 1.0f;
  float radians = direction_degrees * (PI / 180.0f);
  float2 direction = float2(sin_f32(radians + PI * 0.5f), sin_f32(radians));
  float2 centered = point - canvas_size * 0.5f;
  float extent = max(dot(abs(direction), canvas_size), 0.00001f);
  float position = clamp(dot(centered, direction) / extent + 0.5f, 0.0f, 1.0f);
  int band = int(floor_f32(dot(centered, float2(-direction.y, direction.x)) /
                           max(line_width, 1.0f)));
  variation = clamp(variation, 0.0f, 1.0f);
  float edge =
      visibility * (1.0f + variation) - hash_unit(band, 0, 37u) * variation;
  softness = clamp(softness, 0.0f, 128.0f) / extent;
  return 1.0f - smoothstep_exact(edge - softness, edge + softness + 0.00001f,
                                 position);
}

float2 rotate_degrees(float2 value, float degrees) {
  float radians = degrees * (PI / 180.0f);
  float sine = sin(radians);
  float cosine = cos(radians);
  return float2(value.x * cosine - value.y * sine,
                value.x * sine + value.y * cosine);
}

uint clamp_index(int value, uint maximum) {
  return uint(clamp(value, 0, int(maximum) - 1));
}
