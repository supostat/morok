//! A macOS UI kit that renders the whole interface through Metal.
//!
//! An application is built from components: functions that run once, build elements and bind
//! their properties to signals. This crate is the composition root — it gives the components
//! the windows, the text and the renderer of the platform — and the one crate an application
//! depends on: the layers of the kit are its modules.

#![forbid(unsafe_code)]

mod application;
mod application_state;
mod hosted_surface;
mod shared_canvas_painter;
mod styles;
mod window;

pub use application::Application;
pub use morok_components as components;
pub use morok_core as core;
pub use morok_platform::{
    Appearance, AtlasStatistics, AtlasUsage, CanvasEncoder, CanvasPainter, CanvasRegion, FrameImage, FrameStatistics,
    RendererError, RingSector, WindowOptions,
};
pub use morok_reactive as reactive;
pub use morok_style as style;
pub use styles::Styles;
pub use window::Window;

/// The signal graph of an application: its effects act on the surfaces of the application.
pub type Runtime = morok_reactive::Runtime<morok_components::view::Surfaces>;
