use crate::stylesheet_source::StylesheetSource;

/// The stylesheets of the components, in the order the build compiles them: the order of
/// their paths. A host that compiles them first, in this order, and its own stylesheets after
/// them gets a style sheet in which every class of the kit has the ID the components know it by.
pub const STYLESHEETS: [StylesheetSource; 4] = [
    StylesheetSource {
        path: "crates/components/src/button.css",
        text: include_str!("button.css"),
    },
    StylesheetSource {
        path: "crates/components/src/search_field.css",
        text: include_str!("search_field.css"),
    },
    StylesheetSource {
        path: "crates/components/src/text_area.css",
        text: include_str!("text_area.css"),
    },
    StylesheetSource {
        path: "crates/components/src/text_field.css",
        text: include_str!("text_field.css"),
    },
];
