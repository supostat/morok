use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

const FILE_NAME: &str = "Cargo.toml";

/// The manifest of a package or of a workspace, as its directory holds it.
#[derive(Debug)]
pub(crate) struct CargoManifest {
    path: PathBuf,
    table: Table,
}

impl CargoManifest {
    /// Reads the manifest of the package or the workspace at `directory`.
    pub(crate) fn of(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE_NAME);
        let text = fs::read_to_string(&path)
            .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", path.display())))?;
        match text.parse::<Table>() {
            Ok(table) => Ok(Self { path, table }),
            Err(error) => Err(unusable(&path, &error.to_string())),
        }
    }

    /// The value under the keys, each a key of the table the one before it names; none when a
    /// key is not there.
    pub(crate) fn value_at(&self, keys: &[&str]) -> Option<&Value> {
        let (first, nested) = keys.split_first()?;
        nested
            .iter()
            .try_fold(self.table.get(*first)?, |value, key| value.get(*key))
    }

    /// The error of a manifest that reads and still cannot be used.
    pub(crate) fn unusable(&self, problem: &str) -> io::Error {
        unusable(&self.path, problem)
    }
}

fn unusable(manifest: &Path, problem: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{}: {problem}", manifest.display()))
}
