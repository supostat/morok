use std::time::Duration;

use morok::components::view::Mount;
use morok::components::{
    Button, ContextMenu, Dialog, KitIcon, Menu, MenuItem, Popover, ScrollView, Sheet, Toast, ToastStack, ToastTone,
    Tooltip,
};
use morok::core::{ElementId, Key, KeyChord, Modifiers};
use morok::reactive::Signal;

use crate::page_layout::{note, page, readout, row, section};
use crate::styles::CLASSES;

/// How long a toast stays.
const TOAST_LIFETIME: Duration = Duration::from_secs(4);
/// The tones the toasts take in turn.
const TOAST_TONES: [ToastTone; 3] = [ToastTone::Success, ToastTone::Warning, ToastTone::Danger];

/// What the overlays of the page are opened by and answer to.
#[derive(Clone, Copy, Debug)]
struct OverlayState {
    is_dialog_open: Signal<bool>,
    is_sheet_open: Signal<bool>,
    toasts: Signal<Vec<Toast>>,
    answer: Signal<String>,
}

/// The overlays: a popover, a tooltip, a menu with a submenu, a context menu of the system and
/// one of the kit, a dialog, a sheet and toasts. What the menus and the modals were answered
/// with is read back under them. The page scrolls; the dialog, the sheet and the toasts lie
/// over the room of the page, wherever it is scrolled to.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let state = OverlayState {
        is_dialog_open: mount.runtime().signal(false),
        is_sheet_open: mount.runtime().signal(false),
        toasts: mount.runtime().signal(Vec::new()),
        answer: mount.runtime().signal(String::from("nothing chosen yet")),
    };
    let OverlayState {
        is_dialog_open,
        is_sheet_open,
        toasts,
        answer,
    } = state;
    let scrolled = ScrollView::new(move |mount| content(mount, state))
        .label("Overlays")
        .mount(mount);
    let dialog = Dialog::new(is_dialog_open, "Delete 3 items permanently?")
        .message("This cannot be undone.")
        .cancel_action("Cancel", move |runtime| runtime.set(answer, String::from("cancel")))
        .default_action("Delete", move |runtime| runtime.set(answer, String::from("delete")))
        .mount(mount);
    let sheet = Sheet::new(is_sheet_open, "Save changes to “Notes”?")
        .message("Your changes are lost if you don’t save them.")
        .cancel_action("Don’t Save", move |runtime| {
            runtime.set(answer, String::from("don’t save"));
        })
        .default_action("Save", move |runtime| runtime.set(answer, String::from("save")))
        .mount(mount);
    let toast_stack = ToastStack::new(toasts).lifetime(TOAST_LIFETIME).mount(mount);
    mount
        .element()
        .class(CLASSES.page_layers)
        .child(scrolled)
        .child(dialog)
        .child(sheet)
        .child(toast_stack)
        .id()
}

/// The sections of the page.
fn content(mount: &mut Mount<'_>, state: OverlayState) -> ElementId {
    let OverlayState {
        is_dialog_open,
        is_sheet_open,
        toasts,
        answer,
    } = state;
    let is_popover_open = mount.runtime().signal(false);
    let is_menu_open = mount.runtime().signal(false);
    let toasts_shown = mount.runtime().signal(0_u64);
    let answered = mount
        .runtime()
        .memo(move |runtime| format!("last answer: {}", runtime.get(answer)));

    let popover_button = toggle_button(mount, "Show popover", is_popover_open);
    Popover::new(is_popover_open)
        .title("Xcode cache")
        .message("38.4 GB in DerivedData, rebuilt on the next build.")
        .content(move |mount| {
            let clean_up = Button::new("Clean up")
                .primary()
                .on_press(move |runtime| {
                    runtime.set(answer, String::from("clean up"));
                    runtime.set(is_popover_open, false);
                })
                .mount(mount);
            let later = Button::new("Later")
                .on_press(move |runtime| runtime.set(is_popover_open, false))
                .mount(mount);
            mount
                .element()
                .class(CLASSES.page_row)
                .child(clean_up)
                .child(later)
                .id()
        })
        .label("Xcode cache")
        .attach(mount, popover_button);
    let tooltip_button = Button::new("Recalculate").mount(mount);
    let tooltip = Tooltip::new("Recalculate the sizes of every folder").mount(mount, tooltip_button);
    let menu_button = toggle_button(mount, "Show menu", is_menu_open);
    Menu::new(is_menu_open, menu_items(answer))
        .label("File actions")
        .attach(mount, menu_button);
    let floating = row(mount, &[popover_button, tooltip, menu_button]);

    let native_target = note(mount, "Secondary click here for the menu of the system");
    let native_area = mount.element().class(CLASSES.page_tab_panel).child(native_target).id();
    let native_menu = ContextMenu::new(menu_items(answer))
        .label("File actions")
        .mount(mount, native_area);
    let custom_target = note(mount, "Secondary click here for the menu of the kit");
    let custom_area = mount.element().class(CLASSES.page_tab_panel).child(custom_target).id();
    let custom_menu = ContextMenu::new(menu_items(answer))
        .label("File actions")
        .custom()
        .mount(mount, custom_area);
    let context_menus = row(mount, &[native_menu, custom_menu]);

    let dialog_button = toggle_button(mount, "Delete…", is_dialog_open);
    let sheet_button = toggle_button(mount, "Close document…", is_sheet_open);
    let toast_button = Button::new("Show toast")
        .on_press(move |runtime| {
            let id = runtime.get(toasts_shown) + 1;
            runtime.set(toasts_shown, id);
            let tone = TOAST_TONES[(id % TOAST_TONES.len() as u64) as usize];
            runtime.update(toasts, |toasts| {
                toasts.push(Toast {
                    id,
                    title: format!("Freed {id}.2 GB"),
                    detail: Some(String::from("Moved to Trash")),
                    tone,
                });
            });
        })
        .mount(mount);
    let modals = row(mount, &[dialog_button, sheet_button, toast_button]);
    let answered = readout(mount, answered);

    let floating_section = section(mount, "POPOVER, TOOLTIP AND MENU", floating);
    let context_section = section(mount, "CONTEXT MENU", context_menus);
    let modals_section = section(mount, "DIALOG, SHEET AND TOAST", modals);
    let answers_section = section(mount, "ANSWERS", answered);
    page(
        mount,
        "Overlays",
        &[floating_section, context_section, modals_section, answers_section],
    )
}

/// A button that opens what `is_open` shows, and closes it again.
fn toggle_button(mount: &mut Mount<'_>, label: &str, is_open: Signal<bool>) -> ElementId {
    Button::new(label)
        .on_press(move |runtime| runtime.update(is_open, |is_open| *is_open = !*is_open))
        .mount(mount)
}

/// The commands on a file, each noting its name as the answer once chosen.
fn menu_items(answer: Signal<String>) -> Vec<MenuItem> {
    let command =
        |label: &'static str| MenuItem::new(label).on_choose(move |runtime| runtime.set(answer, label.to_lowercase()));
    vec![
        command("Quick Look").icon(KitIcon::Eye).shortcut(KeyChord {
            key: Key::Space,
            modifiers: Modifiers::NONE,
        }),
        command("Copy Path").icon(KitIcon::Copy).shortcut(KeyChord {
            key: Key::Character('c'),
            modifiers: Modifiers::COMMAND.union(Modifiers::OPTION),
        }),
        MenuItem::submenu("Open With", vec![command("Preview"), command("TextEdit")]),
        command("Remove Download").icon(KitIcon::Cloud).disabled(true),
        MenuItem::separator(),
        command("Move to Trash").icon(KitIcon::Trash).destructive(),
    ]
}
