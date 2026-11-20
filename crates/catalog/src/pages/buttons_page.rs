use morok::components::view::Mount;
use morok::components::{Button, IconButton, KitIcon};
use morok::core::ElementId;

use crate::page_layout::{page, readout, row, section};

/// `Button` and `IconButton`: plain, primary and disabled, and an icon button that stays on.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let presses = mount.runtime().signal(0_u32);
    let is_starred = mount.runtime().signal(true);
    let counted = mount
        .runtime()
        .memo(move |runtime| format!("pressed {} times", runtime.get(presses)));

    let reveal = Button::new("Reveal")
        .on_press(move |runtime| runtime.update(presses, |presses| *presses += 1))
        .mount(mount);
    let clean_up = Button::new("Clean up 4.2 GB")
        .primary()
        .on_press(move |runtime| runtime.update(presses, |presses| *presses += 1))
        .mount(mount);
    let unavailable = Button::new("Unavailable").disabled(true).mount(mount);
    let unavailable_primary = Button::new("Empty Trash").primary().disabled(true).mount(mount);
    let buttons = row(mount, &[reveal, clean_up, unavailable, unavailable_primary]);
    let counted = readout(mount, counted);
    let buttons_section = section(mount, "BUTTON", buttons);
    let pressed_section = section(mount, "PRESSES", counted);

    let star = IconButton::new(KitIcon::Star, "Favourite")
        .toggled(is_starred)
        .mount(mount);
    let trash = IconButton::new(KitIcon::Trash, "Move to Trash")
        .on_press(move |runtime| runtime.update(presses, |presses| *presses += 1))
        .mount(mount);
    let settings = IconButton::new(KitIcon::Gear, "Settings").disabled(true).mount(mount);
    let icon_buttons = row(mount, &[star, trash, settings]);
    let icon_buttons_section = section(mount, "ICON BUTTON", icon_buttons);
    page(
        mount,
        "Buttons",
        &[buttons_section, pressed_section, icon_buttons_section],
    )
}
