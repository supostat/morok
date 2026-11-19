mod menu_target;

use morok_core::{Key, KeyChord, Modifiers, NativeMenu, NativeMenuItem};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::{NSEventModifierFlags, NSMenu, NSMenuItem, NSView};
use objc2_foundation::{NSPoint, NSString};

use menu_target::MenuTarget;

/// The modifiers of a chord with the flags `AppKit` knows them by.
const MODIFIER_FLAGS: [(Modifiers, NSEventModifierFlags); 4] = [
    (Modifiers::SHIFT, NSEventModifierFlags::Shift),
    (Modifiers::CONTROL, NSEventModifierFlags::Control),
    (Modifiers::OPTION, NSEventModifierFlags::Option),
    (Modifiers::COMMAND, NSEventModifierFlags::Command),
];
/// The first of the code points `AppKit` names the function keys by.
const FIRST_FUNCTION_KEY: u32 = 0xF704;

/// Shows the menu over the surface of `view`, in the look of the system, and waits until the
/// user chose a line or gave the menu up. Returns the `id` of the command chosen.
///
/// The menu runs an event loop of its own meanwhile: nothing that the callbacks of the system
/// reach may be borrowed while this runs.
pub(crate) fn pop_up(view: &NSView, menu: &NativeMenu, main_thread: MainThreadMarker) -> Option<usize> {
    let target = MenuTarget::new(main_thread);
    let shown = menu_of(&menu.items, &target, main_thread);
    let location = NSPoint::new(f64::from(menu.position.x), f64::from(menu.position.y));
    shown.popUpMenuPositioningItem_atLocation_inView(None, location, Some(view));
    target.chosen()
}

fn menu_of(items: &[NativeMenuItem], target: &MenuTarget, main_thread: MainThreadMarker) -> Retained<NSMenu> {
    let menu = NSMenu::new(main_thread);
    // Each line says itself whether it can be chosen.
    menu.setAutoenablesItems(false);
    for item in items {
        menu.addItem(&menu_item_of(item, target, main_thread));
    }
    menu
}

fn menu_item_of(item: &NativeMenuItem, target: &MenuTarget, main_thread: MainThreadMarker) -> Retained<NSMenuItem> {
    match item {
        NativeMenuItem::Separator => NSMenuItem::separatorItem(main_thread),
        NativeMenuItem::Submenu { label, items } => {
            let opener = NSMenuItem::new(main_thread);
            opener.setTitle(&NSString::from_str(label));
            opener.setSubmenu(Some(&menu_of(items, target, main_thread)));
            opener
        }
        NativeMenuItem::Command {
            id,
            label,
            is_disabled,
            shortcut,
        } => {
            let command = NSMenuItem::new(main_thread);
            command.setTitle(&NSString::from_str(label));
            command.setEnabled(!is_disabled);
            command.setTag(isize::try_from(*id).expect("a menu has fewer commands than an `isize` counts"));
            let receiver: &AnyObject = target;
            // SAFETY: the target implements the selector, taking the item, and outlives the
            // menu: both live until the menu closed.
            unsafe {
                command.setTarget(Some(receiver));
                command.setAction(Some(sel!(commandChosen:)));
            }
            if let Some(chord) = shortcut {
                command.setKeyEquivalent(&NSString::from_str(&key_equivalent(chord.key)));
                command.setKeyEquivalentModifierMask(modifier_flags(*chord));
            }
            command
        }
    }
}

fn modifier_flags(chord: KeyChord) -> NSEventModifierFlags {
    MODIFIER_FLAGS
        .iter()
        .filter(|(modifier, _)| chord.modifiers.contains(*modifier))
        .fold(NSEventModifierFlags::empty(), |flags, (_, flag)| flags | *flag)
}

/// The character `AppKit` shows a key by beside a line of a menu.
fn key_equivalent(key: Key) -> String {
    let character = match key {
        Key::Character(character) => character,
        Key::Enter => '\r',
        Key::Tab => '\t',
        Key::Space => ' ',
        Key::Escape => '\u{1b}',
        Key::Backspace => '\u{8}',
        Key::Delete => '\u{7f}',
        Key::ArrowUp => '\u{f700}',
        Key::ArrowDown => '\u{f701}',
        Key::ArrowLeft => '\u{f702}',
        Key::ArrowRight => '\u{f703}',
        Key::Home => '\u{f729}',
        Key::End => '\u{f72b}',
        Key::PageUp => '\u{f72c}',
        Key::PageDown => '\u{f72d}',
        Key::Function(number) => char::from_u32(FIRST_FUNCTION_KEY + u32::from(number) - 1)
            .expect("the function keys lie in the private use area"),
    };
    character.to_string()
}
