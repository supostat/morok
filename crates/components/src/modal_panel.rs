use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Focusability, Key, KeyEvent, KeyPhase, Propagation};
use morok_reactive::{Runtime, Signal};

use crate::button::Button;
use crate::modal_action::ModalAction;
use crate::modal_classes::ModalClasses;
use crate::panel_content::PanelContent;
use crate::view::{ElementEvent, Mount, Property, Surfaces};

/// A panel the user has to answer before going on with the surface: it lies on a scrim that
/// covers the surface while its signal holds, and focus stays inside it. Return gives the
/// default answer and Escape the cancelling one, unless a control of the panel uses the key;
/// either closes the panel. When it closes, focus goes back to where it was.
#[derive(Clone)]
pub(crate) struct ModalPanel {
    pub(crate) is_open: Signal<bool>,
    pub(crate) title: Property<String>,
    pub(crate) message: Option<Property<String>>,
    pub(crate) content: Option<PanelContent>,
    pub(crate) default_action: Option<ModalAction>,
    pub(crate) cancel_action: Option<ModalAction>,
}

impl ModalPanel {
    pub(crate) fn new(is_open: Signal<bool>, title: Property<String>) -> Self {
        Self {
            is_open,
            title,
            message: None,
            content: None,
            default_action: None,
            cancel_action: None,
        }
    }

    pub(crate) fn action(
        label: Property<String>,
        handler: impl FnMut(&mut Runtime<Surfaces>) + 'static,
    ) -> ModalAction {
        ModalAction {
            label,
            run: Rc::new(RefCell::new(Box::new(handler))),
        }
    }

    /// Builds the element that holds the layer of the panel while the panel is open. The
    /// layer covers the box the element is positioned against: the surface, for an element
    /// with no positioned element around it.
    pub(crate) fn mount(self, mount: &mut Mount<'_>, classes: ModalClasses) -> ElementId {
        let holder = mount.element().id();
        let surface = mount.surface_id;
        let is_open = self.is_open;
        let layers = mount.runtime.scope(|_| {});
        let mut shown = None;
        mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
            let is_open = runtime.get(is_open);
            let shown_on = surfaces.surface_mut(surface);
            match (is_open, shown.take()) {
                (true, None) => {
                    let mut first_focus = None;
                    let (scope, layer) = Mount::in_scope(runtime, shown_on, surface, layers, |mount| {
                        self.mount_layer(mount, classes, &mut first_focus)
                    });
                    shown_on.tree.append_child(holder, layer);
                    shown_on.tree.trap_focus(layer);
                    shown_on
                        .tree
                        .focus(first_focus.expect("the layer is built with its panel"));
                    shown = Some((scope, layer));
                }
                (false, Some((scope, layer))) => {
                    // Before the layer leaves the tree: a trap that goes with its element
                    // gives focus back to no one.
                    shown_on.tree.release_focus_trap(layer);
                    runtime.dispose(scope);
                    shown_on.remove_subtree(layer);
                }
                (_, kept) => shown = kept,
            }
        });
        holder
    }

    /// Builds the scrim with the panel on it and names what takes focus when the panel opens:
    /// the button of the default answer, else that of the cancelling one, else the panel.
    fn mount_layer(
        &self,
        mount: &mut Mount<'_>,
        classes: ModalClasses,
        first_focus: &mut Option<ElementId>,
    ) -> ElementId {
        let title = mount.text(self.title.clone()).class(classes.title).id();
        let message = self
            .message
            .clone()
            .map(|message| mount.text(message).class(classes.message).id());
        let content = self.content.as_ref().map(|content| {
            let built = content(mount);
            mount.element().class(classes.content).child(built).id()
        });
        let cancel_button = self
            .cancel_action
            .as_ref()
            .map(|action| self.mount_button(mount, action, false));
        let default_button = self
            .default_action
            .as_ref()
            .map(|action| self.mount_button(mount, action, true));
        let mut actions = mount.element();
        actions.class(classes.actions);
        for button in cancel_button.into_iter().chain(default_button) {
            actions.child(button);
        }
        let actions = actions.id();

        let keys = self.clone();
        let mut panel = mount.element();
        panel
            .class(classes.panel)
            .focusable(Focusability::Focusable)
            .accessible_role(AccessibleRole::Dialog)
            .accessible_name(self.title.clone())
            .on_event(move |runtime, event| match event {
                ElementEvent::Key(key) => keys.key_event(runtime, key),
                _ => Propagation::Continue,
            })
            .child(title);
        for part in message.into_iter().chain(content).chain([actions]) {
            panel.child(part);
        }
        let panel = panel.id();
        *first_focus = default_button.or(cancel_button).or(Some(panel));
        mount.element().class(classes.layer).child(panel).id()
    }

    fn mount_button(&self, mount: &mut Mount<'_>, action: &ModalAction, is_default: bool) -> ElementId {
        let (answered, action) = (self.clone(), action.clone());
        let button = Button::new(action.label.clone()).on_press(move |runtime| answered.answer(runtime, Some(&action)));
        if is_default {
            button.primary().mount(mount)
        } else {
            button.mount(mount)
        }
    }

    /// Return gives the default answer and Escape the cancelling one; Escape closes a panel
    /// that has no cancelling answer.
    fn key_event(&self, runtime: &mut Runtime<Surfaces>, event: &KeyEvent) -> Propagation {
        if event.phase != KeyPhase::Pressed || !event.modifiers.is_empty() || event.is_repeat {
            return Propagation::Continue;
        }
        match (event.key, &self.default_action) {
            (Key::Enter, Some(default_action)) => self.answer(runtime, Some(default_action)),
            (Key::Escape, _) => self.answer(runtime, self.cancel_action.as_ref()),
            _ => return Propagation::Continue,
        }
        Propagation::Stop
    }

    /// Runs what the answer does, when there is one, and closes the panel.
    fn answer(&self, runtime: &mut Runtime<Surfaces>, action: Option<&ModalAction>) {
        if let Some(action) = action {
            (action.run.borrow_mut())(runtime);
        }
        runtime.set(self.is_open, false);
    }
}

impl Debug for ModalPanel {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("ModalPanel")
            .field("is_open", &self.is_open)
            .field("title", &self.title)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}
