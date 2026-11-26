use std::borrow::Cow;

use crate::class::Class;
use crate::class_style::ClassStyle;
use crate::motion::keyframes::Keyframes;
use crate::motion::keyframes_rule::KeyframesRule;
use crate::style_rule::StyleRule;

/// Compiled stylesheets: every rule in cascade order and the classes they name.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleSheet {
    /// Ordered by specificity, then by source position: a later rule overrides an earlier one.
    pub rules: Cow<'static, [StyleRule]>,
    /// Indexed by [`Class`].
    pub classes: Cow<'static, [ClassStyle]>,
    /// Indexed by [`Keyframes`].
    pub keyframes: Cow<'static, [KeyframesRule]>,
}

impl StyleSheet {
    /// The class the stylesheets wrote as `.name`, by its name without the dot; none when no rule
    /// names it.
    pub fn class_named(&self, name: &str) -> Option<Class> {
        let index = self.classes.iter().position(|class| class.name == name)?;
        Some(Class::new(index as u32))
    }

    /// The rules of a class of this style sheet.
    ///
    /// # Panics
    ///
    /// When the style sheet has no class at the index of `class`, as with a class of another
    /// style sheet.
    pub fn class_style(&self, class: Class) -> &ClassStyle {
        &self.classes[class.index()]
    }

    /// The keyframes of a `@keyframes` rule of this style sheet.
    ///
    /// # Panics
    ///
    /// When the style sheet has no rule at the index of `keyframes`, as with a rule of another
    /// style sheet.
    pub fn keyframes_rule(&self, keyframes: Keyframes) -> &KeyframesRule {
        &self.keyframes[keyframes.index()]
    }
}
