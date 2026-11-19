use std::path::PathBuf;

use crate::transfer::custom_data::CustomData;
use crate::transfer::payload_type::PayloadType;

/// What a clipboard holds or a drag carries: one content in every type its source gives it in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Payload {
    pub text: Option<String>,
    /// The paths of files and folders.
    pub files: Vec<PathBuf>,
    /// Addresses that name no file, each as its text.
    pub urls: Vec<String>,
    /// Data of types of the application's own, one entry to a type.
    pub custom: Vec<CustomData>,
}

impl Payload {
    /// A payload of plain text alone.
    pub fn of_text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }

    /// Whether the payload holds its content in the type.
    pub fn holds(&self, payload_type: &PayloadType) -> bool {
        match payload_type {
            PayloadType::Text => self.text.is_some(),
            PayloadType::Files => !self.files.is_empty(),
            PayloadType::Urls => !self.urls.is_empty(),
            PayloadType::Custom(type_name) => self.custom(type_name).is_some(),
        }
    }

    /// The data the payload holds of the application's own type of that name.
    pub fn custom(&self, type_name: &str) -> Option<&[u8]> {
        self.custom
            .iter()
            .find(|data| data.type_name == type_name)
            .map(|data| data.bytes.as_slice())
    }
}
