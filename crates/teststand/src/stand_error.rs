use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::io;
use std::path::PathBuf;

use morok_css::Diagnostics;
use morok_platform::RendererError;

/// Why the stand could not do what it was asked; a case that fails its gates is not an error.
#[derive(Debug)]
pub enum StandError {
    /// A file of the stand could not be read or written.
    File { path: PathBuf, error: io::Error },
    /// A file of the stand does not hold what it should.
    Malformed { path: PathBuf, reason: String },
    /// The compiler of the kit rejected the tokens or a stylesheet of a suite.
    Rejected(Diagnostics),
    /// A stylesheet cannot be embedded in the harness page.
    Unembeddable { path: PathBuf },
    /// The pinned Chrome could not be fetched, launched or driven.
    Chrome(String),
    /// The kit could not create its renderer.
    Renderer(RendererError),
}

impl Display for StandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::File { path, error } => write!(formatter, "{}: {error}", path.display()),
            Self::Malformed { path, reason } => write!(formatter, "{}: {reason}", path.display()),
            Self::Rejected(diagnostics) => write!(formatter, "{diagnostics}"),
            Self::Unembeddable { path } => write!(
                formatter,
                "{}: a stylesheet that contains `</style` cannot be embedded in the harness page",
                path.display()
            ),
            Self::Chrome(reason) => write!(formatter, "Chrome: {reason}"),
            Self::Renderer(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for StandError {}
