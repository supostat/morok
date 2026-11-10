use crate::motion::animatable_property::AnimatableProperty;

/// A set of animatable properties: what one entry of `transition-property` names, a longhand,
/// a shorthand or `all`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AnimatableProperties(u64);

impl AnimatableProperties {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self((1 << AnimatableProperty::ALL.len()) - 1);

    pub const fn of(property: AnimatableProperty) -> Self {
        Self(1 << property.position())
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The properties of `self` that `other` does not hold.
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn contains(self, property: AnimatableProperty) -> bool {
        self.0 & Self::of(property).0 != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The properties of the set in the order of [`AnimatableProperty::ALL`].
    pub fn iter(self) -> impl Iterator<Item = AnimatableProperty> {
        AnimatableProperty::ALL
            .into_iter()
            .filter(move |property| self.contains(*property))
    }
}

#[cfg(test)]
mod tests {
    use super::AnimatableProperties;
    use crate::motion::animatable_property::AnimatableProperty;
    use crate::side::Side;

    #[test]
    fn all_holds_every_property_and_none_holds_none() {
        for property in AnimatableProperty::ALL {
            assert!(AnimatableProperties::ALL.contains(property));
            assert!(!AnimatableProperties::NONE.contains(property));
        }
    }

    #[test]
    fn a_union_holds_the_properties_of_both_sets() {
        let set = AnimatableProperties::of(AnimatableProperty::Opacity)
            .union(AnimatableProperties::of(AnimatableProperty::Padding(Side::Left)));

        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            [AnimatableProperty::Padding(Side::Left), AnimatableProperty::Opacity]
        );
    }
}
