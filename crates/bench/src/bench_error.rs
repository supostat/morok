use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use morok_css::Diagnostics;
use morok_platform::RendererError;

/// Why the harness could not run a scenario; a scenario that exceeds a budget is not an error.
#[derive(Debug)]
pub enum BenchError {
    /// A file of the harness could not be read.
    File { path: PathBuf, error: io::Error },
    /// A file of the harness does not hold what it should.
    Malformed { path: PathBuf, reason: String },
    /// The compiler of the kit rejected the tokens or the stylesheet of a scene.
    Rejected(Diagnostics),
    /// The kit could not create its renderer.
    Renderer(RendererError),
    /// A window opened on a scene presented nothing in the time given to it.
    NoFirstFrame { waited: Duration },
    /// The process whose start is measured could not be started or followed.
    NoLaunch { executable: PathBuf, error: io::Error },
    /// The window drew no glyph of a keypress in the time given to it.
    NoGlyph { waited: Duration },
    /// The window ran no frame in the time given to it, though one was asked for.
    NoFrame { waited: Duration },
}

impl Display for BenchError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::File { path, error } => write!(formatter, "{}: {error}", path.display()),
            Self::Malformed { path, reason } => write!(formatter, "{}: {reason}", path.display()),
            Self::Rejected(diagnostics) => write!(formatter, "{diagnostics}"),
            Self::Renderer(error) => write!(formatter, "{error}"),
            Self::NoFirstFrame { waited } => write!(
                formatter,
                "the window presented no frame within {:.1} s of opening",
                waited.as_secs_f64()
            ),
            Self::NoLaunch { executable, error } => {
                write!(formatter, "could not launch {}: {error}", executable.display())
            }
            Self::NoGlyph { waited } => write!(
                formatter,
                "the window drew no glyph of a keypress within {:.1} s of it",
                waited.as_secs_f64()
            ),
            Self::NoFrame { waited } => write!(
                formatter,
                "the window ran no frame within {:.1} s of asking for one",
                waited.as_secs_f64()
            ),
        }
    }
}

impl Error for BenchError {}

impl From<RendererError> for BenchError {
    fn from(error: RendererError) -> Self {
        Self::Renderer(error)
    }
}
