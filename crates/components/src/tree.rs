use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, AccessibleItem, ElementId, Focusability, Key, KeyEvent, KeyPhase,
    Modifiers, PointerEventKind, Propagation,
};
use morok_reactive::{Memo, Runtime, Signal};
use morok_style::{Declaration, LengthPercentage, Pixels, Side, States};

use crate::kit_icon::KitIcon;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property, Surfaces};
use crate::virtual_rows::VirtualRows;

/// How high a row of a tree is.
const ROW_HEIGHT: f32 = 28.0;
/// How far a row stands in from the edge, and how much further each level below the top.
const ROW_INSET: f32 = 8.0;
const LEVEL_INDENT: f32 = 16.0;

/// An item of a tree: a label, told apart from the other items by its key, with the items
/// under it.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem<ItemKey> {
    pub key: ItemKey,
    pub label: String,
    pub children: Vec<TreeItem<ItemKey>>,
}

/// Items in levels, each opened by its chevron to show the items under it, of which the user
/// selects one. The arrow keys move up and down through the items shown; the right arrow opens
/// an item, or goes into it when it is open; the left arrow closes an item, or goes out to the
/// item above it when it is closed. Only the rows the viewport shows have elements.
pub struct Tree<ItemKey> {
    roots: Rc<[TreeItem<ItemKey>]>,
    expanded: Signal<Vec<ItemKey>>,
    selected: Signal<Option<ItemKey>>,
    label: Property<String>,
}

impl<ItemKey: Clone + PartialEq + 'static> Tree<ItemKey> {
    /// A tree of `roots`, whose open items are those `expanded` holds the keys of and whose
    /// selected item is that `selected` holds the key of.
    pub fn new(
        roots: Vec<TreeItem<ItemKey>>,
        expanded: Signal<Vec<ItemKey>>,
        selected: Signal<Option<ItemKey>>,
    ) -> Self {
        Self {
            roots: roots.into(),
            expanded,
            selected,
            label: Property::Fixed(String::new()),
        }
    }

    /// What an assistive technology calls the tree.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the tree: a view whose content scrolls, with a row slot for each item in view.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            roots,
            expanded,
            selected,
            label,
        } = self;
        let runtime = mount.runtime();
        let shown = runtime.memo(move |runtime| {
            let mut shown = Vec::new();
            runtime.with(expanded, |expanded| shown_items(&roots, expanded, &mut shown));
            shown
        });
        let count = runtime.memo(move |runtime| runtime.with(shown, Vec::len));
        let selected_row = runtime.memo(move |runtime| {
            let selected = runtime.get(selected)?;
            runtime.with(shown, |shown| shown.iter().position(|item| item.key == selected))
        });
        let keyboard_row = runtime.signal(0);
        let tree_rows = TreeRows {
            shown,
            selected_row,
            expanded,
            selected,
            keyboard_row,
        };
        let rows = VirtualRows::mount(mount, count.into(), ROW_HEIGHT, move |mount, index| {
            tree_rows.mount_row(mount, index)
        });
        mount
            .build(rows.frame.viewport)
            .class(CLASSES.tree)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Tree)
            .accessible_name(label)
            .on_event(move |runtime, event| match event {
                ElementEvent::Key(key) => tree_key(runtime, key, shown, expanded, selected, keyboard_row),
                _ => Propagation::Continue,
            });
        rows.keep_in_view(mount, keyboard_row);
        rows.frame.view
    }
}

impl<ItemKey: Debug> Debug for Tree<ItemKey> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Tree")
            .field("roots", &self.roots)
            .field("expanded", &self.expanded)
            .field("selected", &self.selected)
            .field("label", &self.label)
            .finish()
    }
}

/// What the row slots of a mounted tree read and write.
struct TreeRows<ItemKey> {
    shown: Memo<Vec<ShownItem<ItemKey>>>,
    /// The row of the selected item among the items shown; none while it is not shown.
    selected_row: Memo<Option<usize>>,
    expanded: Signal<Vec<ItemKey>>,
    selected: Signal<Option<ItemKey>>,
    keyboard_row: Signal<usize>,
}

impl<ItemKey> Clone for TreeRows<ItemKey> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<ItemKey> Copy for TreeRows<ItemKey> {}

impl<ItemKey: Clone + PartialEq + 'static> TreeRows<ItemKey> {
    /// Builds what a row slot holds: the chevron and the label of the item the row of `index`
    /// shows, which a rebind writes into the row with nothing cloned.
    fn mount_row(self, mount: &mut Mount<'_>, index: Memo<usize>) -> ElementId {
        let Self {
            shown,
            selected_row,
            expanded,
            selected,
            keyboard_row,
        } = self;
        let runtime = mount.runtime();
        let is_selected = runtime.memo(move |runtime| runtime.get(selected_row) == Some(runtime.get(index)));
        let is_open = runtime.memo(move |runtime| read_item(runtime, shown, index, |item| item.is_open));
        let is_leaf = runtime.memo(move |runtime| read_item(runtime, shown, index, |item| !item.has_children));
        let indent = runtime.memo(move |runtime| {
            let level = read_item(runtime, shown, index, |item| item.level);
            Declaration::Padding(Side::Left, inset_of_level(level))
        });
        let accessible = runtime.memo(move |runtime| {
            read_item(runtime, shown, index, |item| AccessibleItem {
                level: Some(item.level + 1),
                expanded: item.has_children.then_some(item.is_open),
                ..AccessibleItem::default()
            })
        });
        let chevron = mount
            .kit_icon(KitIcon::Right)
            .class(CLASSES.tree_chevron)
            .class_when(CLASSES.expanded, is_open)
            .class_when(CLASSES.leaf, is_leaf)
            .on_event(move |runtime, event| match event {
                ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                    let toggle = read_item(runtime, shown, index, |item| {
                        item.has_children.then(|| (item.key.clone(), !item.is_open))
                    });
                    if let Some((key, opens)) = toggle {
                        open(runtime, expanded, &key, opens);
                    }
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            })
            .id();
        let label_text = mount.text("").class(CLASSES.tree_label_text).id();
        let label_room = mount.element().class(CLASSES.tree_label).child(label_text).id();
        let row = mount
            .element()
            .class(CLASSES.tree_row)
            .inline(indent)
            .state(States::CHECKED, is_selected)
            .accessible_role(AccessibleRole::TreeItem)
            .accessible_item(accessible)
            .accessible_actions(AccessibleActions::CLICK)
            .on_event(move |runtime, event| {
                let clicks = match event {
                    ElementEvent::Pointer(pointer) => matches!(pointer.kind, PointerEventKind::Clicked { .. }),
                    ElementEvent::Accessibility(action) => matches!(action, AccessibilityAction::Click),
                    ElementEvent::Key(_)
                    | ElementEvent::TextChanged(_)
                    | ElementEvent::Drag(_)
                    | ElementEvent::DragEnded(_) => false,
                };
                if !clicks {
                    return Propagation::Continue;
                }
                let key = read_item(runtime, shown, index, |item| item.key.clone());
                runtime.set(selected, Some(key));
                let index = runtime.get(index);
                runtime.set(keyboard_row, index);
                Propagation::Stop
            })
            .child(chevron)
            .child(label_room)
            .id();
        mount.effect(move |runtime, tree| {
            read_item(runtime, shown, index, |item| {
                tree.set_text(label_text, &item.label);
                tree.set_accessible_name(row, &item.label);
            });
        });
        row
    }
}

/// An item of a tree as a row shows it.
#[derive(Clone, Debug, PartialEq)]
struct ShownItem<ItemKey> {
    key: ItemKey,
    label: String,
    /// Zero for the top level.
    level: usize,
    has_children: bool,
    is_open: bool,
    /// The row of the item the item lies under; none at the top level.
    parent_row: Option<usize>,
}

/// Reads the item the row of `index` shows now.
fn read_item<ItemKey: 'static, Value>(
    runtime: &mut Runtime<Surfaces>,
    shown: Memo<Vec<ShownItem<ItemKey>>>,
    index: Memo<usize>,
    read: impl FnOnce(&ShownItem<ItemKey>) -> Value,
) -> Value {
    let index = runtime.get(index);
    runtime.with(shown, |shown| read(&shown[index]))
}

/// Adds the items shown of `items` and of the open items under them, in the order of the rows.
fn shown_items<ItemKey: Clone + PartialEq>(
    items: &[TreeItem<ItemKey>],
    expanded: &[ItemKey],
    shown: &mut Vec<ShownItem<ItemKey>>,
) {
    let mut pending: Vec<(&TreeItem<ItemKey>, usize, Option<usize>)> =
        items.iter().rev().map(|item| (item, 0, None)).collect();
    while let Some((item, level, parent_row)) = pending.pop() {
        let is_open = expanded.contains(&item.key);
        let row = shown.len();
        shown.push(ShownItem {
            key: item.key.clone(),
            label: item.label.clone(),
            level,
            has_children: !item.children.is_empty(),
            is_open,
            parent_row,
        });
        if is_open {
            pending.extend(item.children.iter().rev().map(|child| (child, level + 1, Some(row))));
        }
    }
}

fn open<ItemKey: Clone + PartialEq + 'static>(
    runtime: &mut Runtime<Surfaces>,
    expanded: Signal<Vec<ItemKey>>,
    key: &ItemKey,
    opens: bool,
) {
    runtime.update(expanded, |expanded| {
        expanded.retain(|open| open != key);
        if opens {
            expanded.push(key.clone());
        }
    });
}

/// What a key going down does in a tree with focus.
fn tree_key<ItemKey: Clone + PartialEq + 'static>(
    runtime: &mut Runtime<Surfaces>,
    key: &KeyEvent,
    shown: Memo<Vec<ShownItem<ItemKey>>>,
    expanded: Signal<Vec<ItemKey>>,
    selected: Signal<Option<ItemKey>>,
    keyboard_row: Signal<usize>,
) -> Propagation {
    if key.phase != KeyPhase::Pressed || key.modifiers != Modifiers::NONE {
        return Propagation::Continue;
    }
    let count = runtime.with(shown, Vec::len);
    if count == 0 {
        return Propagation::Continue;
    }
    let now = runtime.get(keyboard_row).min(count - 1);
    let item = runtime.with(shown, |shown| shown[now].clone());
    let moved_to = match key.key {
        Key::ArrowDown => (now + 1).min(count - 1),
        Key::ArrowUp => now.saturating_sub(1),
        Key::Home => 0,
        Key::End => count - 1,
        Key::ArrowRight if item.has_children && !item.is_open => {
            open(runtime, expanded, &item.key, true);
            return Propagation::Stop;
        }
        Key::ArrowRight if item.has_children => (now + 1).min(count - 1),
        Key::ArrowLeft if item.is_open => {
            open(runtime, expanded, &item.key, false);
            return Propagation::Stop;
        }
        Key::ArrowLeft => item.parent_row.unwrap_or(now),
        Key::ArrowRight => now,
        _ => return Propagation::Continue,
    };
    let key = runtime.with(shown, |shown| shown[moved_to].key.clone());
    runtime.set(selected, Some(key));
    runtime.set(keyboard_row, moved_to);
    Propagation::Stop
}

/// How far a row of the level stands in from the edge.
fn inset_of_level(level: usize) -> LengthPercentage {
    #[expect(clippy::cast_precision_loss, reason = "lengths on a surface are single precision")]
    let levels = level as f32;
    LengthPercentage {
        pixels: Pixels(ROW_INSET + levels * LEVEL_INDENT),
        fraction: None,
        tokens: std::borrow::Cow::Borrowed(&[]),
    }
}
