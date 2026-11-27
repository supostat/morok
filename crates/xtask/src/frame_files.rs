use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use crate::finding::Finding;
use crate::line_number;

const FRAMES: &str = "frame";
const FILE: &str = "file";

const MISSING: &str = "the list of the frames of the page is missing";
const NOT_A_LIST: &str = "the file does not read as a list of frames";
const NO_FRAMES: &str = "the list names no frames: its frames are the tables of the array `frame`";
const NO_FILE: &str = "the frame names no `file`";
const UNWRITTEN: &str = "is not written: `cargo run -p morok-teststand -- frames site/frames.toml` writes the frames";

/// The files the list of frames of a site names, each a path from the directory of the list
/// with the line that names it. The stand is what holds the rest of a frame to its shape.
#[derive(Debug)]
pub(crate) struct FrameFiles {
    list_file: PathBuf,
    named: Vec<NamedFile>,
}

#[derive(Debug)]
struct NamedFile {
    file: PathBuf,
    line: usize,
}

/// Why a text is not a list of frames, and the line of it that says so.
struct Unreadable {
    line: usize,
    reason: String,
}

impl FrameFiles {
    /// Reads the list at `list_file`. A list that is not there, or that does not read as one,
    /// is a finding.
    pub(crate) fn read(list_file: &Path) -> io::Result<Result<Self, Finding>> {
        let list_text = match fs::read_to_string(list_file) {
            Ok(list_text) => list_text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(Err(finding(list_file, line_number::FIRST, MISSING.to_owned())));
            }
            Err(error) => return Err(error),
        };
        Ok(match named_files(&list_text) {
            Ok(named) => Ok(Self {
                list_file: list_file.to_path_buf(),
                named,
            }),
            Err(unreadable) => Err(finding(list_file, unreadable.line, unreadable.reason)),
        })
    }

    /// Whether the list names the file, a path from the directory of the list.
    pub(crate) fn names(&self, file: &Path) -> bool {
        self.named.iter().any(|named| named.file == file)
    }

    /// A finding for every file the list names that its directory does not hold.
    pub(crate) fn unwritten(&self) -> Vec<Finding> {
        let directory = self
            .list_file
            .parent()
            .expect("a list that was read lies in a directory");
        self.named
            .iter()
            .filter(|named| !directory.join(&named.file).is_file())
            .map(|named| {
                let reason = format!("the frame `{}` {UNWRITTEN}", named.file.display());
                finding(&self.list_file, named.line, reason)
            })
            .collect()
    }
}

fn named_files(list_text: &str) -> Result<Vec<NamedFile>, Unreadable> {
    let list = DeTable::parse(list_text).map_err(|error| Unreadable {
        line: error
            .span()
            .map_or(line_number::FIRST, |span| line_number::at(list_text, span.start)),
        reason: format!("{NOT_A_LIST}: {}", error.message()),
    })?;
    let frames = list
        .get_ref()
        .get(FRAMES)
        .and_then(|frames| frames.get_ref().as_array())
        .ok_or_else(|| Unreadable {
            line: line_number::FIRST,
            reason: NO_FRAMES.to_owned(),
        })?;
    frames.iter().map(|frame| named_file(list_text, frame)).collect()
}

fn named_file(list_text: &str, frame: &Spanned<DeValue<'_>>) -> Result<NamedFile, Unreadable> {
    if let Some(file) = frame.get_ref().get(FILE)
        && let Some(path) = file.get_ref().as_str()
    {
        Ok(NamedFile {
            file: PathBuf::from(path),
            line: line_number::at(list_text, file.span().start),
        })
    } else {
        Err(Unreadable {
            line: line_number::at(list_text, frame.span().start),
            reason: NO_FILE.to_owned(),
        })
    }
}

fn finding(list_file: &Path, line: usize, reason: String) -> Finding {
    Finding {
        path: list_file.to_path_buf(),
        line,
        reason,
    }
}
