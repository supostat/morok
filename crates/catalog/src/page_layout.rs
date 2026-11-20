//! The parts every page of a component family is built of: a title over sections, each a
//! caption over the components of the family laid out in a row or a column.

use morok::components::view::{Mount, Property};
use morok::core::ElementId;

use crate::styles::CLASSES;

/// A page: its title over its sections.
pub(crate) fn page(mount: &mut Mount<'_>, title: &str, sections: &[ElementId]) -> ElementId {
    let title = mount.text(title).class(CLASSES.page_title).id();
    let mut page = mount.element();
    page.class(CLASSES.page).child(title);
    for section in sections {
        page.child(*section);
    }
    page.id()
}

/// A page as tall as the room it has, whose sections share that room.
pub(crate) fn filled_page(mount: &mut Mount<'_>, title: &str, sections: &[ElementId]) -> ElementId {
    let page = page(mount, title, sections);
    mount.build(page).class(CLASSES.filled);
    page
}

/// A section: a caption over what `content` lays out.
pub(crate) fn section(mount: &mut Mount<'_>, caption: &str, content: ElementId) -> ElementId {
    let caption = mount.text(caption).class(CLASSES.page_caption).id();
    mount
        .element()
        .class(CLASSES.page_section)
        .child(caption)
        .child(content)
        .id()
}

/// A section that takes the room its page leaves.
pub(crate) fn filled_section(mount: &mut Mount<'_>, caption: &str, content: ElementId) -> ElementId {
    let section = section(mount, caption, content);
    mount.build(section).class(CLASSES.filled);
    section
}

/// Components side by side, wrapping onto further lines where the page ends.
pub(crate) fn row(mount: &mut Mount<'_>, children: &[ElementId]) -> ElementId {
    let mut row = mount.element();
    row.class(CLASSES.page_row);
    for child in children {
        row.child(*child);
    }
    row.id()
}

/// Components one under another.
pub(crate) fn column(mount: &mut Mount<'_>, children: &[ElementId]) -> ElementId {
    let mut column = mount.element();
    column.class(CLASSES.page_column);
    for child in children {
        column.child(*child);
    }
    column.id()
}

/// A line under a component that reads back what the component was given.
pub(crate) fn readout(mount: &mut Mount<'_>, text: impl Into<Property<String>>) -> ElementId {
    mount.text(text).class(CLASSES.page_readout).id()
}

/// A line that says what a section shows.
pub(crate) fn note(mount: &mut Mount<'_>, text: &str) -> ElementId {
    mount.text(text).class(CLASSES.page_note).id()
}
