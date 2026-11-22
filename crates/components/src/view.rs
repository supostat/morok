//! The component model: a component is a function called once, when it is mounted. It builds
//! elements of the tree of a surface and binds their properties to signals; from then on a
//! signal that changes updates exactly the elements bound to it, and no component runs again.

mod action;
mod carried_drag;
mod child_fragments;
mod element_builder;
mod element_event;
mod event_dispatch;
mod event_handlers;
mod fragment_extent;
mod fragment_id;
mod keyed_fragment;
mod mount;
mod mounted_item;
mod native_menu_choice;
mod overlay;
mod overlay_anchor;
mod overlay_attachment;
mod overlay_geometry;
mod overlay_interaction;
mod overlay_place;
mod property;
mod scroll_area;
mod shortcut_handler;
mod surface;
mod surface_id;
mod surface_slot;
mod surfaces;
mod timer_handler;

pub use action::Action;
pub use element_builder::ElementBuilder;
pub use element_event::ElementEvent;
pub use mount::Mount;
pub use native_menu_choice::NativeMenuChoice;
pub use overlay::Overlay;
pub use overlay_anchor::OverlayAnchor;
pub use overlay_geometry::OverlayGeometry;
pub use overlay_interaction::OverlayInteraction;
pub use overlay_place::OverlayPlace;
pub use property::Property;
pub use scroll_area::ScrollArea;
pub use surface::Surface;
pub use surface_id::SurfaceId;
pub use surfaces::Surfaces;
