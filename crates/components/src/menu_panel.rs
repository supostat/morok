use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibleActions, ElementId, Focusability, OverlayAlignment, OverlayPlacement, OverlaySide, Propagation,
};
use morok_style::States;

use crate::kit_icon::KitIcon;
use crate::menu_closer::MenuCloser;
use crate::menu_item::MenuItem;
use crate::menu_item_kind::MenuItemKind;
use crate::menu_navigation::MenuNavigation;
use crate::overlay_presence::show_overlay_while;
use crate::shortcut_label::shortcut_label;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Overlay, OverlayAnchor, OverlayInteraction};

/// A submenu opens beside its line with its first line level with that line: the padding
/// and the border of a menu are 5 px, and the borders of the two menus share a pixel.
const SUBMENU_PLACEMENT: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Right,
    alignment: OverlayAlignment::Start,
    gap: 4.0,
    shift: -5.0,
};

/// What an open menu shows: a column of its lines. The line the pointer or the arrow keys are
/// on has focus; a line with a submenu opens it as an overlay beside itself.
pub(crate) struct MenuPanel {
    pub(crate) items: Rc<[MenuItem]>,
    /// What an assistive technology calls the menu.
    pub(crate) label: String,
    /// Closes this menu with every menu it was opened from.
    pub(crate) close_all: MenuCloser,
    /// Closes this menu alone when it is the submenu of a line.
    pub(crate) close_submenu: Option<MenuCloser>,
    /// Whether the first line is highlighted from the start, as in a submenu the keyboard opened.
    pub(crate) highlights_first: bool,
}

impl MenuPanel {
    pub(crate) fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            items,
            label,
            close_all,
            close_submenu,
            highlights_first,
        } = self;
        let runtime = mount.runtime();
        let navigation = MenuNavigation {
            items: Rc::clone(&items),
            highlighted: runtime.signal(None),
            open_submenu: runtime.signal(None),
            submenu_takes_highlight: runtime.signal(false),
            close_all,
            close_submenu,
        };
        if highlights_first {
            let first = navigation.first_choosable(runtime);
            runtime.set(navigation.highlighted, first);
        }

        let lines: Vec<ElementId> = (0..items.len())
            .map(|position| mount_line(mount, &navigation, position))
            .collect();
        let mut panel = mount.element();
        let keys = navigation.clone();
        panel
            .class(CLASSES.menu)
            .focusable(Focusability::Focusable)
            .accessible_role(AccessibleRole::Menu)
            .accessible_name(label.clone())
            .on_event(move |runtime, event| match event {
                ElementEvent::Key(key) => keys.key_event(runtime, key),
                _ => Propagation::Continue,
            });
        for line in &lines {
            panel.child(*line);
        }
        let panel = panel.id();

        let highlighted = navigation.highlighted;
        let focused_lines = lines.clone();
        mount.effect(move |runtime, tree| {
            let focused = runtime
                .get(highlighted)
                .map_or(panel, |position| focused_lines[position]);
            tree.focus(focused);
        });
        for (position, item) in items.iter().enumerate() {
            if let MenuItemKind::Submenu { items: submenu_items } = &item.kind {
                show_submenu(mount, &navigation, position, lines[position], &label, submenu_items);
            }
        }
        panel
    }
}

/// Builds the line at `position`: a separator, or a row of the icon, the label and the
/// shortcut of an item, or its chevron when it opens a submenu.
fn mount_line(mount: &mut Mount<'_>, navigation: &MenuNavigation, position: usize) -> ElementId {
    let item = &navigation.items[position];
    if matches!(item.kind, MenuItemKind::Separator) {
        return mount.element().class(CLASSES.menu_separator).id();
    }
    let icon = item
        .icon
        .map(|icon| mount.kit_icon(icon).class(CLASSES.menu_item_icon).id());
    let label = mount.text(item.label.clone()).id();
    let trailing = match (&item.kind, item.shortcut) {
        (MenuItemKind::Submenu { .. }, _) => Some(mount.kit_icon(KitIcon::Right).class(CLASSES.menu_item_chevron).id()),
        (_, Some(chord)) => Some(mount.text(shortcut_label(chord)).class(CLASSES.menu_item_shortcut).id()),
        (_, None) => None,
    };
    let events = navigation.clone();
    let mut line = mount.element();
    line.class(CLASSES.menu_item);
    if item.is_destructive {
        line.class(CLASSES.danger);
    }
    line.state(States::DISABLED, item.is_disabled.clone())
        .focusable(Focusability::Focusable)
        .accessible_role(AccessibleRole::MenuItem)
        .accessible_name(item.label.clone())
        .accessible_actions(AccessibleActions::CLICK)
        .on_event(move |runtime, event| events.item_event(runtime, position, event));
    for part in icon.into_iter().chain([label]).chain(trailing) {
        line.child(part);
    }
    line.id()
}

/// Keeps the submenu of the line at `position` open beside the line while the line holds it open.
fn show_submenu(
    mount: &mut Mount<'_>,
    navigation: &MenuNavigation,
    position: usize,
    line: ElementId,
    label: &str,
    items: &Rc<[MenuItem]>,
) {
    let open_submenu = navigation.open_submenu;
    let takes_highlight = navigation.submenu_takes_highlight;
    let is_open = mount
        .runtime()
        .memo(move |runtime| runtime.get(open_submenu) == Some(position));
    let close_submenu: MenuCloser = Rc::new(move |runtime| runtime.set(open_submenu, None));
    let dismissed = Rc::clone(&close_submenu);
    let (items, label, close_all) = (Rc::clone(items), label.to_owned(), Rc::clone(&navigation.close_all));
    show_overlay_while(
        mount,
        line,
        is_open,
        move |_| {
            let dismissed = Rc::clone(&dismissed);
            Overlay {
                anchor: OverlayAnchor::Element(line),
                placement: SUBMENU_PLACEMENT,
                interaction: OverlayInteraction::Interactive {
                    on_dismiss: Box::new(move |runtime| dismissed(runtime)),
                },
            }
        },
        move |mount, _| {
            let highlights_first = mount.runtime().get(takes_highlight);
            MenuPanel {
                items: Rc::clone(&items),
                label: label.clone(),
                close_all: Rc::clone(&close_all),
                close_submenu: Some(Rc::clone(&close_submenu)),
                highlights_first,
            }
            .mount(mount)
        },
    );
}
