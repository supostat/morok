use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibleActions, AccessibleValue, ElementId, ElementTree, Focusability, KeyChord, NumericValue, Propagation,
    ShortcutScope,
};
use morok_reactive::{Runtime, Signal, Source};
use morok_style::{Class, Declaration, States};

use crate::view::element_event::ElementEvent;
use crate::view::keyed_fragment::KeyedFragment;
use crate::view::mount::Mount;
use crate::view::property::Property;
use crate::view::shortcut_handler::ShortcutHandler;
use crate::view::surfaces::Surfaces;

/// Builds on one element while its component is mounted: every call gives the element a
/// property, a child or a handler. A property given a signal or a memo follows it from then on,
/// through an effect that writes the property of this very element.
#[derive(Debug)]
pub struct ElementBuilder<'m, 'a> {
    pub(crate) mount: &'m mut Mount<'a>,
    pub(crate) element: ElementId,
}

impl ElementBuilder<'_, '_> {
    /// The element built on.
    pub fn id(&self) -> ElementId {
        self.element
    }

    pub fn class(&mut self, class: Class) -> &mut Self {
        self.mount.surface.tree.add_class(self.element, class);
        self
    }

    /// Gives the element the class for as long as `is_on` holds.
    pub fn class_when(&mut self, class: Class, is_on: impl Into<Property<bool>>) -> &mut Self {
        self.bind(is_on.into(), move |tree, element, is_on| {
            if *is_on {
                tree.add_class(element, class);
            } else {
                tree.remove_class(element, class);
            }
        })
    }

    /// Puts the element in the given states for as long as `is_on` holds. The tree drives
    /// `hover`, `active` and the focus states itself.
    pub fn state(&mut self, states: States, is_on: impl Into<Property<bool>>) -> &mut Self {
        self.bind(is_on.into(), move |tree, element, is_on| {
            tree.set_state(element, states, *is_on);
        })
    }

    /// Sets one property of the style of the element directly, over every rule of the style
    /// sheet.
    pub fn inline(&mut self, declaration: impl Into<Property<Declaration>>) -> &mut Self {
        self.bind(declaration.into(), |tree, element, declaration| {
            tree.set_inline(element, declaration.clone());
        })
    }

    /// The text an element that edits text shows while it holds none.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn placeholder(&mut self, placeholder: impl Into<Property<String>>) -> &mut Self {
        self.bind(placeholder.into(), |tree, element, placeholder| {
            tree.set_placeholder(element, placeholder);
        })
    }

    /// Makes the text an element edits follow `value`. What the user makes of the text reaches
    /// the handler of the element, which writes it back to `value`; the text then equals the
    /// value and is left as it is, its caret and its history with it.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub(crate) fn edits(&mut self, value: Signal<String>) -> &mut Self {
        self.bind(Property::from(value), |tree, element, text: &String| {
            tree.set_text(element, text);
        })
    }

    /// Says whether the element takes focus and whether the Tab key stops on it.
    pub fn focusable(&mut self, focusability: impl Into<Property<Focusability>>) -> &mut Self {
        self.bind(focusability.into(), |tree, element, focusability| {
            tree.set_focusability(element, *focusability);
        })
    }

    /// Gives the element a node of the role in the accessibility tree.
    pub fn accessible_role(&mut self, role: AccessibleRole) -> &mut Self {
        self.mount.surface.tree.set_accessible_role(self.element, Some(role));
        self
    }

    /// Names the node of the element: what an assistive technology calls it.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn accessible_name(&mut self, name: impl Into<Property<String>>) -> &mut Self {
        self.bind(name.into(), |tree, element, name| {
            tree.set_accessible_name(element, name);
        })
    }

    /// Says what number the node of the element stands for, and the range it moves in.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn accessible_number(&mut self, number: impl Into<Property<NumericValue>>) -> &mut Self {
        self.bind(number.into(), |tree, element, number| {
            tree.set_accessible_value(element, AccessibleValue::Number(*number));
        })
    }

    /// Makes the node of the element tell whether the element is checked, as a button that
    /// stays pressed does.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn accessible_toggle(&mut self) -> &mut Self {
        self.mount.surface.tree.set_accessible_toggle(self.element, true);
        self
    }

    /// Says what an assistive technology may ask the element to do.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn accessible_actions(&mut self, actions: AccessibleActions) -> &mut Self {
        self.mount.surface.tree.set_accessible_actions(self.element, actions);
        self
    }

    /// Makes `child` the last child of the element.
    pub fn child(&mut self, child: ElementId) -> &mut Self {
        self.mount.surface.tree.append_child(self.element, child);
        self
    }

    /// A child that is there while `when` holds: `content` is mounted when it comes to hold
    /// and what it built is removed, with its signals and effects, when it holds no longer.
    /// A condition written again with the value it has leaves the child as it is. The child
    /// lies after the children the element has now.
    pub fn show(
        &mut self,
        when: impl Into<Source<bool>>,
        content: impl Fn(&mut Mount<'_>) -> ElementId + 'static,
    ) -> &mut Self {
        let when = when.into();
        let mut fragment = self.keyed_fragment::<()>();
        let mut keys = Vec::new();
        self.mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
            if runtime.get(when) {
                keys.push(());
            }
            fragment.reconcile(runtime, surfaces, &mut keys, &|mount, _| content(mount));
        });
        self
    }

    /// A child for every item of `items`, in their order, told apart by `key`: `item` is
    /// mounted once for a key that comes, an item whose key stays keeps its element wherever
    /// the item moves, and the element of a key that goes is removed with its signals and
    /// effects. What changes inside an item that keeps its key is carried by the signals the
    /// item holds. The children lie after the children the element has now.
    pub fn for_each<Item, Key>(
        &mut self,
        items: impl Into<Source<Vec<Item>>>,
        key: impl Fn(&Item) -> Key + 'static,
        item: impl Fn(&mut Mount<'_>, &Item) -> ElementId + 'static,
    ) -> &mut Self
    where
        Item: Clone + 'static,
        Key: PartialEq + 'static,
    {
        let items = items.into();
        let mut fragment = self.keyed_fragment::<Key>();
        let mut keys = Vec::new();
        self.mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
            runtime.with(items, |items| keys.extend(items.iter().map(&key)));
            fragment.reconcile(runtime, surfaces, &mut keys, &|mount, position| {
                let mounted = mount.runtime.with(items, |items| items[position].clone());
                item(mount, &mounted)
            });
        });
        self
    }

    /// Names what hears of the events of the element: what the pointer does to it, the keys
    /// that go down and up while focus is on it or inside it, and what an assistive technology
    /// asks of it. An event the handler does not use goes on to the element around this one.
    /// The handler writes signals; it replaces a handler the element had.
    pub fn on_event(
        &mut self,
        handler: impl FnMut(&mut Runtime<Surfaces>, &ElementEvent<'_>) -> Propagation + 'static,
    ) -> &mut Self {
        self.mount
            .surface
            .event_handlers
            .set_element_handler(self.element, Box::new(handler));
        self
    }

    /// Registers a shortcut that answers while focus is on the element or inside it, and on
    /// the root of a surface also while nothing has focus. It goes with the element.
    pub fn shortcut(&mut self, chord: KeyChord, handler: impl FnMut(&mut Runtime<Surfaces>) + 'static) -> &mut Self {
        let surface = &mut *self.mount.surface;
        let shortcut = surface
            .tree
            .register_shortcut(ShortcutScope::Element(self.element), chord);
        surface.event_handlers.set_shortcut_handler(
            shortcut,
            ShortcutHandler {
                element: self.element,
                run: Box::new(handler),
            },
        );
        self
    }

    /// Gives the element the value of the property through `apply`: at once for a value that
    /// stays, and in every flush after its source changed for a value that is bound.
    pub(crate) fn bind<Value: 'static>(
        &mut self,
        property: Property<Value>,
        apply: impl Fn(&mut ElementTree, ElementId, &Value) + 'static,
    ) -> &mut Self {
        let element = self.element;
        match property {
            Property::Fixed(value) => apply(&mut self.mount.surface.tree, element, &value),
            Property::Bound(source) => {
                let surface = self.mount.surface_id;
                self.mount.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
                    let tree = &mut surfaces.surface_mut(surface).tree;
                    runtime.with(source, |value| apply(tree, element, value));
                });
            }
        }
        self
    }

    /// Opens a fragment after the children the element has now.
    fn keyed_fragment<Key>(&mut self) -> KeyedFragment<Key> {
        let surface = &mut *self.mount.surface;
        let children_now = surface.tree.children(self.element).len();
        KeyedFragment {
            surface: self.mount.surface_id,
            fragment: surface.child_fragments.open(self.element, children_now),
            scope: self.mount.runtime.scope(|_| {}),
            items: Vec::new(),
        }
    }
}
