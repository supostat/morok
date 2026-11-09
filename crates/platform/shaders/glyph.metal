#include "shapes.metal"

// Mirrors `GlyphInstance` of `src/render`.
struct GlyphInstance {
    float4 bounds;
    float4 color;
    float2 atlas_origin;
    uint clips_start;
    uint clips_count;
    uint is_color;
    uint padding[3];
};

struct GlyphVaryings {
    float4 position [[position]];
    uint instance [[flat]];
};

vertex GlyphVaryings glyph_vertex(
    uint vertex_id [[vertex_id]],
    uint instance_id [[instance_id]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device GlyphInstance* instances [[buffer(4)]]
) {
    const float4 bounds = instances[instance_id].bounds;
    const float2 corner = float2(vertex_id & 1, vertex_id >> 1);
    const float2 surface_point = bounds.xy + corner * bounds.zw;
    const float2 normalized = surface_point / frame.viewport_size * 2.0 - 1.0;

    GlyphVaryings out;
    out.position = float4(normalized.x, -normalized.y, 0.0, 1.0);
    out.instance = instance_id;
    return out;
}

// A glyph is drawn texel for pixel: its quad starts on a device pixel, so the texel of a
// fragment is read, not sampled. A mask glyph is tinted with the color of the text; a color
// glyph keeps its own colors and takes only the opacity of the text.
fragment float4 glyph_fragment(
    GlyphVaryings in [[stage_in]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device RoundedRect* clips [[buffer(3)]],
    const device GlyphInstance* instances [[buffer(4)]],
    texture2d<float> atlas [[texture(0)]]
) {
    const GlyphInstance instance = instances[in.instance];
    const float2 quad_origin = instance.bounds.xy * frame.scale;
    const uint2 texel = uint2(instance.atlas_origin + floor(in.position.xy - quad_origin));
    const float4 stored = atlas.read(texel);
    float4 color = instance.is_color != 0 ? stored * instance.color.a : instance.color * stored.r;

    const float2 surface_point = in.position.xy / frame.scale;
    for (uint index = 0; index < instance.clips_count; index++) {
        color *= rounded_rect_coverage(surface_point, clips[instance.clips_start + index], frame.scale);
    }
    return color;
}
