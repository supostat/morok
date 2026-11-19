use std::time::Duration;

use morok_core::{ElementId, Key, KeyEvent, KeyPhase, Modifiers};
use morok_reactive::{Runtime, Signal};

use crate::view::{Mount, Surfaces};

/// How long after its last character the start of a name is remembered.
const MEMORY: Duration = Duration::from_millis(1000);

/// The start of a name the user types to reach one of several named things: a character that
/// follows the one before within a moment adds to it, and a pause begins a new one.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TypeAhead {
    typed: Signal<String>,
}

impl TypeAhead {
    /// Listens for `owner`, an element of the surface of `mount`: what was typed is forgotten
    /// by a timer of that element.
    pub(crate) fn mount(mount: &mut Mount<'_>, owner: ElementId) -> Self {
        let typed = mount.runtime.signal(String::new());
        let surface = mount.surface_id;
        let mut forgetting = None;
        mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
            let surface = surfaces.surface_mut(surface);
            if let Some(timer) = forgetting.take() {
                surface.cancel_timer(timer);
            }
            if runtime.with(typed, |typed| !typed.is_empty()) {
                let forget = Box::new(move |runtime: &mut Runtime<Surfaces>| runtime.set(typed, String::new()));
                forgetting = Some(surface.start_timer(owner, MEMORY, forget));
            }
        });
        Self { typed }
    }

    /// Adds the character the key types to the start of the name and returns the start as it
    /// stands now, in lower case; none for a key that types nothing. The space bar types only
    /// inside a name: before one it is the key that presses a control.
    pub(crate) fn typed_by(self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Option<String> {
        let types = key.phase == KeyPhase::Pressed && (key.modifiers.is_empty() || key.modifiers == Modifiers::SHIFT);
        if !types {
            return None;
        }
        let is_inside_a_name = runtime.with(self.typed, |typed| !typed.is_empty());
        let character = match key.key {
            Key::Character(character) => character,
            Key::Space if is_inside_a_name => ' ',
            _ => return None,
        };
        runtime.update(self.typed, |typed| typed.extend(character.to_lowercase()));
        Some(runtime.get(self.typed))
    }
}
