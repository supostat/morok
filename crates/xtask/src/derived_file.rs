use std::fs;
use std::io;
use std::path::PathBuf;

use crate::finding::Finding;
use crate::line_number;

const MISSING: &str = "the file is missing: `cargo xtask site-tokens .` writes it";
const BEHIND: &str = "the file is not what the tokens of the kit render: `cargo xtask site-tokens .` writes it again";

/// A file derived from the tokens of the kit: where it lies and the text it must hold.
#[derive(Debug)]
pub(crate) struct DerivedFile {
    pub(crate) path: PathBuf,
    pub(crate) text: String,
}

impl DerivedFile {
    /// Writes the file, creating the directories above it.
    pub(crate) fn write(&self) -> io::Result<()> {
        let directory = self
            .path
            .parent()
            .expect("a derived file lies in a directory of the workspace");
        fs::create_dir_all(directory)?;
        fs::write(&self.path, &self.text)
    }

    /// The finding when the file on disk is missing or holds another text; none when it is
    /// what it must be.
    pub(crate) fn finding(&self) -> io::Result<Option<Finding>> {
        match fs::read_to_string(&self.path) {
            Ok(found) if found == self.text => Ok(None),
            Ok(found) => Ok(Some(self.finding_at(self.first_differing_line(&found), BEHIND))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Ok(Some(self.finding_at(line_number::FIRST, MISSING)))
            }
            Err(error) => Err(error),
        }
    }

    fn first_differing_line(&self, found: &str) -> usize {
        let same_lines = found
            .lines()
            .zip(self.text.lines())
            .take_while(|(found_line, line)| found_line == line)
            .count();
        same_lines + line_number::FIRST
    }

    fn finding_at(&self, line: usize, reason: &str) -> Finding {
        Finding {
            path: self.path.clone(),
            line,
            reason: reason.to_owned(),
        }
    }
}
