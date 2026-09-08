static const float F32_EPSILON = 1.1920929e-7f;
static const float PI = 3.14159265358979323846f;
static const float SQRT_2 = 1.4142135623730950488f;

float floor_f32(float value) {
  float truncated = float(int(value));
  return truncated > value ? truncated - 1.0f : truncated;
}

float ceil_f32(float value) { return -floor_f32(-value); }

float lerp_f32(float left, float right, float amount) {
  return left + (right - left) * amount;
}

float gaussian_weight(int distance_value, uint radius) {
  float sigma = max(float(radius) * 0.5f, 0.5f);
  float exponent =
      -float(distance_value * distance_value) / (2.0f * sigma * sigma);
  float result = 1.0f + exponent / 256.0f;
  for (uint i = 0u; i < 8u; ++i)
    result *= result;
  return max(result, 0.0f);
}

float sin_f32(float value) {
  float tau = PI * 2.0f;
  float x = value - floor_f32((value + PI) / tau) * tau;
  if (x > PI)
    x -= tau;
  float x2 = x * x;
  return x * (1.0f + x2 * (-1.0f / 6.0f +
                           x2 * (1.0f / 120.0f +
                                 x2 * (-1.0f / 5040.0f + x2 / 362880.0f))));
}

float atan2_f32(float y, float x) {
  float abs_y = (y < 0.0f ? -y : y) + 0.0000001f;
  float base;
  float ratio;
  if (x >= 0.0f) {
    base = PI * 0.25f;
    ratio = (x - abs_y) / (x + abs_y);
  } else {
    base = 3.0f * PI * 0.25f;
    ratio = (x + abs_y) / (abs_y - x);
  }
  float angle = base + (0.1963f * ratio * ratio - 0.9817f) * ratio;
  return y < 0.0f ? -angle : angle;
}
