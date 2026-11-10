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
    pub fn class_named(&self, name: &str) -> Option<Class> {
        let index = self.classes.iter().position(|class| class.name == name)?;
        Some(Class::new(index as u32))
    }

    pub fn class_style(&self, class: Class) -> &ClassStyle {
        &self.classes[class.index()]
    }

    pub fn keyframes_rule(&self, keyframes: Keyframes) -> &KeyframesRule {
        &self.keyframes[keyframes.index()]
    }
}
