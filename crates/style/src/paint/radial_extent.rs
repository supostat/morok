/// How far a radial gradient reaches from its center.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadialExtent {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
}
