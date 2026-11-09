#include "shapes.metal"

// The structures below mirror the `#[repr(C)]` ones of `src/render` field for field.

// A color filter of a group; a blur is a pass of its own and is skipped here.
struct FilterEntry {
    uint kind;
    float amount;
    float padding[2];
};

constant uint FILTER_BLUR = 0;
constant uint FILTER_BRIGHTNESS = 1;
constant uint FILTER_CONTRAST = 2;
constant uint FILTER_GRAYSCALE = 3;
constant uint FILTER_SATURATE = 4;
constant uint FILTER_OPACITY = 5;

constant uint BLEND_NORMAL = 0;
constant uint BLEND_MULTIPLY = 1;
constant uint BLEND_SCREEN = 2;
constant uint BLEND_OVERLAY = 3;
constant uint BLEND_DARKEN = 4;
constant uint BLEND_LIGHTEN = 5;

// A picture composited onto the target: the quad of `pixels` in device pixels of the target,
// read from the texture starting at its top left corner.
struct CompositeInstance {
    float4 pixels;
    float opacity;
    uint blend_mode;
    uint filters_start;
    uint filters_count;
    // The shape the picture is cut to, when `has_clip` is set.
    ClipShape clip;
    uint has_clip;
    uint padding[3];
};

// One pass of a separable Gaussian blur, or a plain copy when the deviation is zero.
struct BlurUniforms {
    // In device pixels.
    float deviation;
    uint radius;
    // The direction of the pass: (1, 0) or (0, 1).
    float2 direction;
    // The texel of the source under the top left pixel of the target.
    float2 source_offset;
    // The texels of the source that hold a picture; the rest of the texture is stale.
    float2 source_size;
};

struct EffectVaryings {
    float4 position [[position]];
    uint instance [[flat]];
};

vertex EffectVaryings composite_vertex(
    uint vertex_id [[vertex_id]],
    uint instance_id [[instance_id]],
    const device CompositeInstance* instances [[buffer(6)]],
    constant FrameUniforms& frame [[buffer(1)]]
) {
    const float4 pixels = instances[instance_id].pixels;
    const float2 corner = float2(vertex_id & 1, vertex_id >> 1);
    const float2 in_target = pixels.xy + corner * pixels.zw;
    const float2 normalized = in_target / frame.target_size * 2.0 - 1.0;

    EffectVaryings out;
    out.position = float4(normalized.x, -normalized.y, 0.0, 1.0);
    out.instance = instance_id;
    return out;
}

constant float3 LUMINANCE = float3(0.2126, 0.7152, 0.0722);

// The color filters of CSS over a color that is not premultiplied.
static float4 filtered(float4 color, const device FilterEntry* filters, uint start, uint count) {
    for (uint index = 0; index < count; index++) {
        const FilterEntry filter = filters[start + index];
        const float amount = filter.amount;
        switch (filter.kind) {
            case FILTER_BRIGHTNESS: {
                color.rgb = saturate(color.rgb * amount);
                break;
            }
            case FILTER_CONTRAST: {
                color.rgb = saturate((color.rgb - 0.5) * amount + 0.5);
                break;
            }
            case FILTER_GRAYSCALE: {
                const float gray = dot(color.rgb, LUMINANCE);
                color.rgb = mix(color.rgb, float3(gray), amount);
                break;
            }
            case FILTER_SATURATE: {
                const float gray = dot(color.rgb, LUMINANCE);
                color.rgb = saturate(float3(gray) + (color.rgb - float3(gray)) * amount);
                break;
            }
            case FILTER_OPACITY: {
                color.a *= amount;
                break;
            }
            default: {
                break;
            }
        }
    }
    return color;
}

static float3 blend_channels(uint mode, float3 backdrop, float3 source) {
    switch (mode) {
        case BLEND_MULTIPLY: return backdrop * source;
        case BLEND_SCREEN: return backdrop + source - backdrop * source;
        case BLEND_OVERLAY: {
            const float3 multiplied = 2.0 * backdrop * source;
            const float3 screened = 1.0 - 2.0 * (1.0 - backdrop) * (1.0 - source);
            return select(screened, multiplied, backdrop <= 0.5);
        }
        case BLEND_DARKEN: return min(backdrop, source);
        case BLEND_LIGHTEN: return max(backdrop, source);
        default: return source;
    }
}

// Composites the picture over the target as CSS does: the color is filtered, faded, blended
// with the backdrop by the mode and laid over it. Colors are blended as the sRGB values they
// are stored as, like a browser does.
fragment float4 composite_fragment(
    EffectVaryings in [[stage_in]],
    const device CompositeInstance* instances [[buffer(6)]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device FilterEntry* filters [[buffer(7)]],
    const device TransformEntry* transforms [[buffer(5)]],
    texture2d<float> picture [[texture(0)]],
    float4 backdrop [[color(0)]]
) {
    const CompositeInstance instance = instances[in.instance];
    const float2 texel = in.position.xy - instance.pixels.xy;
    float4 source = picture.read(uint2(texel));
    if (source.a > 0.0) {
        float4 straight = float4(source.rgb / source.a, source.a);
        straight = filtered(straight, filters, instance.filters_start, instance.filters_count);
        source = float4(straight.rgb * straight.a, straight.a);
    }
    source *= instance.opacity;
    if (instance.has_clip != 0) {
        source *= clip_coverage(surface_point_of(in.position.xy, frame), instance.clip, transforms, frame.scale);
    }
    if (instance.blend_mode == BLEND_NORMAL) {
        return source + backdrop * (1.0 - source.a);
    }
    const float3 source_straight = source.a > 0.0 ? source.rgb / source.a : float3(0.0);
    const float3 backdrop_straight = backdrop.a > 0.0 ? backdrop.rgb / backdrop.a : float3(0.0);
    const float3 blended = (1.0 - backdrop.a) * source_straight
        + backdrop.a * blend_channels(instance.blend_mode, backdrop_straight, source_straight);
    const float alpha = source.a + backdrop.a * (1.0 - source.a);
    return float4(blended * source.a + backdrop.rgb * (1.0 - source.a), alpha);
}

struct BlurVaryings {
    float4 position [[position]];
};

// The quad covers the whole target.
vertex BlurVaryings blur_vertex(uint vertex_id [[vertex_id]]) {
    const float2 corner = float2(vertex_id & 1, vertex_id >> 1);
    BlurVaryings out;
    out.position = float4(corner.x * 2.0 - 1.0, 1.0 - corner.y * 2.0, 0.0, 1.0);
    return out;
}

// One pass of a Gaussian blur along `direction` over premultiplied colors; texels outside the
// source are transparent. With a deviation of zero the source is copied.
fragment float4 blur_fragment(
    BlurVaryings in [[stage_in]],
    constant BlurUniforms& blur [[buffer(8)]],
    texture2d<float> source [[texture(0)]]
) {
    const float2 center = in.position.xy + blur.source_offset;
    const int2 size = int2(blur.source_size);
    if (blur.radius == 0) {
        const int2 texel = int2(floor(center));
        const bool inside = all(texel >= 0) && all(texel < size);
        return inside ? source.read(uint2(texel)) : float4(0.0);
    }
    const float two_deviations_squared = 2.0 * blur.deviation * blur.deviation;
    float4 sum = float4(0.0);
    float weights = 0.0;
    for (int offset = -int(blur.radius); offset <= int(blur.radius); offset++) {
        const float weight = exp(-float(offset * offset) / two_deviations_squared);
        const int2 texel = int2(floor(center + blur.direction * float(offset)));
        const bool inside = all(texel >= 0) && all(texel < size);
        sum += weight * (inside ? source.read(uint2(texel)) : float4(0.0));
        weights += weight;
    }
    return sum / weights;
}
