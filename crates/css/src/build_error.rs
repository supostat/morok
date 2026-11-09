use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::diagnostics::Diagnostics;

#[derive(Debug)]
pub(crate) enum BuildError {
    Unreadable { path: PathBuf, error: io::Error },
    Rejected(Diagnostics),
    Unwritable { path: PathBuf, error: io::Error },
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, error } => write!(formatter, "morok-css: cannot read {}: {error}", path.display()),
            Self::Rejected(diagnostics) => write!(formatter, "{diagnostics}"),
            Self::Unwritable { path, error } => {
                write!(formatter, "morok-css: cannot write {}: {error}", path.display())
            }
        }
    }
}
