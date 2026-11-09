use bytemuck::{Pod, Zeroable};

use crate::render::rounded_rect::RoundedRect;

/// One instanced quad of the rectangle pipeline. Lengths are CSS pixels on the surface.
///
/// What the shapes, the colors and the parameters mean depends on the kind:
///
/// | kind | `outer` | `inner` | `colors` | `parameters` |
/// |---|---|---|---|---|
/// | fill | the filled shape | — | the fill | — |
/// | linear gradient | the filled shape | the tile | — | direction x, y; line length |
/// | radial gradient | the filled shape | the tile | — | center x, y; radius x, y |
/// | border | the outer edge | the inner edge | top, right, bottom, left | — |
/// | box | the outer edge | the inner edge | the border: top, right, bottom, left | the fill |
/// | shadow | the blurred shape | the box that hides it | the shadow | blur deviation |
/// | inset shadow | the blurred opening | the box that shows it | the shadow | blur deviation |
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct RectInstance {
    /// The quad to rasterize: origin and size.
    pub(crate) bounds: [f32; 4],
    pub(crate) outer: RoundedRect,
    pub(crate) inner: RoundedRect,
    /// Premultiplied.
    pub(crate) colors: [[f32; 4]; 4],
    pub(crate) parameters: [f32; 4],
    /// A [`RectKind`](crate::render::rect_kind::RectKind).
    pub(crate) kind: u32,
    pub(crate) gradient_stops_start: u32,
    pub(crate) gradient_stops_count: u32,
    /// The rounded clips the instance is masked by, in the clip buffer of the frame.
    pub(crate) clips_start: u32,
    pub(crate) clips_count: u32,
    pub(crate) padding: [u32; 3],
}
