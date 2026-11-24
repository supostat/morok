use morok::core::{CustomData, Payload};

/// A picture of the catalog's own. The windows of the catalog show it by the one image the
/// application added for it, and a drag carries it from one window to another by its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GalleryPicture {
    /// A landscape PNG of 160 by 100 pixels.
    Meadow,
    /// A portrait JPEG of 100 by 150 pixels.
    Dunes,
    /// Bytes no decoder reads.
    Unreadable,
}

impl GalleryPicture {
    pub const ALL: [Self; 3] = [Self::Meadow, Self::Dunes, Self::Unreadable];
    /// The type of the catalog's own a picture is dragged in: its name.
    pub const DRAGGED_TYPE: &str = "org.morok.catalog.picture";

    /// The name an assistive technology reads for the picture.
    pub fn label(self) -> &'static str {
        match self {
            Self::Meadow => "Meadow",
            Self::Dunes => "Dunes",
            Self::Unreadable => "Unreadable picture",
        }
    }

    /// What a drag of the picture carries.
    pub(crate) fn payload(self) -> Payload {
        Payload {
            custom: vec![CustomData {
                type_name: String::from(Self::DRAGGED_TYPE),
                bytes: self.label().as_bytes().to_vec(),
            }],
            ..Payload::default()
        }
    }

    /// The picture a drag carries, when it carries one of the catalog's.
    pub(crate) fn carried_by(payload: &Payload) -> Option<Self> {
        let name = payload.custom(Self::DRAGGED_TYPE)?;
        Self::ALL.into_iter().find(|picture| picture.label().as_bytes() == name)
    }
}
