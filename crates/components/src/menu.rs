use std::rc::Rc;

use morok_core::{ElementId, OverlayAlignment, OverlayPlacement, OverlaySide};
use morok_reactive::Signal;

use crate::menu_closer::MenuCloser;
use crate::menu_item::MenuItem;
use crate::menu_panel::MenuPanel;
use crate::overlay_presence::show_overlay_while;
use crate::view::{Mount, Overlay, OverlayAnchor, OverlayInteraction};

const PLACEMENT: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Below,
    alignment: OverlayAlignment::Start,
    gap: 4.0,
    shift: 0.0,
};

/// A menu the kit draws: a floating column of commands, open while its signal holds. The
/// pointer and the arrow keys move the highlight; a click, Return and Space choose the
/// highlighted command, which closes the menu; the right arrow opens the submenu of a line
/// and the left arrow closes it. A press outside the menu and Escape close it too.
#[derive(Debug)]
pub struct Menu {
    is_open: Signal<bool>,
    items: Vec<MenuItem>,
    label: String,
}

impl Menu {
    pub fn new(is_open: Signal<bool>, items: Vec<MenuItem>) -> Self {
        Self {
            is_open,
            items,
            label: String::new(),
        }
    }

    /// What an assistive technology calls the menu.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Shows the menu under `anchor`, an element of the surface of `mount`, while the signal
    /// holds: in an overlay of its own, which flips over the anchor where the screen ends. A
    /// press on the anchor is left to the anchor, which opens and closes the menu.
    pub fn attach(self, mount: &mut Mount<'_>, anchor: ElementId) {
        let Self { is_open, items, label } = self;
        let items: Rc<[MenuItem]> = items.into();
        let close: MenuCloser = Rc::new(move |runtime| runtime.set(is_open, false));
        let dismissed = Rc::clone(&close);
        show_overlay_while(
            mount,
            anchor,
            is_open,
            move |_| {
                let dismissed = Rc::clone(&dismissed);
                Overlay {
                    anchor: OverlayAnchor::Element(anchor),
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

    /// Builds the panel of the menu on the surface of `mount` itself, for a host that shows
    /// the panel in a place of its own. Choosing a command writes the signal as it does in
    /// an overlay.
    pub fn mount_panel(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { is_open, items, label } = self;
        MenuPanel {
            items: items.into(),
            label,
            close_all: Rc::new(move |runtime| runtime.set(is_open, false)),
            close_submenu: None,
            highlights_first: false,
        }
        .mount(mount)
    }
}
