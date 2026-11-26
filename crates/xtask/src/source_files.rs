use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::source_language::SourceLanguage;

/// The sources under `directory` in a language the checks read, in the order of their paths.
pub(crate) fn under(directory: &Path) -> io::Result<Vec<(PathBuf, SourceLanguage)>> {
    let mut source_files = Vec::new();
    let mut unvisited_directories = vec![directory.to_path_buf()];
    while let Some(unvisited) = unvisited_directories.pop() {
        for entry in fs::read_dir(unvisited)? {
            let path = entry?.path();
            if path.is_dir() {
                unvisited_directories.push(path);
            } else if let Some(language) = SourceLanguage::of(&path) {
                source_files.push((path, language));
            }
        }
    }
    source_files.sort_by(|(left, _), (right, _)| left.cmp(right));
    Ok(source_files)
}
