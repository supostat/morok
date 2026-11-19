use super::ElementTree;
use crate::input::input_handler::InputHandler;
use crate::input::shortcut::Shortcut;
use crate::input::shortcut_id::ShortcutId;
use crate::menu::menu_command::MenuCommand;
use crate::menu::native_menu_item::NativeMenuItem;

/// The lines the shortcuts of a tree give the menu bar: a host shows them while the window of
/// the tree has the keyboard, and tells the tree the line the user chose.
impl ElementTree {
    /// Shows the shortcut in the menu bar, on the line `command` names.
    ///
    /// # Panics
    ///
    /// When the shortcut is not registered.
    pub fn show_shortcut_in_menu_bar(&mut self, shortcut: ShortcutId, command: MenuCommand) {
        let registered = self
            .input
            .shortcuts
            .iter_mut()
            .find(|registered| registered.id == shortcut)
            .expect("a shortcut is registered before it is shown in the menu bar");
        registered.menu_command = Some(command);
        self.menu_bar_changed();
    }

    /// How many times the lines of the menu bar changed: a host builds the bar anew when the
    /// number it saw last is another.
    pub fn menu_bar_revision(&self) -> u64 {
        self.input.menu_bar_revision
    }

    /// The menus of the menu bar: one for every title the shortcuts name, in the order the
    /// titles first came, each with the lines of its shortcuts in the order they were
    /// registered. The `id` of a line is what
    /// [`perform_menu_command`](Self::perform_menu_command) takes.
    pub fn menu_bar(&self) -> Vec<NativeMenuItem> {
        let mut menus: Vec<NativeMenuItem> = Vec::new();
        for (shortcut, command) in self.menu_bar_shortcuts() {
            let line = NativeMenuItem::Command {
                id: line_id(shortcut.id),
                label: command.title.clone(),
                is_disabled: false,
                shortcut: Some(shortcut.chord),
            };
            let menu_of_title = menus.iter_mut().find_map(|menu| match menu {
                NativeMenuItem::Submenu { label, items } if *label == command.menu => Some(items),
                _ => None,
            });
            match menu_of_title {
                Some(lines) => lines.push(line),
                None => menus.push(NativeMenuItem::Submenu {
                    label: command.menu.clone(),
                    items: vec![line],
                }),
            }
        }
        menus
    }

    /// The user chose the line `id` of the menu bar: the handler of the input hears of its
    /// shortcut as if the chord were pressed. Returns whether the tree still has the shortcut.
    pub fn perform_menu_command(&mut self, id: usize, handler: &mut dyn InputHandler) -> bool {
        let chosen = self
            .menu_bar_shortcuts()
            .map(|(shortcut, _)| shortcut.id)
            .find(|shortcut| line_id(*shortcut) == id);
        if let Some(shortcut) = chosen {
            handler.shortcut(shortcut);
        }
        chosen.is_some()
    }

    /// The shortcuts the menu bar shows, each with its line.
    pub(super) fn menu_bar_shortcuts(&self) -> impl Iterator<Item = (&Shortcut, &MenuCommand)> {
        self.input
            .shortcuts
            .iter()
            .filter_map(|shortcut| shortcut.menu_command.as_ref().map(|command| (shortcut, command)))
    }

    /// The lines of the menu bar changed: the host builds the bar anew after the next frame.
    pub(super) fn menu_bar_changed(&mut self) {
        self.input.menu_bar_revision += 1;
        self.request_frame();
    }
}

/// What names the line of a shortcut to the host that shows the menu bar.
fn line_id(shortcut: ShortcutId) -> usize {
    usize::try_from(shortcut.serial()).expect("a tree registers fewer shortcuts than a `usize` counts")
}
