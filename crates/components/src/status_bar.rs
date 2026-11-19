use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId, Focusability, KeyChord};
use morok_reactive::Runtime;

use crate::activation::{ActivationKeys, handle_activation};
use crate::shortcut_label::shortcut_label;
use crate::status_part::StatusPart;
use crate::styles::CLASSES;
use crate::view::{Action, Mount, Property, Surfaces};

/// The bar at the bottom of a window: lines of text that say what the window shows now, and
/// the commands of the window with the chords that give them. A click on a command, Space or
/// Return while it has focus and an assistive technology give the command; the bar answers
/// no chord itself — it shows the one the application registered for the same command.
#[derive(Debug)]
pub struct StatusBar {
    parts: Vec<StatusPart>,
    label: Property<String>,
}

impl StatusBar {
    /// A status bar that holds nothing yet.
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            label: Property::Fixed(String::new()),
        }
    }

    /// Adds a line of text, after what was added before.
    #[must_use]
    pub fn text(mut self, text: impl Into<Property<String>>) -> Self {
        self.parts.push(StatusPart::Text(text.into()));
        self
    }

    /// Adds a command: its chord in a key cap, then its name. A press of it runs `on_press`.
    #[must_use]
    pub fn command(
        mut self,
        chord: KeyChord,
        label: impl Into<String>,
        on_press: impl FnMut(&mut Runtime<Surfaces>) + 'static,
    ) -> Self {
        self.parts.push(StatusPart::Command {
            chord,
            label: label.into(),
            on_press: Box::new(on_press),
        });
        self
    }

    /// Adds room that takes what the other parts leave: what is added next stands at the far
    /// end of the bar.
    #[must_use]
    pub fn spacer(mut self) -> Self {
        self.parts.push(StatusPart::Spacer);
        self
    }

    /// What an assistive technology calls the status bar.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the status bar: a row of its parts.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { parts, label } = self;
        let part_elements: Vec<ElementId> = parts
            .into_iter()
            .map(|part| match part {
                StatusPart::Text(text) => mount.text(text).id(),
                StatusPart::Command { chord, label, on_press } => mount_command(mount, chord, &label, on_press),
                StatusPart::Spacer => mount.element().class(CLASSES.status_spacer).id(),
            })
            .collect();
        let mut bar = mount.element();
        bar.class(CLASSES.statusbar)
            .accessible_role(AccessibleRole::Status)
            .accessible_name(label);
        for part in part_elements {
            bar.child(part);
        }
        bar.id()
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a command of the bar: the key cap of its chord and its name.
fn mount_command(mount: &mut Mount<'_>, chord: KeyChord, label: &str, mut on_press: Action) -> ElementId {
    let key_name = mount.text(shortcut_label(chord)).id();
    let key_cap = mount.element().class(CLASSES.status_key).child(key_name).id();
    let name = mount.text(label).id();
    let is_disabled = Property::Fixed(false);
    mount
        .element()
        .class(CLASSES.status_command)
        .focusable(Focusability::TabStop)
        .accessible_role(AccessibleRole::Button)
        .accessible_name(label)
        .accessible_actions(AccessibleActions::CLICK)
        .on_event(move |runtime, event| {
            handle_activation(
                runtime,
                event,
                ActivationKeys::SpaceAndReturn,
                &is_disabled,
                &mut on_press,
            )
        })
        .child(key_cap)
        .child(name)
        .id()
}
