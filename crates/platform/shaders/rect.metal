#include <metal_stdlib>

using namespace metal;

// The structures below mirror the `#[repr(C)]` ones of `src/render` field for field; what the
// fields of an instance mean for each kind is documented there.

struct RoundedRect {
    float4 rect;
    float4 horizontal_radii;
    float4 vertical_radii;
};

struct RectInstance {
    float4 bounds;
    RoundedRect outer;
    RoundedRect inner;
    float4 colors[4];
    float4 parameters;
    uint kind;
    uint gradient_stops_start;
    uint gradient_stops_count;
    uint clips_start;
    uint clips_count;
    uint padding[3];
};

struct GradientStop {
    float4 color;
    float offset;
    float padding[3];
};

struct FrameUniforms {
    float2 viewport_size;
    float scale;
    float padding;
};

constant uint KIND_FILL = 0;
constant uint KIND_LINEAR_GRADIENT = 1;
constant uint KIND_RADIAL_GRADIENT = 2;
constant uint KIND_BORDER = 3;
constant uint KIND_SHADOW = 4;
constant uint KIND_INSET_SHADOW = 5;

// A blur narrower than this is drawn as a hard edge: the Gaussian would divide by its width.
constant float MINIMUM_BLUR_DEVIATION = 0.01;
constant uint BLUR_ROW_SAMPLES = 6;

struct RectVaryings {
    float4 position [[position]];
    uint instance [[flat]];
};

vertex RectVaryings rect_vertex(
    uint vertex_id [[vertex_id]],
    uint instance_id [[instance_id]],
    const device RectInstance* instances [[buffer(0)]],
    constant FrameUniforms& frame [[buffer(1)]]
) {
    const float4 bounds = instances[instance_id].bounds;
    const float2 corner = float2(vertex_id & 1, vertex_id >> 1);
    const float2 surface_point = bounds.xy + corner * bounds.zw;
    const float2 normalized = surface_point / frame.viewport_size * 2.0 - 1.0;

    RectVaryings out;
    out.position = float4(normalized.x, -normalized.y, 0.0, 1.0);
    out.instance = instance_id;
    return out;
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

static float4 gradient_color(float position, const device GradientStop* stops, uint start, uint count) {
    float4 color = stops[start].color;
    float previous_offset = stops[start].offset;
    if (position <= previous_offset) {
        return color;
    }
    for (uint index = 1; index < count; index++) {
        const GradientStop stop = stops[start + index];
        if (position < stop.offset) {
            return mix(color, stop.color, (position - previous_offset) / (stop.offset - previous_offset));
        }
        color = stop.color;
        previous_offset = stop.offset;
    }
    return color;
}

// The background image is one tile over the padding box, repeated under the borders.
static float2 point_in_tile(float2 surface_point, float4 tile) {
    return surface_point - tile.zw * floor((surface_point - tile.xy) / tile.zw);
}

static float linear_gradient_position(float2 tile_point, float4 tile, float4 parameters) {
    const float2 from_center = tile_point - (tile.xy + tile.zw * 0.5);
    return dot(from_center, parameters.xy) / parameters.z + 0.5;
}

static float radial_gradient_position(float2 tile_point, float4 parameters) {
    return length((tile_point - parameters.xy) / max(parameters.zw, 1e-6));
}

// Each side of a border owns the points that lie less deep into it, by the measure of its own
// width, than into any other side: the sides meet on the lines from the outer corners to the
// inner ones. The meeting line is anti-aliased like any other edge.
static float4 border_side_color(float2 surface_point, RectInstance instance, float scale) {
    const float4 top = instance.colors[0];
    if (all(top == instance.colors[1]) && all(top == instance.colors[2]) && all(top == instance.colors[3])) {
        return top;
    }
    const float4 outer = instance.outer.rect;
    const float4 inner = instance.inner.rect;
    const float4 widths = float4(
        inner.y - outer.y,
        outer.x + outer.z - inner.x - inner.z,
        outer.y + outer.w - inner.y - inner.w,
        inner.x - outer.x
    );
    const float4 depths = float4(
        surface_point.y - outer.y,
        outer.x + outer.z - surface_point.x,
        outer.y + outer.w - surface_point.y,
        surface_point.x - outer.x
    );
    const float2 inward[4] = {float2(0.0, 1.0), float2(-1.0, 0.0), float2(0.0, -1.0), float2(1.0, 0.0)};

    uint nearest = 0;
    uint second = 0;
    float nearest_depth = INFINITY;
    float second_depth = INFINITY;
    for (uint side = 0; side < 4; side++) {
        if (widths[side] <= 0.0) {
            continue;
        }
        const float depth = depths[side] / widths[side];
        if (depth < nearest_depth) {
            second = nearest;
            second_depth = nearest_depth;
            nearest = side;
            nearest_depth = depth;
        } else if (depth < second_depth) {
            second = side;
            second_depth = depth;
        }
    }
    if (isinf(second_depth)) {
        return instance.colors[nearest];
    }
    const float2 slope = inward[second] / widths[second] - inward[nearest] / widths[nearest];
    const float distance_to_meeting_line = (second_depth - nearest_depth) / length(slope);
    return mix(instance.colors[second], instance.colors[nearest], saturate(0.5 + distance_to_meeting_line * scale));
}

static float gaussian(float distance, float deviation) {
    return exp(-(distance * distance) / (2.0 * deviation * deviation)) / (sqrt(2.0 * M_PI_F) * deviation);
}

static float2 error_function(float2 value) {
    const float2 magnitude = abs(value);
    float2 denominator = 1.0 + (0.278393 + (0.230389 + 0.078108 * (magnitude * magnitude)) * magnitude) * magnitude;
    denominator *= denominator;
    return sign(value) * (1.0 - 1.0 / (denominator * denominator));
}

// How much of one row of a rounded rectangle a Gaussian centered `horizontal` from the middle
// of the row covers. `row` is the vertical position of the row from the center of the shape.
static float blurred_row_coverage(float horizontal, float row, float deviation, float2 half_size, float4 radii) {
    const float2 radius = row < 0.0 ? radii.xy : radii.wz;
    const float2 into_corner = min(half_size.y - radius - abs(row), 0.0);
    const float2 half_width = half_size.x - radius + sqrt(max(radius * radius - into_corner * into_corner, 0.0));
    const float2 integral = 0.5 + 0.5 * error_function(
        (horizontal + float2(-half_width.y, half_width.x)) * (M_SQRT1_2_F / deviation)
    );
    return integral.y - integral.x;
}

// The coverage of a rounded rectangle blurred by a Gaussian: exact along a row, integrated
// across the rows the Gaussian reaches. The corners are taken as circles.
static float blurred_rounded_rect_coverage(float2 surface_point, RoundedRect shape, float deviation, float scale) {
    if (deviation < MINIMUM_BLUR_DEVIATION) {
        return rounded_rect_coverage(surface_point, shape, scale);
    }
    const float2 half_size = shape.rect.zw * 0.5;
    const float2 from_center = surface_point - (shape.rect.xy + half_size);
    const float reach = 3.0 * deviation;
    const float first_row = clamp(-reach, from_center.y - half_size.y, from_center.y + half_size.y);
    const float last_row = clamp(reach, from_center.y - half_size.y, from_center.y + half_size.y);
    const float row_step = (last_row - first_row) / float(BLUR_ROW_SAMPLES);
    float offset = first_row + row_step * 0.5;
    float coverage = 0.0;
    for (uint sample = 0; sample < BLUR_ROW_SAMPLES; sample++) {
        coverage += blurred_row_coverage(from_center.x, from_center.y - offset, deviation, half_size, shape.horizontal_radii)
            * gaussian(offset, deviation) * row_step;
        offset += row_step;
    }
    return saturate(coverage);
}

fragment float4 rect_fragment(
    RectVaryings in [[stage_in]],
    const device RectInstance* instances [[buffer(0)]],
    constant FrameUniforms& frame [[buffer(1)]],
    const device GradientStop* gradient_stops [[buffer(2)]],
    const device RoundedRect* clips [[buffer(3)]]
) {
    const RectInstance instance = instances[in.instance];
    const float scale = frame.scale;
    const float2 surface_point = in.position.xy / scale;

    float4 color;
    switch (instance.kind) {
        case KIND_FILL: {
            color = instance.colors[0] * rounded_rect_coverage(surface_point, instance.outer, scale);
            break;
        }
        case KIND_LINEAR_GRADIENT:
        case KIND_RADIAL_GRADIENT: {
            const float4 tile = instance.inner.rect;
            const float2 tile_point = point_in_tile(surface_point, tile);
            const float position = instance.kind == KIND_LINEAR_GRADIENT
                ? linear_gradient_position(tile_point, tile, instance.parameters)
                : radial_gradient_position(tile_point, instance.parameters);
            color = gradient_color(position, gradient_stops, instance.gradient_stops_start, instance.gradient_stops_count)
                * rounded_rect_coverage(surface_point, instance.outer, scale);
            break;
        }
        case KIND_BORDER: {
            const float ring = min(
                rounded_rect_coverage(surface_point, instance.outer, scale),
                1.0 - rounded_rect_coverage(surface_point, instance.inner, scale)
            );
            color = border_side_color(surface_point, instance, scale) * ring;
            break;
        }
        case KIND_SHADOW: {
            const float shadow = blurred_rounded_rect_coverage(surface_point, instance.outer, instance.parameters.x, scale);
            color = instance.colors[0] * shadow * (1.0 - rounded_rect_coverage(surface_point, instance.inner, scale));
            break;
        }
        case KIND_INSET_SHADOW: {
            const float light = blurred_rounded_rect_coverage(surface_point, instance.outer, instance.parameters.x, scale);
            color = instance.colors[0] * (1.0 - light) * rounded_rect_coverage(surface_point, instance.inner, scale);
            break;
        }
        default: {
            color = float4(0.0);
            break;
        }
    }

    for (uint index = 0; index < instance.clips_count; index++) {
        color *= rounded_rect_coverage(surface_point, clips[instance.clips_start + index], scale);
    }
    return color;
}
