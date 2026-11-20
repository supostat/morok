use morok::components::view::Mount;
use morok::components::{List, RowSelection, Tabs, Tree, TreeItem};
use morok::core::ElementId;

use crate::page_layout::{page, row, section};
use crate::styles::CLASSES;

/// The rows of the list.
const LIST_ROWS: usize = 10_000;

/// A tab of the strip of tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Panel {
    General,
    Sharing,
    History,
}

/// `List`, `Tree` and `Tabs`: a list of ten thousand rows with one and with several selected,
/// a tree with folders open, and tabs that switch the panel under them.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let one_selected = mount.runtime().signal(vec![2]);
    let several_selected = mount.runtime().signal(vec![1, 2, 4]);
    let expanded = mount.runtime().signal(vec!["projects", "morok"]);
    let tree_selected = mount.runtime().signal(Some("crates"));
    let panel = mount.runtime().signal(Panel::General);

    let single = List::new(LIST_ROWS, |row| format!("Document {}", row + 1), one_selected)
        .label("Documents")
        .mount(mount);
    let multiple = List::new(LIST_ROWS, |row| format!("Photo {}", row + 1), several_selected)
        .selection(RowSelection::Multiple)
        .label("Photos")
        .mount(mount);
    let tree = Tree::new(folders(), expanded, tree_selected)
        .label("Projects")
        .mount(mount);
    let lists = row(mount, &[single, multiple, tree]);
    let tabs = Tabs::new(panel)
        .tab(Panel::General, "General")
        .tab(Panel::Sharing, "Sharing")
        .tab(Panel::History, "History")
        .label("Inspector")
        .mount(mount);
    let panels = [
        (Panel::General, "Kind: folder · 12 items · 6.4 MB"),
        (Panel::Sharing, "Shared with nobody"),
        (Panel::History, "Created on Nov 3, modified today"),
    ]
    .map(|(shown, text)| (mount.runtime().memo(move |runtime| runtime.get(panel) == shown), text));
    let mut tabbed = mount.element();
    tabbed.class(CLASSES.page_tabbed).child(tabs);
    for (is_shown, text) in panels {
        tabbed.show(is_shown, move |mount| {
            mount.text(text).class(CLASSES.page_tab_panel).id()
        });
    }
    let tabbed = tabbed.id();
    let lists_section = section(mount, "LIST AND TREE", lists);
    let tabs_section = section(mount, "TABS", tabbed);
    page(mount, "Collections", &[lists_section, tabs_section])
}

fn folder(key: &'static str, label: &str, children: Vec<TreeItem<&'static str>>) -> TreeItem<&'static str> {
    TreeItem {
        key,
        label: label.to_owned(),
        children,
    }
}

fn folders() -> Vec<TreeItem<&'static str>> {
    vec![
        folder(
            "projects",
            "Projects",
            vec![
                folder(
                    "morok",
                    "morok",
                    vec![
                        folder("crates", "crates", Vec::new()),
                        folder("design", "design", Vec::new()),
                        folder("docs", "docs", Vec::new()),
                    ],
                ),
                folder("domovoi", "domovoi", vec![folder("src", "src", Vec::new())]),
            ],
        ),
        folder("archive", "Archive", vec![folder("backups", "Backups", Vec::new())]),
    ]
}
