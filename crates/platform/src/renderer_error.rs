use std::error::Error;
use std::fmt::{Display, Formatter, Result};

/// Why the shared GPU resources could not be created.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererError {
    /// The system has no Metal device.
    NoDevice,
    /// The device does not run Metal 4.
    Metal4Unsupported,
    /// Metal refused to create an object the renderer cannot work without.
    Metal { object: &'static str, reason: String },
    /// The SVG of an icon does not parse.
    UnreadableIcon(String),
}

impl Display for RendererError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::NoDevice => formatter.write_str("this Mac has no Metal device"),
            Self::Metal4Unsupported => formatter.write_str("the Metal device does not support Metal 4"),
            Self::Metal { object, reason } => write!(formatter, "Metal created no {object}: {reason}"),
            Self::UnreadableIcon(reason) => write!(formatter, "the icon is not an SVG the kit reads: {reason}"),
        }
    }
}

impl Error for RendererError {}
