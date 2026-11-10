//! The component model: a component is a function called once, when it is mounted. It builds
//! elements of the tree of a surface and binds their properties to signals; from then on a
//! signal that changes updates exactly the elements bound to it, and no component runs again.

mod action;
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
mod property;
mod scroll_area;
mod shortcut_handler;
mod surface;
mod surface_id;
mod surface_slot;
mod surfaces;

pub use action::Action;
pub use element_builder::ElementBuilder;
pub use element_event::ElementEvent;
pub use mount::Mount;
pub use property::Property;
pub use scroll_area::ScrollArea;
pub use surface::Surface;
pub use surface_id::SurfaceId;
pub use surfaces::Surfaces;
