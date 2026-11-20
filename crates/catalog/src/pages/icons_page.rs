use morok::components::KitIcon;
use morok::components::view::Mount;
use morok::core::ElementId;

use crate::page_layout::{page, section};
use crate::styles::CLASSES;

/// The icons of the kit by name, and one of them at every size the kit draws icons at.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let mut cells = Vec::with_capacity(KitIcon::ALL.len());
    for icon in KitIcon::ALL {
        let glyph = mount.kit_icon(icon).class(CLASSES.icon_glyph).id();
        let name = mount.text(format!("{icon:?}")).id();
        cells.push(mount.element().class(CLASSES.icon_cell).child(glyph).child(name).id());
    }
    let mut grid = mount.element();
    grid.class(CLASSES.icon_grid);
    for cell in cells {
        grid.child(cell);
    }
    let grid = grid.id();
    let small = mount
        .kit_icon(KitIcon::Folder)
        .class(CLASSES.icon_glyph)
        .class(CLASSES.small)
        .id();
    let regular = mount.kit_icon(KitIcon::Folder).class(CLASSES.icon_glyph).id();
    let large = mount
        .kit_icon(KitIcon::Folder)
        .class(CLASSES.icon_glyph)
        .class(CLASSES.large)
        .id();
    let accent = mount
        .kit_icon(KitIcon::Folder)
        .class(CLASSES.icon_glyph)
        .class(CLASSES.large)
        .class(CLASSES.accent)
        .id();
    let flag = mount
        .kit_icon(KitIcon::Flag)
        .class(CLASSES.icon_glyph)
        .class(CLASSES.large)
        .id();
    let sizes = mount
        .element()
        .class(CLASSES.icon_sizes)
        .child(small)
        .child(regular)
        .child(large)
        .child(accent)
        .child(flag)
        .id();
    let set_section = section(mount, "THE SET", grid);
    let sizes_section = section(mount, "SIZES AND COLOR", sizes);
    page(mount, "Icons", &[set_section, sizes_section])
}
