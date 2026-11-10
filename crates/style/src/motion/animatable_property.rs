use crate::corner::Corner;
use crate::side::Side;

/// A longhand whose value a transition or a keyframe animation interpolates: a length, a color,
/// a shadow, the opacity or the transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimatableProperty {
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Margin(Side),
    Padding(Side),
    Inset(Side),
    RowGap,
    ColumnGap,
    BorderWidth(Side),
    BorderColor(Side),
    BorderRadius(Corner),
    OutlineWidth,
    OutlineColor,
    OutlineOffset,
    Color,
    BackgroundColor,
    BoxShadow,
    Opacity,
    Transform,
}

impl AnimatableProperty {
    /// Every animatable property, `color` first: the colors that are `currentColor` follow it.
    pub const ALL: [Self; 40] = [
        Self::Color,
        Self::Width,
        Self::Height,
        Self::MinWidth,
        Self::MinHeight,
        Self::MaxWidth,
        Self::MaxHeight,
        Self::Margin(Side::Top),
        Self::Margin(Side::Right),
        Self::Margin(Side::Bottom),
        Self::Margin(Side::Left),
        Self::Padding(Side::Top),
        Self::Padding(Side::Right),
        Self::Padding(Side::Bottom),
        Self::Padding(Side::Left),
        Self::Inset(Side::Top),
        Self::Inset(Side::Right),
        Self::Inset(Side::Bottom),
        Self::Inset(Side::Left),
        Self::RowGap,
        Self::ColumnGap,
        Self::BorderWidth(Side::Top),
        Self::BorderWidth(Side::Right),
        Self::BorderWidth(Side::Bottom),
        Self::BorderWidth(Side::Left),
        Self::BorderColor(Side::Top),
        Self::BorderColor(Side::Right),
        Self::BorderColor(Side::Bottom),
        Self::BorderColor(Side::Left),
        Self::BorderRadius(Corner::TopLeft),
        Self::BorderRadius(Corner::TopRight),
        Self::BorderRadius(Corner::BottomRight),
        Self::BorderRadius(Corner::BottomLeft),
        Self::OutlineWidth,
        Self::OutlineColor,
        Self::OutlineOffset,
        Self::BackgroundColor,
        Self::BoxShadow,
        Self::Opacity,
        Self::Transform,
    ];

    /// The position of the property in [`Self::ALL`].
    pub const fn position(self) -> u32 {
        match self {
            Self::Color => 0,
            Self::Width => 1,
            Self::Height => 2,
            Self::MinWidth => 3,
            Self::MinHeight => 4,
            Self::MaxWidth => 5,
            Self::MaxHeight => 6,
            Self::Margin(side) => 7 + side_position(side),
            Self::Padding(side) => 11 + side_position(side),
            Self::Inset(side) => 15 + side_position(side),
            Self::RowGap => 19,
            Self::ColumnGap => 20,
            Self::BorderWidth(side) => 21 + side_position(side),
            Self::BorderColor(side) => 25 + side_position(side),
            Self::BorderRadius(corner) => 29 + corner_position(corner),
            Self::OutlineWidth => 33,
            Self::OutlineColor => 34,
            Self::OutlineOffset => 35,
            Self::BackgroundColor => 36,
            Self::BoxShadow => 37,
            Self::Opacity => 38,
            Self::Transform => 39,
        }
    }
}

const fn side_position(side: Side) -> u32 {
    match side {
        Side::Top => 0,
        Side::Right => 1,
        Side::Bottom => 2,
        Side::Left => 3,
    }
}

const fn corner_position(corner: Corner) -> u32 {
    match corner {
        Corner::TopLeft => 0,
        Corner::TopRight => 1,
        Corner::BottomRight => 2,
        Corner::BottomLeft => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::AnimatableProperty;

    #[test]
    fn every_property_sits_at_its_own_position() {
        for (position, property) in AnimatableProperty::ALL.into_iter().enumerate() {
            assert_eq!(property.position() as usize, position, "{property:?}");
        }
    }
}
