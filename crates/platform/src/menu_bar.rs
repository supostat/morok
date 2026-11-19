pub(crate) mod menu_bar_target;

use std::cell::Cell;
use std::ptr;

use morok_core::NativeMenuItem;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::{NSProcessInfo, NSString};

use crate::menu_bar::menu_bar_target::MenuBarTarget;
use crate::native_menu;
use crate::surface_view::SurfaceView;

thread_local! {
    /// The view whose menus the menu bar shows, by its address; none until a view showed any.
    static SHOWN_FOR: Cell<Option<usize>> = const { Cell::new(None) };
}

/// Whether the menu bar shows the menus of `view`.
pub(crate) fn shows_menus_of(view: &SurfaceView) -> bool {
    SHOWN_FOR.get() == Some(ptr::from_ref(view).addr())
}

/// Whether the menu bar shows the menus of any view.
pub(crate) fn shows_menus() -> bool {
    SHOWN_FOR.get().is_some()
}

/// Shows `menus` in the menu bar for `view`, after the menu of the application itself: each a
/// menu of the bar, whose lines tell `target` when the user chooses them.
pub(crate) fn show(
    view: &SurfaceView,
    menus: &[NativeMenuItem],
    target: &MenuBarTarget,
    main_thread: MainThreadMarker,
) {
    let receiver: &AnyObject = target;
    let bar = native_menu::menu_of(menus, receiver, sel!(menuBarCommandChosen:), main_thread);
    bar.insertItem_atIndex(&application_menu(main_thread), 0);
    NSApplication::sharedApplication(main_thread).setMainMenu(Some(&bar));
    SHOWN_FOR.set(Some(ptr::from_ref(view).addr()));
}

/// The view closes: when the menu bar shows its menus, the bar is left with the menu of the
/// application until another view shows its own.
pub(crate) fn forget(view: &SurfaceView, main_thread: MainThreadMarker) {
    if !shows_menus_of(view) {
        return;
    }
    let bar = NSMenu::new(main_thread);
    bar.addItem(&application_menu(main_thread));
    NSApplication::sharedApplication(main_thread).setMainMenu(Some(&bar));
    SHOWN_FOR.set(None);
}

/// The menu the system titles with the name of the application; the kit puts the line that
/// quits the application into it.
fn application_menu(main_thread: MainThreadMarker) -> Retained<NSMenuItem> {
    let name = NSProcessInfo::processInfo().processName();
    let quit = NSMenuItem::new(main_thread);
    quit.setTitle(&NSString::from_str(&format!("Quit {name}")));
    quit.setKeyEquivalent(&NSString::from_str("q"));
    // SAFETY: the application implements the selector, and a line with no target goes to
    // the first responder that does.
    unsafe { quit.setAction(Some(sel!(terminate:))) };
    let menu = NSMenu::new(main_thread);
    menu.addItem(&quit);
    let opener = NSMenuItem::new(main_thread);
    opener.setSubmenu(Some(&menu));
    opener
}
