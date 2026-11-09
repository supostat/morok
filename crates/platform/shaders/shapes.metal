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

struct FrameUniforms {
    float2 viewport_size;
    float scale;
    float padding;
};

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
