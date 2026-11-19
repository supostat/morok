use std::cell::RefCell;

use morok_core::KeyChord;
use morok_reactive::Runtime;

use crate::kit_icon::KitIcon;
use crate::menu_item_kind::MenuItemKind;
use crate::view::{Property, Surfaces};

/// A line of a menu: a command the user chooses, a line that opens a submenu, or a separator.
#[derive(Debug)]
pub struct MenuItem {
    pub(crate) label: String,
    pub(crate) icon: Option<KitIcon>,
    pub(crate) shortcut: Option<KeyChord>,
    pub(crate) is_disabled: Property<bool>,
    pub(crate) is_destructive: bool,
    pub(crate) kind: MenuItemKind,
}

impl MenuItem {
    /// A command that does nothing when chosen.
    pub fn new(label: impl Into<String>) -> Self {
        Self::of_kind(
            label.into(),
            MenuItemKind::Command {
                on_choose: RefCell::new(Box::new(|_| {})),
            },
        )
    }

    /// A line that opens a menu of `items` beside it.
    pub fn submenu(label: impl Into<String>, items: Vec<Self>) -> Self {
        Self::of_kind(label.into(), MenuItemKind::Submenu { items: items.into() })
    }

    /// A thin line between groups of items.
    pub fn separator() -> Self {
        Self::of_kind(String::new(), MenuItemKind::Separator)
    }

    fn of_kind(label: String, kind: MenuItemKind) -> Self {
        Self {
            label,
            icon: None,
            shortcut: None,
            is_disabled: Property::Fixed(false),
            is_destructive: false,
            kind,
        }
    }

    /// Shows an icon of the kit before the label. The menu of the system shows none.
    #[must_use]
    pub fn icon(mut self, icon: KitIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows the chord beside the label: the shortcut the application registered for the
    /// same command. The item itself answers no chord.
    #[must_use]
    pub fn shortcut(mut self, chord: KeyChord) -> Self {
        self.shortcut = Some(chord);
        self
    }

    /// A disabled item is shown dimmed, and neither the pointer nor the keys stop on it.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Marks a command that destroys something; a menu the kit draws shows it in the color
    /// of danger.
    #[must_use]
    pub fn destructive(mut self) -> Self {
        self.is_destructive = true;
        self
    }

    /// Names what choosing the command does.
    ///
    /// # Panics
    ///
    /// When the item is no command.
    #[must_use]
    pub fn on_choose(mut self, handler: impl FnMut(&mut Runtime<Surfaces>) + 'static) -> Self {
        assert!(
            matches!(self.kind, MenuItemKind::Command { .. }),
            "only a command is chosen"
        );
        self.kind = MenuItemKind::Command {
            on_choose: RefCell::new(Box::new(handler)),
        };
        self
    }
}
