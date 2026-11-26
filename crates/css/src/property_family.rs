use crate::property::Property;
use crate::property_syntax::{background, border, effect, flex, grid, layout, motion, text};

/// The properties one module of the property syntax reads, under the title the reference of the
/// subset lists them by.
pub(crate) struct PropertyFamily {
    pub(crate) title: &'static str,
    pub(crate) properties: &'static [Property],
}

impl PropertyFamily {
    /// Every property of the subset, family by family.
    pub(crate) const ALL: &'static [Self] = &[
        Self {
            title: "Layout",
            properties: layout::PROPERTIES,
        },
        Self {
            title: "Flex and alignment",
            properties: flex::PROPERTIES,
        },
        Self {
            title: "Grid",
            properties: grid::PROPERTIES,
        },
        Self {
            title: "Border and outline",
            properties: border::PROPERTIES,
        },
        Self {
            title: "Paint",
            properties: background::PROPERTIES,
        },
        Self {
            title: "Text",
            properties: text::PROPERTIES,
        },
        Self {
            title: "Effects",
            properties: effect::PROPERTIES,
        },
        Self {
            title: "Motion",
            properties: motion::PROPERTIES,
        },
    ];
}
