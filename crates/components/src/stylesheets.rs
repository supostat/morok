use crate::stylesheet_source::StylesheetSource;

/// The canonical tokens of the kit the build compiled the stylesheets against. An application
/// compiles its own tokens file over them: it defines every one of them.
pub const KIT_TOKENS: StylesheetSource = StylesheetSource {
    path: "design/system/tokens.css",
    text: include_str!("../../../design/system/tokens.css"),
};

/// The stylesheets of the components, in the order the build compiles them: the order of
/// their paths. A host that compiles them first, in this order, and its own stylesheets after
/// them gets a style sheet in which every class of the kit has the ID the components know it by.
pub const STYLESHEETS: [StylesheetSource; 32] = [
    StylesheetSource {
        path: "crates/components/src/button.css",
        text: include_str!("button.css"),
    },
    StylesheetSource {
        path: "crates/components/src/checkbox.css",
        text: include_str!("checkbox.css"),
    },
    StylesheetSource {
        path: "crates/components/src/combobox.css",
        text: include_str!("combobox.css"),
    },
    StylesheetSource {
        path: "crates/components/src/dialog.css",
        text: include_str!("dialog.css"),
    },
    StylesheetSource {
        path: "crates/components/src/icon_button.css",
        text: include_str!("icon_button.css"),
    },
    StylesheetSource {
        path: "crates/components/src/image.css",
        text: include_str!("image.css"),
    },
    StylesheetSource {
        path: "crates/components/src/list.css",
        text: include_str!("list.css"),
    },
    StylesheetSource {
        path: "crates/components/src/menu.css",
        text: include_str!("menu.css"),
    },
    StylesheetSource {
        path: "crates/components/src/option_list.css",
        text: include_str!("option_list.css"),
    },
    StylesheetSource {
        path: "crates/components/src/overlay_surface.css",
        text: include_str!("overlay_surface.css"),
    },
    StylesheetSource {
        path: "crates/components/src/popover.css",
        text: include_str!("popover.css"),
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
        path: "crates/components/src/select.css",
        text: include_str!("select.css"),
    },
    StylesheetSource {
        path: "crates/components/src/sheet.css",
        text: include_str!("sheet.css"),
    },
    StylesheetSource {
        path: "crates/components/src/sidebar.css",
        text: include_str!("sidebar.css"),
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
        path: "crates/components/src/split_view.css",
        text: include_str!("split_view.css"),
    },
    StylesheetSource {
        path: "crates/components/src/status_bar.css",
        text: include_str!("status_bar.css"),
    },
    StylesheetSource {
        path: "crates/components/src/switch.css",
        text: include_str!("switch.css"),
    },
    StylesheetSource {
        path: "crates/components/src/table.css",
        text: include_str!("table.css"),
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
        path: "crates/components/src/toast_stack.css",
        text: include_str!("toast_stack.css"),
    },
    StylesheetSource {
        path: "crates/components/src/toolbar.css",
        text: include_str!("toolbar.css"),
    },
    StylesheetSource {
        path: "crates/components/src/tooltip.css",
        text: include_str!("tooltip.css"),
    },
    StylesheetSource {
        path: "crates/components/src/tree.css",
        text: include_str!("tree.css"),
    },
];
