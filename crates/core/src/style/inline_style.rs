use std::mem;

use morok_style::Declaration;

/// The declarations set on one element directly; they override every rule of the style sheet.
#[derive(Debug, Default)]
pub(crate) struct InlineStyle {
    declarations: Vec<Declaration>,
    revision: u32,
}

impl InlineStyle {
    /// Sets a property, replacing the declaration that set it before.
    pub(crate) fn set(&mut self, declaration: Declaration) {
        self.revision = self.revision.wrapping_add(1);
        match self
            .declarations
            .iter_mut()
            .find(|existing| declares_same_property(existing, &declaration))
        {
            Some(existing) => *existing = declaration,
            None => self.declarations.push(declaration),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.declarations.clear();
    }

    pub(crate) fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub(crate) fn revision(&self) -> u32 {
        self.revision
    }
}

fn declares_same_property(existing: &Declaration, new: &Declaration) -> bool {
    match (existing, new) {
        (Declaration::Margin(existing_side, _), Declaration::Margin(new_side, _))
        | (Declaration::Padding(existing_side, _), Declaration::Padding(new_side, _))
        | (Declaration::Inset(existing_side, _), Declaration::Inset(new_side, _))
        | (Declaration::BorderWidth(existing_side, _), Declaration::BorderWidth(new_side, _))
        | (Declaration::BorderStyle(existing_side, _), Declaration::BorderStyle(new_side, _))
        | (Declaration::BorderColor(existing_side, _), Declaration::BorderColor(new_side, _)) => {
            existing_side == new_side
        }
        (Declaration::BorderRadius(existing_corner, _), Declaration::BorderRadius(new_corner, _)) => {
            existing_corner == new_corner
        }
        _ => mem::discriminant(existing) == mem::discriminant(new),
    }
}

#[cfg(test)]
mod tests {
    use morok_style::{Declaration, Display, LengthPercentageAuto, Side};

    use super::InlineStyle;

    #[test]
    fn a_property_set_twice_keeps_its_last_value_at_its_first_position() {
        let mut inline_style = InlineStyle::default();

        inline_style.set(Declaration::Display(Display::Flex));
        inline_style.set(Declaration::FlexGrow(1.0));
        inline_style.set(Declaration::Display(Display::Grid));

        assert_eq!(
            inline_style.declarations(),
            [Declaration::Display(Display::Grid), Declaration::FlexGrow(1.0)]
        );
    }

    #[test]
    fn the_sides_of_a_property_are_set_independently() {
        let mut inline_style = InlineStyle::default();

        inline_style.set(Declaration::Margin(Side::Top, LengthPercentageAuto::Auto));
        inline_style.set(Declaration::Margin(Side::Left, LengthPercentageAuto::Auto));
        inline_style.set(Declaration::Inset(Side::Top, LengthPercentageAuto::Auto));

        assert_eq!(inline_style.declarations().len(), 3);
    }

    #[test]
    fn every_change_gets_a_new_revision() {
        let mut inline_style = InlineStyle::default();
        let initial = inline_style.revision();

        inline_style.set(Declaration::FlexGrow(1.0));
        let after_set = inline_style.revision();
        inline_style.clear();

        assert_ne!(initial, after_set);
        assert_ne!(after_set, inline_style.revision());
        assert!(inline_style.declarations().is_empty());
    }
}
