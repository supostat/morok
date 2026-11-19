use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::{ElementId, OverlayAlignment, OverlayPlacement, OverlaySide};
use morok_reactive::Signal;

use crate::overlay_presence::show_overlay_while;
use crate::popover_panel::PopoverPanel;
use crate::view::{Mount, Overlay, OverlayAnchor, OverlayInteraction, Property};

/// The arrow of the panel reaches 6 px out of it and stops short of the anchor.
const PLACEMENT: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Below,
    alignment: OverlayAlignment::Start,
    gap: 10.0,
    shift: 0.0,
};

/// A floating panel that belongs to a control: open while its signal holds, in an overlay of
/// its own with the keyboard in it. A press outside the panel and Escape close it.
pub struct Popover {
    is_open: Signal<bool>,
    panel: PopoverPanel,
}

impl Popover {
    /// A popover that shows nothing yet.
    pub fn new(is_open: Signal<bool>) -> Self {
        Self {
            is_open,
            panel: PopoverPanel {
                label: Property::Fixed(String::new()),
                title: None,
                message: None,
                content: None,
            },
        }
    }

    /// What an assistive technology calls the popover.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.panel.label = label.into();
        self
    }

    /// A line in bold at the top of the panel.
    #[must_use]
    pub fn title(mut self, title: impl Into<Property<String>>) -> Self {
        self.panel.title = Some(title.into());
        self
    }

    /// A paragraph in small type under the title.
    #[must_use]
    pub fn message(mut self, message: impl Into<Property<String>>) -> Self {
        self.panel.message = Some(message.into());
        self
    }

    /// What the panel shows under its message: built anew every time the popover opens.
    #[must_use]
    pub fn content(mut self, content: impl Fn(&mut Mount<'_>) -> ElementId + 'static) -> Self {
        self.panel.content = Some(Rc::new(content));
        self
    }

    /// Shows the popover under `anchor`, an element of the surface of `mount`, while the
    /// signal holds: it flips over the anchor where the screen ends. A press on the anchor
    /// is left to the anchor, which opens and closes the popover.
    pub fn attach(self, mount: &mut Mount<'_>, anchor: ElementId) {
        let Self { is_open, panel } = self;
        show_overlay_while(
            mount,
            anchor,
            is_open,
            move |_| Overlay {
                anchor: OverlayAnchor::Element(anchor),
                placement: PLACEMENT,
                interaction: OverlayInteraction::Interactive {
                    on_dismiss: Box::new(move |runtime| runtime.set(is_open, false)),
                },
            },
            move |mount, side| panel.mount(mount, &side.into()),
        );
    }

    /// Builds the panel of the popover on the surface of `mount` itself, as it lies on
    /// `side` of what it belongs to, for a host that shows the panel in a place of its own.
    pub fn mount_panel(self, mount: &mut Mount<'_>, side: impl Into<Property<OverlaySide>>) -> ElementId {
        self.panel.mount(mount, &side.into())
    }
}

impl Debug for Popover {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Popover")
            .field("is_open", &self.is_open)
            .field("label", &self.panel.label)
            .field("title", &self.panel.title)
            .field("message", &self.panel.message)
            .finish_non_exhaustive()
    }
}
