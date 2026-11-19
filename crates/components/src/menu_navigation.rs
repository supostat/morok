use std::rc::Rc;

use morok_core::{AccessibilityAction, Key, KeyEvent, KeyPhase, PointerEventKind, Propagation};
use morok_reactive::{Runtime, Signal};

use crate::menu_closer::MenuCloser;
use crate::menu_item::MenuItem;
use crate::menu_item_kind::MenuItemKind;
use crate::menu_step::MenuStep;
use crate::view::{ElementEvent, Surfaces};

/// What the pointer and the keys do in one open menu: which line is highlighted, which
/// submenu is open, and what choosing a line does.
#[derive(Clone)]
pub(crate) struct MenuNavigation {
    pub(crate) items: Rc<[MenuItem]>,
    /// The line the pointer or the arrow keys are on.
    pub(crate) highlighted: Signal<Option<usize>>,
    /// The line whose submenu is open.
    pub(crate) open_submenu: Signal<Option<usize>>,
    /// Whether the submenu that opens next is opened from the keyboard: its first line is
    /// highlighted from the start then.
    pub(crate) submenu_takes_highlight: Signal<bool>,
    /// Closes this menu with every menu it was opened from.
    pub(crate) close_all: MenuCloser,
    /// Closes this menu alone when it is the submenu of a line.
    pub(crate) close_submenu: Option<MenuCloser>,
}

impl MenuNavigation {
    /// The first line the highlight can be on.
    pub(crate) fn first_choosable(&self, runtime: &mut Runtime<Surfaces>) -> Option<usize> {
        self.neighbor(runtime, None, MenuStep::Down)
    }

    /// What a line does with the events it hears of.
    pub(crate) fn item_event(
        &self,
        runtime: &mut Runtime<Surfaces>,
        position: usize,
        event: &ElementEvent<'_>,
    ) -> Propagation {
        match event {
            ElementEvent::Pointer(pointer) => match pointer.kind {
                PointerEventKind::Entered => self.pointer_entered(runtime, position),
                PointerEventKind::Left => self.pointer_left(runtime, position),
                PointerEventKind::Clicked { .. } => {
                    self.choose(runtime, position, false);
                    return Propagation::Stop;
                }
                _ => {}
            },
            ElementEvent::Accessibility(AccessibilityAction::Click) => {
                self.choose(runtime, position, false);
                return Propagation::Stop;
            }
            _ => {}
        }
        Propagation::Continue
    }

    /// What the menu does with a key that no line used.
    pub(crate) fn key_event(&self, runtime: &mut Runtime<Surfaces>, event: &KeyEvent) -> Propagation {
        if event.phase != KeyPhase::Pressed || !event.modifiers.is_empty() {
            return Propagation::Continue;
        }
        let highlighted = runtime.get(self.highlighted);
        match event.key {
            Key::ArrowDown => self.move_highlight(runtime, highlighted, MenuStep::Down),
            Key::ArrowUp => self.move_highlight(runtime, highlighted, MenuStep::Up),
            Key::Home => self.move_highlight(runtime, None, MenuStep::Down),
            Key::End => self.move_highlight(runtime, None, MenuStep::Up),
            Key::Enter | Key::Space => {
                if let Some(position) = highlighted {
                    self.choose(runtime, position, true);
                }
            }
            Key::ArrowRight => {
                if let Some(position) = highlighted
                    && matches!(self.items[position].kind, MenuItemKind::Submenu { .. })
                {
                    self.choose(runtime, position, true);
                }
            }
            Key::ArrowLeft => match &self.close_submenu {
                Some(close_submenu) => close_submenu(runtime),
                None => return Propagation::Continue,
            },
            // Focus stays in a menu until the menu closes.
            Key::Tab => {}
            _ => return Propagation::Continue,
        }
        Propagation::Stop
    }

    fn pointer_entered(&self, runtime: &mut Runtime<Surfaces>, position: usize) {
        let is_choosable = self.is_choosable(runtime, position);
        let opens_submenu = is_choosable && matches!(self.items[position].kind, MenuItemKind::Submenu { .. });
        if opens_submenu {
            set_changed(runtime, self.submenu_takes_highlight, false);
        }
        set_changed(runtime, self.highlighted, is_choosable.then_some(position));
        set_changed(runtime, self.open_submenu, opens_submenu.then_some(position));
    }

    /// The highlight goes with the pointer, unless the pointer went on to the submenu of the line.
    fn pointer_left(&self, runtime: &mut Runtime<Surfaces>, position: usize) {
        let keeps_submenu_open = runtime.get(self.open_submenu) == Some(position);
        if !keeps_submenu_open && runtime.get(self.highlighted) == Some(position) {
            runtime.set(self.highlighted, None);
        }
    }

    fn move_highlight(&self, runtime: &mut Runtime<Surfaces>, from: Option<usize>, step: MenuStep) {
        let to = self.neighbor(runtime, from, step);
        set_changed(runtime, self.highlighted, to);
        set_changed(runtime, self.open_submenu, None);
    }

    /// Chooses the line: a command runs and the whole menu closes, a submenu opens.
    fn choose(&self, runtime: &mut Runtime<Surfaces>, position: usize, by_keyboard: bool) {
        if !self.is_choosable(runtime, position) {
            return;
        }
        match &self.items[position].kind {
            MenuItemKind::Command { on_choose } => {
                (on_choose.borrow_mut())(runtime);
                (self.close_all)(runtime);
            }
            MenuItemKind::Submenu { .. } => {
                set_changed(runtime, self.submenu_takes_highlight, by_keyboard);
                set_changed(runtime, self.highlighted, Some(position));
                set_changed(runtime, self.open_submenu, Some(position));
            }
            MenuItemKind::Separator => {}
        }
    }

    /// The line after or before `from` the highlight can be on, around the ends; from no line
    /// the first or the last such line.
    fn neighbor(&self, runtime: &mut Runtime<Surfaces>, from: Option<usize>, step: MenuStep) -> Option<usize> {
        let count = self.items.len();
        let mut position = from;
        for _ in 0..count {
            let next = match (position, step) {
                (None, MenuStep::Down) => 0,
                (None, MenuStep::Up) => count - 1,
                (Some(position), MenuStep::Down) => (position + 1) % count,
                (Some(position), MenuStep::Up) => (position + count - 1) % count,
            };
            if self.is_choosable(runtime, next) {
                return Some(next);
            }
            position = Some(next);
        }
        None
    }

    fn is_choosable(&self, runtime: &mut Runtime<Surfaces>, position: usize) -> bool {
        let item = &self.items[position];
        !matches!(item.kind, MenuItemKind::Separator) && !item.is_disabled.get(runtime)
    }
}

/// Writes the signal when the value differs: a pointer that moves inside a line wakes nothing.
fn set_changed<Value: Clone + PartialEq + 'static>(
    runtime: &mut Runtime<Surfaces>,
    signal: Signal<Value>,
    value: Value,
) {
    if runtime.get(signal) != value {
        runtime.set(signal, value);
    }
}
