use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, OverlaySide};

use crate::panel_content::PanelContent;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// What an open popover shows: its arrow, then its title, its message and its content, each
/// when the popover has one.
#[derive(Clone)]
pub(crate) struct PopoverPanel {
    /// What an assistive technology calls the popover.
    pub(crate) label: Property<String>,
    pub(crate) title: Option<Property<String>>,
    pub(crate) message: Option<Property<String>>,
    pub(crate) content: Option<PanelContent>,
}

impl PopoverPanel {
    /// Builds the panel with its arrow on the edge that faces the anchor: the top edge for a
    /// popover below its anchor, the bottom edge for one above it.
    pub(crate) fn mount(&self, mount: &mut Mount<'_>, side: &Property<OverlaySide>) -> ElementId {
        let is_above = match *side {
            Property::Fixed(side) => Property::Fixed(side == OverlaySide::Above),
            Property::Bound(side) => mount
                .runtime()
                .memo(move |runtime| runtime.get(side) == OverlaySide::Above)
                .into(),
        };
        let arrow = mount
            .element()
            .class(CLASSES.popover_arrow)
            .class_when(CLASSES.above, is_above)
            .id();
        let title = self
            .title
            .clone()
            .map(|title| mount.text(title).class(CLASSES.popover_title).id());
        let message = self
            .message
            .clone()
            .map(|message| mount.text(message).class(CLASSES.popover_message).id());
        let content = self.content.as_ref().map(|content| content(mount));
        let mut panel = mount.element();
        panel
            .class(CLASSES.popover)
            .accessible_role(AccessibleRole::Dialog)
            .accessible_name(self.label.clone())
            .child(arrow);
        for part in title.into_iter().chain(message).chain(content) {
            panel.child(part);
        }
        panel.id()
    }
}
