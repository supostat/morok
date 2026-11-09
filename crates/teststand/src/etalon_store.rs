use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use morok_platform::FrameImage;
use serde::{Deserialize, Serialize};

use crate::case_rendering::CaseRendering;
use crate::element_box::ElementBox;
use crate::etalon::Etalon;
use crate::frame_png;
use crate::rendering::Rendering;
use crate::stand_error::StandError;
use crate::toml_file;

const MANIFEST_FILE: &str = "manifest.toml";
const BOXES_FILE: &str = "boxes.toml";
const LIGHT_FRAME_FILE: &str = "light.png";
const DARK_FRAME_FILE: &str = "dark.png";

/// The etalons of a stand on disk: a directory per suite, and in it a directory per case with
/// the manifest, the element boxes and a frame per color scheme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtalonStore {
    directory: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredBoxes {
    light: Vec<ElementBox>,
    dark: Vec<ElementBox>,
}

impl EtalonStore {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// The etalon of the case, or none when the case has never been rendered.
    pub fn read(&self, suite_name: &str, case_name: &str) -> Result<Option<Etalon>, StandError> {
        let directory = self.case_directory(suite_name, case_name);
        if !directory.is_dir() {
            return Ok(None);
        }
        let boxes: StoredBoxes = toml_file::read(&directory.join(BOXES_FILE))?;
        Ok(Some(Etalon {
            manifest: toml_file::read(&directory.join(MANIFEST_FILE))?,
            rendering: CaseRendering {
                light: Rendering {
                    image: read_frame(&directory.join(LIGHT_FRAME_FILE))?,
                    boxes: boxes.light,
                },
                dark: Rendering {
                    image: read_frame(&directory.join(DARK_FRAME_FILE))?,
                    boxes: boxes.dark,
                },
            },
        }))
    }

    /// Stores the etalon of the case in place of the one it had.
    pub fn write(&self, suite_name: &str, case_name: &str, etalon: &Etalon) -> Result<(), StandError> {
        let directory = self.case_directory(suite_name, case_name);
        fs::create_dir_all(&directory).map_err(|error| file_error(&directory, error))?;
        toml_file::write(&directory.join(MANIFEST_FILE), &etalon.manifest)?;
        toml_file::write(
            &directory.join(BOXES_FILE),
            &StoredBoxes {
                light: etalon.rendering.light.boxes.clone(),
                dark: etalon.rendering.dark.boxes.clone(),
            },
        )?;
        write_frame(&directory.join(LIGHT_FRAME_FILE), &etalon.rendering.light.image)?;
        write_frame(&directory.join(DARK_FRAME_FILE), &etalon.rendering.dark.image)
    }

    /// The names of the cases of the suite that have an etalon, in order.
    pub fn case_names(&self, suite_name: &str) -> Result<Vec<String>, StandError> {
        let directory = self.suite_directory(suite_name);
        if !directory.is_dir() {
            return Ok(Vec::new());
        }
        let mut case_names = Vec::new();
        for entry in fs::read_dir(&directory).map_err(|error| file_error(&directory, error))? {
            let entry = entry.map_err(|error| file_error(&directory, error))?;
            if entry.path().is_dir() {
                case_names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        case_names.sort();
        Ok(case_names)
    }

    /// Drops the etalon of the case.
    pub fn remove(&self, suite_name: &str, case_name: &str) -> Result<(), StandError> {
        let directory = self.case_directory(suite_name, case_name);
        fs::remove_dir_all(&directory).map_err(|error| file_error(&directory, error))
    }

    fn suite_directory(&self, suite_name: &str) -> PathBuf {
        self.directory.join(suite_name)
    }

    fn case_directory(&self, suite_name: &str, case_name: &str) -> PathBuf {
        self.suite_directory(suite_name).join(case_name)
    }
}

fn read_frame(path: &Path) -> Result<FrameImage, StandError> {
    let png = fs::read(path).map_err(|error| file_error(path, error))?;
    frame_png::decode(&png).map_err(|reason| StandError::Malformed {
        path: path.to_path_buf(),
        reason,
    })
}

fn write_frame(path: &Path, image: &FrameImage) -> Result<(), StandError> {
    fs::write(path, frame_png::encode(image)).map_err(|error| file_error(path, error))
}

fn file_error(path: &Path, error: io::Error) -> StandError {
    StandError::File {
        path: path.to_path_buf(),
        error,
    }
}
