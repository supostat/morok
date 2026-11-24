use morok::core::{ElementId, ElementTree};
use morok_catalog::styles::CLASSES;
use morok_catalog::{LIST_ROWS, Page, TABLE_ROWS};

/// A collection of rows on a page of the catalog that a scenario scrolls from its first row to
/// its last. Every row of either is named "Document" and its number, from 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrolledCollection {
    /// The table of a million rows of the page of the table.
    MillionRowTable,
    /// The list "Documents" of the page of collections.
    DocumentList,
}

/// The elements of a collection a scroll goes through: the one whose content scrolls, and the
/// one that holds the row slots, each slot holding its row.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CollectionParts {
    pub(crate) viewport: ElementId,
    pub(crate) slots: ElementId,
}

impl ScrolledCollection {
    pub(crate) fn page(self) -> Page {
        match self {
            Self::MillionRowTable => Page::Table,
            Self::DocumentList => Page::Collections,
        }
    }

    pub(crate) fn row_count(self) -> usize {
        match self {
            Self::MillionRowTable => TABLE_ROWS,
            Self::DocumentList => LIST_ROWS,
        }
    }

    /// How high the kit makes a row of the collection.
    pub(crate) fn row_height(self) -> f32 {
        match self {
            Self::MillionRowTable => 26.0,
            Self::DocumentList => 28.0,
        }
    }

    pub(crate) fn last_row_name(self) -> String {
        format!("Document {}", self.row_count())
    }

    /// The parts of the collection on the page its tree shows.
    pub(crate) fn parts(self, tree: &ElementTree) -> CollectionParts {
        match self {
            Self::MillionRowTable => {
                let table = elements_with_class(tree, CLASSES.table)
                    .next()
                    .expect("the page of the table shows a table");
                let viewport = tree.children(table)[0];
                let content = tree.children(viewport)[0];
                CollectionParts {
                    viewport,
                    slots: tree.children(content)[0],
                }
            }
            Self::DocumentList => elements_with_class(tree, CLASSES.list)
                .map(|viewport| CollectionParts {
                    viewport,
                    slots: tree.children(viewport)[0],
                })
                .find(|parts| row_name_in_slot(tree, tree.children(parts.slots)[0]) == "Document 1")
                .expect("the page of collections shows the list of documents"),
        }
    }
}

/// The name of the row a slot of the collection shows: the text of the first child of its row.
pub(crate) fn row_name_in_slot(tree: &ElementTree, slot: ElementId) -> &str {
    let row = tree.children(slot)[0];
    tree.text(tree.children(row)[0])
}

/// The elements of the tree that have the class.
fn elements_with_class(tree: &ElementTree, class: morok::style::Class) -> impl Iterator<Item = ElementId> + '_ {
    let mut unvisited = vec![tree.root()];
    std::iter::from_fn(move || {
        while let Some(element) = unvisited.pop() {
            unvisited.extend(tree.children(element));
            if tree.classes(element).contains(&class) {
                return Some(element);
            }
        }
        None
    })
}
