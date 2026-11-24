use std::fmt::{Debug, Formatter, Result};
use std::ops::RangeInclusive;

use morok_core::ElementId;
use morok_reactive::Signal;

use crate::view::Mount;

/// A pane of a split view: what it shows, and how long it is along the axis of the view.
pub struct SplitPane {
    pub(crate) content: Box<dyn FnOnce(&mut Mount<'_>) -> ElementId>,
    /// How long the pane is along the axis; none for the pane that takes the room the others
    /// leave.
    pub(crate) length: Option<Signal<f32>>,
    /// The shortest and the longest the divider of the pane makes it.
    pub(crate) limits: RangeInclusive<f32>,
    /// Whether the pane is folded away, for a pane the divider folds.
    pub(crate) is_collapsed: Option<Signal<bool>>,
}

impl SplitPane {
    /// A pane that takes the room the other panes leave. A split view has one.
    pub fn flexible(content: impl FnOnce(&mut Mount<'_>) -> ElementId + 'static) -> Self {
        Self {
            content: Box::new(content),
            length: None,
            limits: 0.0..=0.0,
            is_collapsed: None,
        }
    }

    /// A pane as long as `length` holds along the axis of its view, in CSS pixels: as wide in
    /// a view of panes side by side, as tall in one of panes one above another. The divider
    /// between it and the pane next to it writes the signal, within `limits`.
    pub fn sized(
        length: Signal<f32>,
        limits: RangeInclusive<f32>,
        content: impl FnOnce(&mut Mount<'_>) -> ElementId + 'static,
    ) -> Self {
        Self {
            content: Box::new(content),
            length: Some(length),
            limits,
            is_collapsed: None,
        }
    }

    /// Lets the divider fold the pane away: a drag past half of its shortest length, or the
    /// arrow key that shortens it at that length, writes `true` to the signal, and a drag or
    /// an arrow back out writes `false`. The application folds and unfolds the pane by the
    /// same signal.
    ///
    /// # Panics
    ///
    /// When the pane is the flexible one.
    #[must_use]
    pub fn collapsible(mut self, is_collapsed: Signal<bool>) -> Self {
        assert!(self.length.is_some(), "only a pane of a length of its own folds away");
        self.is_collapsed = Some(is_collapsed);
        self
    }
}

impl Debug for SplitPane {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SplitPane")
            .field("length", &self.length)
            .field("limits", &self.limits)
            .field("is_collapsed", &self.is_collapsed)
            .finish_non_exhaustive()
    }
}
