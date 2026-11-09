use std::path::Path;

use serde::Deserialize;

use crate::stand_error::StandError;
use crate::toml_file;

/// The Chrome for Testing build every etalon of a stand is rendered by.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChromeBuild {
    /// The full version of the build, such as `157.0.8103.38`.
    pub version: String,
}

impl ChromeBuild {
    pub fn read(path: &Path) -> Result<Self, StandError> {
        toml_file::read(path)
    }
}
