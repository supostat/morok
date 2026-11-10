use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Focusability, ScrollBars};

use crate::styles::CLASSES;
use crate::view::{Mount, Property};

type Content = Box<dyn FnOnce(&mut Mount<'_>) -> ElementId>;

/// A region whose content scrolls inside it — by the wheel and the trackpad, by the keys while
/// focus is inside it, and by an assistive technology — under overlay scroll bars that show
/// while the content moves and while the pointer is over a bar. Whoever uses it gives it its
/// size.
pub struct ScrollView {
    content: Content,
    label: Property<String>,
}

impl ScrollView {
    /// A view of what `content` builds.
    pub fn new(content: impl FnOnce(&mut Mount<'_>) -> ElementId + 'static) -> Self {
        Self {
            content: Box::new(content),
            label: Property::Fixed(String::new()),
        }
    }

    /// What an assistive technology calls the view.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the view: a box that holds the viewport the content scrolls in and a bar of each
    /// axis over its edge, with a thumb on it.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { content, label } = self;
        let content = content(mount);
        let viewport = mount
            .scroll()
            .class(CLASSES.scroll_viewport)
            .focusable(Focusability::Focusable)
            .accessible_role(AccessibleRole::ScrollView)
            .accessible_name(label)
            .child(content)
            .id();
        let vertical_thumb = mount.element().class(CLASSES.scroll_thumb).id();
        let vertical_bar = mount
            .element()
            .class(CLASSES.scroll_bar)
            .class(CLASSES.vertical)
            .child(vertical_thumb)
            .id();
        let horizontal_thumb = mount.element().class(CLASSES.scroll_thumb).id();
        let horizontal_bar = mount
            .element()
            .class(CLASSES.scroll_bar)
            .class(CLASSES.horizontal)
            .child(horizontal_thumb)
            .id();
        mount.tree().set_scroll_bars(
            viewport,
            ScrollBars {
                vertical: Some(vertical_thumb),
                horizontal: Some(horizontal_thumb),
                showing: CLASSES.showing,
            },
        );
        mount
            .element()
            .class(CLASSES.scroll_view)
            .child(viewport)
            .child(vertical_bar)
            .child(horizontal_bar)
            .id()
    }
}

impl Debug for ScrollView {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("ScrollView")
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}
