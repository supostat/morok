#include "shapes.metal"

// Mirrors `ImageInstance` of `src/render`.
struct ImageInstance {
    float4 bounds;
    float4 texels;
    uint clips_start;
    uint clips_count;
    uint transform;
    uint padding;
};

struct ImageVaryings {
    float4 position [[position]];
    // The texel of the picture under the fragment, interpolated over the quad.
    float2 texel;
    uint instance [[flat]];
};

vertex ImageVaryings image_vertex(
    uint vertex_id [[vertex_id]],
    uint instance_id [[instance_id]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device TransformEntry* transforms [[buffer(5)]],
    const device ImageInstance* instances [[buffer(9)]]
) {
    const ImageInstance instance = instances[instance_id];
    const TransformEntry transform = transforms[instance.transform];
    const float2 corner = float2(vertex_id & 1, vertex_id >> 1);
    const float2 local_point = instance.bounds.xy + corner * instance.bounds.zw;
    const float2 surface_point = transform_point(transform.forward_linear, transform.forward_offset, local_point);

    ImageVaryings out;
    out.position = clip_position(surface_point, frame);
    out.texel = instance.texels.xy + corner * instance.texels.zw;
    out.instance = instance_id;
    return out;
}

// A picture is sampled with linear filtering: a quad rarely shows it texel for pixel. The
// colors of the texture are premultiplied.
fragment float4 image_fragment(
    ImageVaryings in [[stage_in]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device ClipShape* clips [[buffer(3)]],
    const device TransformEntry* transforms [[buffer(5)]],
    const device ImageInstance* instances [[buffer(9)]],
    texture2d<float> picture [[texture(0)]]
) {
    constexpr sampler linear_sampler(coord::pixel, filter::linear, address::clamp_to_edge);
    const ImageInstance instance = instances[in.instance];
    float4 color = picture.sample(linear_sampler, in.texel);
    const float2 on_surface = surface_point_of(in.position.xy, frame);
    for (uint index = 0; index < instance.clips_count; index++) {
        color *= clip_coverage(on_surface, clips[instance.clips_start + index], transforms, frame.scale);
    }
    return color;
}
