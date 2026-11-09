use std::error::Error;
use std::fmt;

use crate::diagnostic::Diagnostic;

/// Every error found in one compilation, ordered by position in the source; never empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub(crate) fn new(mut diagnostics: Vec<Diagnostic>) -> Self {
        diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column));
        Self(diagnostics)
    }

    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.0
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, diagnostic) in self.0.iter().enumerate() {
            if position > 0 {
                write!(formatter, "\n\n")?;
            }
            write!(formatter, "{diagnostic}")?;
        }
        Ok(())
    }
}

impl Error for Diagnostics {}
