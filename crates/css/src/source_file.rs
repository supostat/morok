use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The text of a stylesheet together with the path diagnostics name it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    path: PathBuf,
    text: String,
}

impl SourceFile {
    /// A stylesheet whose text is already at hand: no file is read, and `path` is only what
    /// diagnostics name it by.
    pub fn new(path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    /// The stylesheet read from the file at `path`.
    pub fn read(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let text = fs::read_to_string(&path)?;
        Ok(Self { path, text })
    }

    /// The path diagnostics name the stylesheet by.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The text of the stylesheet.
    pub fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn line_number(&self, offset: usize) -> usize {
        self.text[..offset].matches('\n').count() + 1
    }
}
