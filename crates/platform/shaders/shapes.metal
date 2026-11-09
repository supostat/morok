#pragma once

#include <metal_stdlib>

using namespace metal;

// What every pipeline shares. The structures mirror the `#[repr(C)]` ones of `src/render`
// field for field.

struct RoundedRect {
    float4 rect;
    float4 horizontal_radii;
    float4 vertical_radii;
};

// The target a pass draws into: a window of the surface in device pixels, at `origin` of the
// surface, `target_size` pixels large.
struct FrameUniforms {
    float2 viewport_size;
    float scale;
    float padding;
    float2 origin;
    float2 target_size;
};

// A 2D affine transform of the surface, as CSS `matrix(a, b, c, d, tx, ty)`, with its inverse
// and the factor it scales lengths by. Entry 0 is the identity.
struct TransformEntry {
    float4 forward_linear;
    float2 forward_offset;
    float2 padding_forward;
    float4 inverse_linear;
    float2 inverse_offset;
    float scale_factor;
    float padding_inverse;
};

// A clip shape in the coordinates its transform maps to the surface.
struct ClipShape {
    RoundedRect shape;
    uint transform;
    uint padding[3];
};

static float2 transform_point(float4 linear, float2 offset, float2 point) {
    return float2(linear.x * point.x + linear.z * point.y + offset.x, linear.y * point.x + linear.w * point.y + offset.y);
}

// The position in clip space of a point of the surface drawn into the target of `frame`.
static float4 clip_position(float2 surface_point, FrameUniforms frame) {
    const float2 in_target = surface_point * frame.scale - frame.origin;
    const float2 normalized = in_target / frame.target_size * 2.0 - 1.0;
    return float4(normalized.x, -normalized.y, 0.0, 1.0);
}

// The point of the surface a fragment of the target lies on.
static float2 surface_point_of(float2 position, FrameUniforms frame) {
    return (position + frame.origin) / frame.scale;
}

// The signed distance from a point to the edge of a rounded rectangle, negative inside. Exact
// for circular corners; for elliptical ones a first-order estimate that is exact on the edge,
// which is where anti-aliasing reads it.
static float rounded_rect_distance(float2 surface_point, RoundedRect shape) {
    const float2 half_size = shape.rect.zw * 0.5;
    const float2 from_center = surface_point - (shape.rect.xy + half_size);
    const uint corner = from_center.y > 0.0 ? (from_center.x > 0.0 ? 2 : 3) : (from_center.x > 0.0 ? 1 : 0);
    const float2 radius = float2(shape.horizontal_radii[corner], shape.vertical_radii[corner]);
    const float2 from_edges = abs(from_center) - half_size;
    const float2 from_ellipse_center = from_edges + radius;
    if (radius.x <= 0.0 || radius.y <= 0.0 || from_ellipse_center.x <= 0.0 || from_ellipse_center.y <= 0.0) {
        return min(max(from_edges.x, from_edges.y), 0.0) + length(max(from_edges, 0.0));
    }
    const float2 unit = from_ellipse_center / radius;
    const float level = length(unit);
    return (level - 1.0) * level / length(unit / radius);
}

// The share of a pixel a shape covers, for a pixel whose center lies `distance` from its edge.
static float edge_coverage(float distance, float scale) {
    return saturate(0.5 - distance * scale);
}

static float rounded_rect_coverage(float2 surface_point, RoundedRect shape, float scale) {
    return edge_coverage(rounded_rect_distance(surface_point, shape), scale);
}

// The coverage of a clip shape at a point of the surface: the point is carried into the
// coordinates of the shape, and the edge is as wide as a device pixel is there.
static float clip_coverage(float2 surface_point, ClipShape clip, const device TransformEntry* transforms, float scale) {
    const TransformEntry transform = transforms[clip.transform];
    const float2 local_point = transform_point(transform.inverse_linear, transform.inverse_offset, surface_point);
    return rounded_rect_coverage(local_point, clip.shape, scale * transform.scale_factor);
}
