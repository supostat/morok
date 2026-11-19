use std::time::Duration;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, KeyPhase, OverlayAlignment, OverlayPlacement, OverlaySide, PointerEventKind, Propagation};

use crate::overlay_presence::show_overlay_while;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Overlay, OverlayAnchor, OverlayInteraction, Property};

/// How long the pointer rests on the target before the tooltip shows.
const RESTING_TIME: Duration = Duration::from_millis(600);

const PLACEMENT: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Below,
    alignment: OverlayAlignment::Center,
    gap: 6.0,
    shift: 0.0,
};

/// A short note about a control: it shows under the control once the pointer has rested on
/// it, in an overlay the pointer and the keys pass by, and goes when the pointer leaves, a
/// button is pressed or a key goes down.
#[derive(Debug)]
pub struct Tooltip {
    text: Property<String>,
}

impl Tooltip {
    pub fn new(text: impl Into<Property<String>>) -> Self {
        Self { text: text.into() }
    }

    /// Builds a box around `target`: the tooltip belongs to everything inside it.
    pub fn mount(self, mount: &mut Mount<'_>, target: ElementId) -> ElementId {
        let pointer_rests = mount.runtime().signal(false);
        let mut area = mount.element();
        area.on_event(move |runtime, event| {
            let rests_now = match event {
                ElementEvent::Pointer(pointer) => match pointer.kind {
                    PointerEventKind::Entered => Some(true),
                    PointerEventKind::Left | PointerEventKind::Pressed { .. } => Some(false),
                    _ => None,
                },
                ElementEvent::Key(key) if key.phase == KeyPhase::Pressed => Some(false),
                _ => None,
            };
            if let Some(rests_now) = rests_now
                && runtime.get(pointer_rests) != rests_now
            {
                runtime.set(pointer_rests, rests_now);
            }
            Propagation::Continue
        })
        .child(target);
        let is_shown = area.held_for(pointer_rests, RESTING_TIME);
        let area = area.id();
        show_overlay_while(
            mount,
            area,
            is_shown,
            move |_| Overlay {
                anchor: OverlayAnchor::Element(area),
                placement: PLACEMENT,
                interaction: OverlayInteraction::Passive,
            },
            move |mount, _| mount_bubble(mount, self.text.clone()),
        );
        area
    }

    /// Builds the bubble of the tooltip on the surface of `mount` itself, for a host that
    /// shows the bubble in a place of its own.
    pub fn mount_bubble(self, mount: &mut Mount<'_>) -> ElementId {
        mount_bubble(mount, self.text)
    }
}

fn mount_bubble(mount: &mut Mount<'_>, text: Property<String>) -> ElementId {
    let note = mount.text(text.clone()).id();
    mount
        .element()
        .class(CLASSES.tooltip)
        .accessible_role(AccessibleRole::Tooltip)
        .accessible_name(text)
        .child(note)
        .id()
}
