use morok_core::{ElementId, ScrollBars};

use crate::styles::CLASSES;
use crate::view::Mount;

/// What a view whose content scrolls is built from: a box of the class `scroll-view` that holds
/// the viewport the content scrolls in and a bar of each axis over its edge, with a thumb the
/// tree moves. The component that builds it gives the viewport its role and its focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrollFrame {
    pub(crate) view: ElementId,
    pub(crate) viewport: ElementId,
}

impl ScrollFrame {
    /// Builds the frame around `content`.
    pub(crate) fn mount(mount: &mut Mount<'_>, content: ElementId) -> Self {
        let viewport = mount.scroll().class(CLASSES.scroll_viewport).child(content).id();
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
        let view = mount
            .element()
            .class(CLASSES.scroll_view)
            .child(viewport)
            .child(vertical_bar)
            .child(horizontal_bar)
            .id();
        Self { view, viewport }
    }
}
