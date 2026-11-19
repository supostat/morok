use std::rc::Rc;

use morok_core::ElementId;
use morok_reactive::{Runtime, Signal};

use crate::modal_classes::ModalClasses;
use crate::modal_panel::ModalPanel;
use crate::styles::CLASSES;
use crate::view::{Mount, Property, Surfaces};

/// A question the user answers before going on: a panel in the middle of a scrim that covers
/// the surface while the signal holds. Focus stays inside the dialog and goes back to where it
/// was when the dialog closes. Return gives the default answer and Escape the cancelling one.
#[derive(Debug)]
pub struct Dialog {
    panel: ModalPanel,
}

impl Dialog {
    /// A dialog with a title and nothing else yet.
    pub fn new(is_open: Signal<bool>, title: impl Into<Property<String>>) -> Self {
        Self {
            panel: ModalPanel::new(is_open, title.into()),
        }
    }

    /// A paragraph under the title.
    #[must_use]
    pub fn message(mut self, message: impl Into<Property<String>>) -> Self {
        self.panel.message = Some(message.into());
        self
    }

    /// What the panel shows over its buttons: built anew every time the dialog opens.
    #[must_use]
    pub fn content(mut self, content: impl Fn(&mut Mount<'_>) -> ElementId + 'static) -> Self {
        self.panel.content = Some(Rc::new(content));
        self
    }

    /// The answer the dialog is opened for: the button that stands out, pressed by Return
    /// as well. `handler` runs and the dialog closes.
    #[must_use]
    pub fn default_action(
        mut self,
        label: impl Into<Property<String>>,
        handler: impl FnMut(&mut Runtime<Surfaces>) + 'static,
    ) -> Self {
        self.panel.default_action = Some(ModalPanel::action(label.into(), handler));
        self
    }

    /// The answer that gives the dialog up: a plain button, pressed by Escape as well.
    /// `handler` runs and the dialog closes.
    #[must_use]
    pub fn cancel_action(
        mut self,
        label: impl Into<Property<String>>,
        handler: impl FnMut(&mut Runtime<Surfaces>) + 'static,
    ) -> Self {
        self.panel.cancel_action = Some(ModalPanel::action(label.into(), handler));
        self
    }

    /// Builds the element that holds the dialog while it is open. The scrim covers the box
    /// that element is positioned against: mounted under the root, the whole surface.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        self.panel.mount(
            mount,
            ModalClasses {
                layer: CLASSES.dialog_layer,
                panel: CLASSES.dialog,
                title: CLASSES.dialog_title,
                message: CLASSES.dialog_message,
                content: CLASSES.dialog_content,
                actions: CLASSES.dialog_actions,
            },
        )
    }
}
