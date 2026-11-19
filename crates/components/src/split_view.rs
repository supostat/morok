use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;
use morok_style::Declaration;

use crate::length_in_pixels::pixels;
use crate::split_divider::SplitDivider;
use crate::split_pane::SplitPane;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// Panes side by side that fill the box they are put in, with a divider between every two of
/// them. One pane takes the room the others leave; each of the others is as wide as its
/// signal holds, and the divider beside it changes that width: the pointer drags it, the
/// arrow keys and an assistive technology move it by steps, within the limits of the pane. A
/// pane that can fold is folded away by a drag far past its narrowest width and by Return or
/// Space on its divider.
pub struct SplitView {
    panes: Vec<SplitPane>,
    label: Property<String>,
}

impl SplitView {
    /// A split view with no panes yet.
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            label: Property::Fixed(String::new()),
        }
    }

    /// Adds a pane after the ones added before.
    #[must_use]
    pub fn pane(mut self, pane: SplitPane) -> Self {
        self.panes.push(pane);
        self
    }

    /// What an assistive technology calls the dividers of the view.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the view: a row of the panes with a divider between every two.
    ///
    /// # Panics
    ///
    /// When the view has not exactly one flexible pane.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { panes, label } = self;
        let flexible = panes.iter().position(|pane| pane.width.is_none());
        assert_eq!(
            panes.iter().filter(|pane| pane.width.is_none()).count(),
            1,
            "a split view has one pane that takes the room the others leave"
        );
        let flexible = flexible.expect("the count of flexible panes was checked");
        let mut parts = Vec::with_capacity(panes.len() * 2);
        for (position, pane) in panes.into_iter().enumerate() {
            // A divider belongs to the sized pane beside it: to the pane before it up to the
            // flexible pane, and to the pane after it from there on.
            let divider = pane.width.map(|width| SplitDivider {
                label: label.clone(),
                width,
                limits: pane.limits.clone(),
                is_collapsed: pane.is_collapsed,
                pane_lies_before: position < flexible,
            });
            let pane_element = mount_pane(mount, pane.width, pane.is_collapsed, pane.content);
            match divider {
                Some(divider) if divider.pane_lies_before => {
                    parts.push(pane_element);
                    parts.push(divider.mount(mount));
                }
                Some(divider) => {
                    parts.push(divider.mount(mount));
                    parts.push(pane_element);
                }
                None => parts.push(pane_element),
            }
        }
        let mut view = mount.element();
        view.class(CLASSES.split);
        for part in parts {
            view.child(part);
        }
        view.id()
    }
}

/// Builds the box of a pane around its content: as wide as the signal of the pane holds, gone
/// while the pane is folded, or flexible.
fn mount_pane(
    mount: &mut Mount<'_>,
    width: Option<morok_reactive::Signal<f32>>,
    is_collapsed: Option<morok_reactive::Signal<bool>>,
    content: Box<dyn FnOnce(&mut Mount<'_>) -> ElementId>,
) -> ElementId {
    let content = content(mount);
    let as_wide_as_asked = width.map(|width| {
        mount
            .runtime()
            .memo(move |runtime| Declaration::Width(pixels(runtime.get(width))))
    });
    let mut pane = mount.element();
    pane.class(CLASSES.split_pane);
    match as_wide_as_asked {
        Some(width) => {
            pane.inline(width);
        }
        None => {
            pane.class(CLASSES.flexible);
        }
    }
    if let Some(is_collapsed) = is_collapsed {
        pane.class_when(CLASSES.collapsed, is_collapsed);
    }
    pane.child(content).id()
}

impl Default for SplitView {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for SplitView {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SplitView")
            .field("panes", &self.panes)
            .field("label", &self.label)
            .finish()
    }
}
