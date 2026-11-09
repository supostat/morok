use morok_style::{Declaration, StyleSheet};

use crate::style::inline_style::InlineStyle;

/// Where the winning declaration of a property is stored.
///
/// A computed style holds its variable-length values — track lists, gradients — by their source
/// and reads them in place, so resolving a style copies no list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclarationSource {
    Rule {
        rule: u32,
        declaration: u32,
    },
    /// The revision tells an inline declaration from the one it replaced at the same position.
    Inline {
        declaration: u32,
        revision: u32,
    },
}

impl DeclarationSource {
    pub(crate) fn declaration<'a>(self, style_sheet: &'a StyleSheet, inline_style: &'a InlineStyle) -> &'a Declaration {
        match self {
            Self::Rule { rule, declaration } => &style_sheet.rules[rule as usize].declarations[declaration as usize],
            Self::Inline { declaration, .. } => &inline_style.declarations()[declaration as usize],
        }
    }
}
