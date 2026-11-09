use std::fs;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::stand_error::StandError;

pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<T, StandError> {
    let text = fs::read_to_string(path).map_err(|error| StandError::File {
        path: path.to_path_buf(),
        error,
    })?;
    toml::from_str(&text).map_err(|error| StandError::Malformed {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

pub(crate) fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), StandError> {
    let text = toml::to_string(value).map_err(|error| StandError::Malformed {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    fs::write(path, text).map_err(|error| StandError::File {
        path: path.to_path_buf(),
        error,
    })
}
