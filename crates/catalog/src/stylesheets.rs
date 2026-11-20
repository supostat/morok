use morok::components::StylesheetSource;

/// The tokens file the catalog is built with: a host that styles trees of the catalog compiles
/// it over the tokens of the kit.
pub const TOKENS_STYLESHEET: StylesheetSource = StylesheetSource {
    path: "crates/catalog/tokens/base.css",
    text: include_str!("../tokens/base.css"),
};

/// The ember tokens file the catalog switches to.
pub const EMBER_TOKENS_STYLESHEET: StylesheetSource = StylesheetSource {
    path: "crates/catalog/tokens/ember.css",
    text: include_str!("../tokens/ember.css"),
};

/// The stylesheets of the catalog, in the order the build compiles them: the order of their
/// paths. A host that compiles the stylesheets of the kit first and these after them gets the
/// style sheet of the catalog, in which every class has the ID the catalog knows it by.
pub const STYLESHEETS: [StylesheetSource; 4] = [
    StylesheetSource {
        path: "crates/catalog/styles/catalog.css",
        text: include_str!("../styles/catalog.css"),
    },
    StylesheetSource {
        path: "crates/catalog/styles/disk_map.css",
        text: include_str!("../styles/disk_map.css"),
    },
    StylesheetSource {
        path: "crates/catalog/styles/file_manager.css",
        text: include_str!("../styles/file_manager.css"),
    },
    StylesheetSource {
        path: "crates/catalog/styles/pages.css",
        text: include_str!("../styles/pages.css"),
    },
];
