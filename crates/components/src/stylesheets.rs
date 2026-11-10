use crate::stylesheet_source::StylesheetSource;

/// The stylesheets of the components, in the order the build compiles them: the order of
/// their paths. A host that compiles them first, in this order, and its own stylesheets after
/// them gets a style sheet in which every class of the kit has the ID the components know it by.
pub const STYLESHEETS: [StylesheetSource; 1] = [StylesheetSource {
    path: "crates/components/src/button.css",
    text: include_str!("button.css"),
}];
