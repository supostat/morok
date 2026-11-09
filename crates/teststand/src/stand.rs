use std::path::{Path, PathBuf};

const CHROME_BUILD_FILE: &str = "chrome.toml";
const THRESHOLDS_FILE: &str = "thresholds.toml";
const ETALONS_DIRECTORY: &str = "etalons";
const DIFFS_DIRECTORY: &str = "target/teststand";

/// Where a stand keeps its files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stand {
    /// Names the Chrome for Testing build that renders the etalons.
    pub chrome_build_file: PathBuf,
    pub thresholds_file: PathBuf,
    /// Holds a directory per suite with a directory per case.
    pub etalons_directory: PathBuf,
    /// Receives the diff images of failing cases, a directory per suite.
    pub diffs_directory: PathBuf,
}

impl Stand {
    /// The stand of this workspace: its files are committed next to the crate and its diff
    /// images go under the target directory.
    pub fn of_workspace() -> Self {
        let crate_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = crate_directory
            .ancestors()
            .nth(2)
            .expect("the stand lives two levels below the workspace root");
        Self {
            chrome_build_file: crate_directory.join(CHROME_BUILD_FILE),
            thresholds_file: crate_directory.join(THRESHOLDS_FILE),
            etalons_directory: crate_directory.join(ETALONS_DIRECTORY),
            diffs_directory: workspace_root.join(DIFFS_DIRECTORY),
        }
    }
}
