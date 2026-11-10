use cssparser::Parser;
use morok_style::Keyframes;

use crate::keyframes_syntax;
use crate::rule_prelude::{self, RuleEnd};
use crate::source_file::SourceFile;

/// The names of the `@keyframes` rules of stylesheets compiled together, in the order they are
/// first declared; the order gives each rule its ID. A rule may be named before it is declared.
#[derive(Debug, Default)]
pub(crate) struct KeyframesTable {
    names: Vec<String>,
}

impl KeyframesTable {
    /// Collects the names of the well-formed `@keyframes` rules of the sources; reading the
    /// sources reports the others.
    pub(crate) fn of(sources: &[SourceFile]) -> Self {
        let mut table = Self::default();
        for source in sources {
            let text = source.text();
            let mut input = Parser::new(text);
            loop {
                let (prelude, rule_end) = rule_prelude::read(&mut input, text);
                if let Some(Ok(name)) = keyframes_syntax::name(text, prelude)
                    && table.id(&name.text).is_none()
                {
                    table.names.push(name.text);
                }
                if rule_end == RuleEnd::EndOfInput {
                    break;
                }
            }
        }
        table
    }

    pub(crate) fn id(&self, name: &str) -> Option<Keyframes> {
        let index = self.names.iter().position(|declared| declared == name)?;
        Some(Keyframes::new(
            u32::try_from(index).expect("stylesheets declare fewer than 2^32 keyframes rules"),
        ))
    }

    pub(crate) fn names(&self) -> &[String] {
        &self.names
    }
}
