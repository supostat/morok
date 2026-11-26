use std::rc::Rc;

use morok_core::{
    ElementId, NativeMenu, NativeMenuItem, OverlayAlignment, OverlayPlacement, OverlaySide, Point, PointerButton,
    PointerEventKind, Propagation,
};
use morok_reactive::{Runtime, Signal};

use crate::menu_closer::MenuCloser;
use crate::menu_item::MenuItem;
use crate::menu_item_kind::MenuItemKind;
use crate::menu_panel::MenuPanel;
use crate::overlay_presence::show_overlay_for;
use crate::view::{ElementEvent, Mount, Overlay, OverlayAnchor, OverlayInteraction, Surfaces};

/// A menu opens with its top left corner where the pointer is.
const PLACEMENT: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Below,
    alignment: OverlayAlignment::Start,
    gap: 0.0,
    shift: 0.0,
};

/// The menu a press of the secondary button opens where the pointer is. By default it is the
/// menu of the system, in the look of the system; [`custom`](Self::custom) makes it a menu
/// the kit draws, as [`Menu`](crate::Menu) is.
#[derive(Debug)]
pub struct ContextMenu {
    items: Vec<MenuItem>,
    label: String,
    is_custom: bool,
}

impl ContextMenu {
    /// A context menu of `items`: the menu of the system, until [`custom`](Self::custom) says
    /// otherwise.
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            label: String::new(),
            is_custom: false,
        }
    }

    /// What an assistive technology calls the menu the kit draws.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Has the kit draw the menu, in an overlay of its own, in place of the menu of the system.
    #[must_use]
    pub fn custom(mut self) -> Self {
        self.is_custom = true;
        self
    }

    /// Builds a box around `target`: a press of the secondary button on anything inside it
    /// opens the menu.
    pub fn mount(self, mount: &mut Mount<'_>, target: ElementId) -> ElementId {
        let Self {
            items,
            label,
            is_custom,
        } = self;
        let items: Rc<[MenuItem]> = items.into();
        let requested_at = mount.runtime().signal(None);
        let area = mount
            .element()
            .on_event(move |runtime, event| match event {
                ElementEvent::Pointer(pointer) if opens_menu(pointer.kind) => {
                    runtime.set(requested_at, Some(pointer.position));
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            })
            .child(target)
            .id();
        if is_custom {
            show_custom_menu(mount, area, requested_at, items, label);
        } else {
            show_native_menu(mount, requested_at, items);
        }
        area
    }
}

fn opens_menu(kind: PointerEventKind) -> bool {
    matches!(
        kind,
        PointerEventKind::Pressed {
            button: PointerButton::Secondary,
            ..
        }
    )
}

/// Keeps the menu the kit draws open at the point asked for until it is closed or asked for
/// at another point.
fn show_custom_menu(
    mount: &mut Mount<'_>,
    area: ElementId,
    requested_at: Signal<Option<Point>>,
    items: Rc<[MenuItem]>,
    label: String,
) {
    let close: MenuCloser = Rc::new(move |runtime| runtime.set(requested_at, None));
    let dismissed = Rc::clone(&close);
    show_overlay_for(
        mount,
        area,
        requested_at,
        move |_, position: &Point| {
            let dismissed = Rc::clone(&dismissed);
            Overlay {
                anchor: OverlayAnchor::Point(*position),
                placement: PLACEMENT,
                interaction: OverlayInteraction::Interactive {
                    on_dismiss: Box::new(move |runtime| dismissed(runtime)),
                },
            }
        },
        move |mount, _| {
            MenuPanel {
                items: Rc::clone(&items),
                label: label.clone(),
                close_all: Rc::clone(&close),
                close_submenu: None,
                highlights_first: false,
            }
            .mount(mount)
        },
    );
}

/// Asks the host of the surface for the menu of the system whenever the menu is asked for.
fn show_native_menu(mount: &mut Mount<'_>, requested_at: Signal<Option<Point>>, items: Rc<[MenuItem]>) {
    let surface = mount.surface_id;
    mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
        let Some(position) = runtime.get(requested_at) else {
            return;
        };
        let menu = NativeMenu {
            position,
            items: runtime.untracked(|runtime| native_items(runtime, &items, &mut 0)),
        };
        let chosen_from = Rc::clone(&items);
        surfaces.surface_mut(surface).request_native_menu(
            menu,
            Box::new(move |runtime, id| {
                if let Some(MenuItemKind::Command { on_choose }) = command(&chosen_from, &mut 0, id) {
                    (on_choose.borrow_mut())(runtime);
                }
            }),
        );
        runtime.set(requested_at, None);
    });
}

/// The lines as the system shows them; the commands are numbered from `next_id` in the order
/// they are met, submenus included.
fn native_items(runtime: &mut Runtime<Surfaces>, items: &[MenuItem], next_id: &mut usize) -> Vec<NativeMenuItem> {
    items
        .iter()
        .map(|item| match &item.kind {
            MenuItemKind::Command { .. } => {
                let id = *next_id;
                *next_id += 1;
                NativeMenuItem::Command {
                    id,
                    label: item.label.clone(),
                    is_disabled: item.is_disabled.get(runtime),
                    shortcut: item.shortcut,
                }
            }
            MenuItemKind::Submenu { items } => NativeMenuItem::Submenu {
                label: item.label.clone(),
                items: native_items(runtime, items, next_id),
            },
            MenuItemKind::Separator => NativeMenuItem::Separator,
        })
        .collect()
}

/// The command numbered `id` by [`native_items`], counting from `next_id`.
fn command<'a>(items: &'a [MenuItem], next_id: &mut usize, id: usize) -> Option<&'a MenuItemKind> {
    for item in items {
        match &item.kind {
            MenuItemKind::Command { .. } => {
                if *next_id == id {
                    return Some(&item.kind);
                }
                *next_id += 1;
            }
            MenuItemKind::Submenu { items } => {
                if let Some(found) = command(items, next_id, id) {
                    return Some(found);
                }
            }
            MenuItemKind::Separator => {}
        }
    }
    None
}
