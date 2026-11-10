use morok_core::{ElementId, IconId};
use morok_reactive::Runtime;
use morok_style::{StyleSheet, Theme};

use crate::kit_icon::KitIcon;
use crate::view::mount::Mount;
use crate::view::surface::Surface;
use crate::view::surface_id::SurfaceId;
use crate::view::surface_slot::SurfaceSlot;

const CLOSED_SURFACE: &str = "a surface is not used after it is closed";

/// The surfaces of an application: what the effects of its components act on. One signal
/// graph serves them all, so a signal may be shown on several surfaces, and a flush updates
/// every surface it concerns.
#[derive(Debug, Default)]
pub struct Surfaces {
    slots: Vec<SurfaceSlot>,
    free_indices: Vec<u32>,
    /// The IDs of the icons of the kit, by [`KitIcon`]; empty until the host registers them.
    kit_icon_ids: Vec<IconId>,
}

impl Surfaces {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the icons of the kit with the renderer of the host through `register`, which
    /// takes the SVG of an icon and gives the ID the renderer shows it by. Surfaces opened from
    /// then on show the icons the components ask for.
    pub fn register_kit_icons(&mut self, mut register: impl FnMut(&'static str) -> IconId) {
        self.kit_icon_ids = KitIcon::ALL.iter().map(|icon| register(icon.svg())).collect();
    }

    /// Opens a surface with an empty tree styled by `style_sheet` under `theme`.
    pub fn open(&mut self, runtime: &mut Runtime<Self>, style_sheet: StyleSheet, theme: Theme) -> SurfaceId {
        let surface = Surface::new(style_sheet, theme, runtime.scope(|_| {}), self.kit_icon_ids.clone());
        if let Some(index) = self.free_indices.pop() {
            let slot = &mut self.slots[index as usize];
            slot.surface = Some(surface);
            return SurfaceId::new(index, slot.generation);
        }
        let index = u32::try_from(self.slots.len()).expect("an application opens fewer than 2^32 surfaces");
        self.slots.push(SurfaceSlot {
            generation: 0,
            surface: Some(surface),
        });
        SurfaceId::new(index, 0)
    }

    /// Closes the surface: everything mounted on it is disposed and its tree is dropped.
    ///
    /// # Panics
    ///
    /// When the surface was closed already.
    pub fn close(&mut self, runtime: &mut Runtime<Self>, surface: SurfaceId) {
        let slot = &mut self.slots[surface.index()];
        assert_eq!(slot.generation, surface.generation(), "{CLOSED_SURFACE}");
        let closed = slot.surface.take().expect(CLOSED_SURFACE);
        slot.generation += 1;
        self.free_indices
            .push(u32::try_from(surface.index()).expect("the index came out of a surface ID"));
        runtime.dispose(closed.scope);
    }

    /// # Panics
    ///
    /// When the surface was closed.
    pub fn surface(&self, surface: SurfaceId) -> &Surface {
        let slot = &self.slots[surface.index()];
        assert_eq!(slot.generation, surface.generation(), "{CLOSED_SURFACE}");
        slot.surface.as_ref().expect(CLOSED_SURFACE)
    }

    /// # Panics
    ///
    /// When the surface was closed.
    pub fn surface_mut(&mut self, surface: SurfaceId) -> &mut Surface {
        let slot = &mut self.slots[surface.index()];
        assert_eq!(slot.generation, surface.generation(), "{CLOSED_SURFACE}");
        slot.surface.as_mut().expect(CLOSED_SURFACE)
    }

    /// Asks the host of every surface for a frame. A frame begins with a flush of the effects,
    /// so this is how a signal written between frames gets shown.
    pub fn request_frames(&mut self) {
        for surface in self.slots.iter_mut().filter_map(|slot| slot.surface.as_mut()) {
            surface.tree.request_frame();
        }
    }

    /// Mounts a component on the surface: calls it, once, and makes the element it built the
    /// last child of `parent`. What the component creates lives until the surface is closed.
    ///
    /// # Panics
    ///
    /// When the surface was closed.
    pub fn mount(
        &mut self,
        runtime: &mut Runtime<Self>,
        surface: SurfaceId,
        parent: ElementId,
        component: impl FnOnce(&mut Mount<'_>) -> ElementId,
    ) -> ElementId {
        let mounted_on = self.surface_mut(surface);
        let owner = mounted_on.scope;
        let (_, element) = Mount::in_scope(runtime, mounted_on, surface, owner, component);
        mounted_on.tree.append_child(parent, element);
        element
    }
}
