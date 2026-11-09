#include "shapes.metal"

// Mirrors `GlyphInstance` of `src/render`.
struct GlyphInstance {
    float4 bounds;
    float4 color;
    float2 atlas_origin;
    uint clips_start;
    uint clips_count;
    uint is_color;
    uint transform;
    uint padding[2];
};

struct GlyphVaryings {
    float4 position [[position]];
    // The texel of the atlas under the fragment, interpolated over the quad.
    float2 texel;
    uint instance [[flat]];
};

vertex GlyphVaryings glyph_vertex(
    uint vertex_id [[vertex_id]],
    uint instance_id [[instance_id]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device GlyphInstance* instances [[buffer(4)]],
    const device TransformEntry* transforms [[buffer(5)]]
) {
    const GlyphInstance instance = instances[instance_id];
    const TransformEntry transform = transforms[instance.transform];
    const float2 corner = float2(vertex_id & 1, vertex_id >> 1);
    const float2 local_point = instance.bounds.xy + corner * instance.bounds.zw;
    const float2 surface_point = transform_point(transform.forward_linear, transform.forward_offset, local_point);

    GlyphVaryings out;
    out.position = clip_position(surface_point, frame);
    out.texel = instance.atlas_origin + corner * instance.bounds.zw * frame.scale;
    out.instance = instance_id;
    return out;
}

// A glyph is drawn texel for pixel: its quad starts on a device pixel, so the texel under a
// fragment is read, not sampled. A mask glyph is tinted with the color of the text; a color
// glyph keeps its own colors and takes only the opacity of the text.
fragment float4 glyph_fragment(
    GlyphVaryings in [[stage_in]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device ClipShape* clips [[buffer(3)]],
    const device GlyphInstance* instances [[buffer(4)]],
    const device TransformEntry* transforms [[buffer(5)]],
    texture2d<float> atlas [[texture(0)]]
) {
    const GlyphInstance instance = instances[in.instance];
    const float4 stored = atlas.read(uint2(floor(in.texel)));
    float4 color = instance.is_color != 0 ? stored * instance.color.a : instance.color * stored.r;
    const float2 on_surface = surface_point_of(in.position.xy, frame);
    for (uint index = 0; index < instance.clips_count; index++) {
        color *= clip_coverage(on_surface, clips[instance.clips_start + index], transforms, frame.scale);
    }
    return color;
}
