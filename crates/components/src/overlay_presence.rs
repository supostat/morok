use morok_core::{ElementId, OverlaySide};
use morok_reactive::{Runtime, Signal, Source};

use crate::view::{Mount, Overlay, Surfaces};

/// Keeps an overlay open over the surface of `mount` for as long as `shown` holds a key, and
/// opens it anew when the key changes: `overlay` says where the overlay for the key goes and
/// `content` builds what it shows. The overlay belongs to `owner`, an element of the surface.
pub(crate) fn show_overlay_for<Key: Clone + PartialEq + 'static>(
    mount: &mut Mount<'_>,
    owner: ElementId,
    shown: impl Into<Source<Option<Key>>>,
    overlay: impl Fn(&mut Runtime<Surfaces>, &Key) -> Overlay + 'static,
    content: impl Fn(&mut Mount<'_>, Signal<OverlaySide>) -> ElementId + 'static,
) {
    let shown = shown.into();
    let parent = mount.surface_id;
    let mut opened = None;
    mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
        let key = runtime.get(shown);
        // The surfaces close an overlay themselves when its owner or its parent goes.
        let still_open = opened.take().filter(|(_, overlay)| surfaces.contains(*overlay));
        if let Some((open_for, overlay)) = still_open {
            if key.as_ref() == Some(&open_for) {
                opened = Some((open_for, overlay));
                return;
            }
            surfaces.close(runtime, overlay);
        }
        if let Some(key) = key {
            let asked = runtime.untracked(|runtime| overlay(runtime, &key));
            opened = Some((key, surfaces.open_overlay(runtime, parent, owner, asked, &content)));
        }
    });
}

/// Keeps an overlay open over the surface of `mount` for as long as `is_open` holds, as
/// [`show_overlay_for`] does for a key.
pub(crate) fn show_overlay_while(
    mount: &mut Mount<'_>,
    owner: ElementId,
    is_open: impl Into<Source<bool>>,
    overlay: impl Fn(&mut Runtime<Surfaces>) -> Overlay + 'static,
    content: impl Fn(&mut Mount<'_>, Signal<OverlaySide>) -> ElementId + 'static,
) {
    let is_open = is_open.into();
    let shown = mount.runtime.memo(move |runtime| runtime.get(is_open).then_some(()));
    show_overlay_for(mount, owner, shown, move |runtime, ()| overlay(runtime), content);
}
