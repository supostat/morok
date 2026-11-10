use crate::stylesheet_source::StylesheetSource;

/// The stylesheets of the components, in the order the build compiles them: the order of
/// their paths. A host that compiles them first, in this order, and its own stylesheets after
/// them gets a style sheet in which every class of the kit has the ID the components know it by.
pub const STYLESHEETS: [StylesheetSource; 16] = [
    StylesheetSource {
        path: "crates/components/src/button.css",
        text: include_str!("button.css"),
    },
    StylesheetSource {
        path: "crates/components/src/checkbox.css",
        text: include_str!("checkbox.css"),
    },
    StylesheetSource {
        path: "crates/components/src/icon_button.css",
        text: include_str!("icon_button.css"),
    },
    StylesheetSource {
        path: "crates/components/src/list.css",
        text: include_str!("list.css"),
    },
    StylesheetSource {
        path: "crates/components/src/progress_bar.css",
        text: include_str!("progress_bar.css"),
    },
    StylesheetSource {
        path: "crates/components/src/radio_group.css",
        text: include_str!("radio_group.css"),
    },
    StylesheetSource {
        path: "crates/components/src/scroll_view.css",
        text: include_str!("scroll_view.css"),
    },
    StylesheetSource {
        path: "crates/components/src/search_field.css",
        text: include_str!("search_field.css"),
    },
    StylesheetSource {
        path: "crates/components/src/segmented_control.css",
        text: include_str!("segmented_control.css"),
    },
    StylesheetSource {
        path: "crates/components/src/slider.css",
        text: include_str!("slider.css"),
    },
    StylesheetSource {
        path: "crates/components/src/spinner.css",
        text: include_str!("spinner.css"),
    },
    StylesheetSource {
        path: "crates/components/src/switch.css",
        text: include_str!("switch.css"),
    },
    StylesheetSource {
        path: "crates/components/src/tabs.css",
        text: include_str!("tabs.css"),
    },
    StylesheetSource {
        path: "crates/components/src/text_area.css",
        text: include_str!("text_area.css"),
    },
    StylesheetSource {
        path: "crates/components/src/text_field.css",
        text: include_str!("text_field.css"),
    },
    StylesheetSource {
        path: "crates/components/src/tree.css",
        text: include_str!("tree.css"),
    },
];
