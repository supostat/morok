use std::path::{Component, Path, PathBuf};

use morok_style::ColorScheme;
use serde::Deserialize;

use crate::case::Case;
use crate::case_rendering::COLOR_SCHEMES;
use crate::listed_frame::ListedFrame;
use crate::stand_error::StandError;
use crate::suite::Suite;
use crate::suites;
use crate::toml_file;

/// The frames of the kit a file asks the stand to write. Every frame of it is resolved: its
/// suite and its case are ones the stand has, and its file lies under the directory of the
/// list.
#[derive(Clone, Debug)]
pub(crate) struct FrameList {
    /// The suites the frames belong to, each once, in the order the list first names them.
    pub(crate) suites: Vec<Suite>,
    pub(crate) frames: Vec<ListedFrame>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredList {
    frame: Vec<StoredFrame>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredFrame {
    suite: String,
    case: String,
    color_scheme: String,
    file: PathBuf,
}

impl FrameList {
    /// Reads the list at `list_file`. A list with one frame the stand cannot serve is refused
    /// whole, so that nothing is drawn of a list that is wrong.
    pub(crate) fn read(list_file: &Path) -> Result<Self, StandError> {
        let stored: StoredList = toml_file::read(list_file)?;
        let directory = list_file.parent().expect("a file that was read lies in a directory");
        let mut list = Self {
            suites: Vec::new(),
            frames: Vec::new(),
        };
        for stored_frame in &stored.frame {
            list.add(stored_frame, directory)
                .map_err(|reason| StandError::Malformed {
                    path: list_file.to_path_buf(),
                    reason,
                })?;
        }
        Ok(list)
    }

    /// The frames of the list that belong to the suite, in the order of the list.
    pub(crate) fn frames_of<'list>(&'list self, suite: &'list Suite) -> impl Iterator<Item = &'list ListedFrame> {
        self.frames.iter().filter(|frame| frame.suite_name == suite.name)
    }

    fn add(&mut self, stored: &StoredFrame, directory: &Path) -> Result<(), String> {
        let suite = suites::named(&stored.suite).ok_or_else(|| {
            format!(
                "no suite is named `{}`; the suites are: {}",
                stored.suite,
                suites::NAMES.join(", ")
            )
        })?;
        let frame = ListedFrame {
            suite_name: suite.name,
            case: case_named(&suite, &stored.case)?,
            color_scheme: color_scheme_named(&stored.color_scheme)?,
            path: path_under(directory, &stored.file)?,
        };
        if self.suites.iter().all(|listed| listed.name != suite.name) {
            self.suites.push(suite);
        }
        self.frames.push(frame);
        Ok(())
    }
}

fn case_named(suite: &Suite, name: &str) -> Result<Case, String> {
    suite
        .cases
        .iter()
        .find(|case| case.name == name)
        .copied()
        .ok_or_else(|| format!("the suite `{}` has no case `{name}`", suite.name))
}

fn color_scheme_named(name: &str) -> Result<ColorScheme, String> {
    COLOR_SCHEMES
        .into_iter()
        .find(|color_scheme| color_scheme.to_string() == name)
        .ok_or_else(|| {
            let color_schemes = COLOR_SCHEMES.map(|color_scheme| format!("`{color_scheme}`"));
            format!(
                "`{name}` is no color scheme: a frame is drawn under {}",
                color_schemes.join(" or ")
            )
        })
}

/// The file of a frame as a path from where the stand runs. A file that is absolute, or that
/// climbs out of the directory of its list, is refused: a list writes under its own directory
/// alone.
fn path_under(directory: &Path, file: &Path) -> Result<PathBuf, String> {
    if names_a_file_under_its_directory(file) {
        Ok(directory.join(file))
    } else {
        Err(format!(
            "the file `{}` is not a path under the directory of the list",
            file.display()
        ))
    }
}

fn names_a_file_under_its_directory(file: &Path) -> bool {
    let mut components = file.components().peekable();
    components.peek().is_some() && components.all(|component| matches!(component, Component::Normal(_)))
}
