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
    pub fn new(path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    pub fn read(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let text = fs::read_to_string(&path)?;
        Ok(Self { path, text })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn line_number(&self, offset: usize) -> usize {
        self.text[..offset].matches('\n').count() + 1
    }
}
