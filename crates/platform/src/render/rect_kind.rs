/// What an instance draws; the values are the `KIND_` constants of the shader.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RectKind {
    Fill = 0,
    LinearGradient = 1,
    RadialGradient = 2,
    Border = 3,
    Shadow = 4,
    InsetShadow = 5,
}
