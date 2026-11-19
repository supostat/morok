use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{Focusability, Key, KeyEvent, KeyPhase, Propagation};
use morok_reactive::{Runtime, Signal};
use morok_style::States;

use crate::field_elements::FieldElements;
use crate::kit_icon::KitIcon;
use crate::styles::CLASSES;
use crate::view::{Action, ElementEvent, Mount, Property, Surfaces};

/// What a component built around a field does with a key before the field does anything
/// with it; a key it uses goes no further.
pub(crate) type FieldKeyHandler = Box<dyn FnMut(&mut Runtime<Surfaces>, &KeyEvent) -> Propagation>;

/// Which of the fields of the kit is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FieldKind {
    Line,
    Search,
    Area,
}

impl FieldKind {
    const fn accessible_role(self) -> AccessibleRole {
        match self {
            Self::Line => AccessibleRole::TextInput,
            Self::Search => AccessibleRole::SearchInput,
            Self::Area => AccessibleRole::MultilineTextInput,
        }
    }

    /// Whether the key submits what the field holds: Return in one line. In lines Return
    /// breaks a line.
    fn submits(self, key: &KeyEvent) -> bool {
        self != Self::Area && key.phase == KeyPhase::Pressed && key.key == Key::Enter && key.modifiers.is_empty()
    }
}

/// What the fields of the kit are built from: a box of the class `field` around the text the
/// user edits, and for a search field a magnifier before it.
pub(crate) struct FieldParts {
    pub(crate) value: Signal<String>,
    pub(crate) placeholder: Property<String>,
    pub(crate) label: Property<String>,
    pub(crate) is_disabled: Property<bool>,
    pub(crate) is_invalid: Property<bool>,
    pub(crate) on_submit: Option<Action>,
    /// Hears of every key before the field: the list a combo box drops down is moved by them.
    pub(crate) on_key: Option<FieldKeyHandler>,
    /// Hears of every change the user makes to the text, once the value has taken it.
    pub(crate) on_edit: Option<Action>,
}

impl FieldParts {
    pub(crate) fn new(value: Signal<String>) -> Self {
        Self {
            value,
            placeholder: Property::Fixed(String::new()),
            label: Property::Fixed(String::new()),
            is_disabled: Property::Fixed(false),
            is_invalid: Property::Fixed(false),
            on_submit: None,
            on_key: None,
            on_edit: None,
        }
    }

    pub(crate) fn mount(self, kind: FieldKind, mount: &mut Mount<'_>) -> FieldElements {
        let Self {
            value,
            placeholder,
            label,
            is_disabled,
            is_invalid,
            mut on_submit,
            mut on_key,
            mut on_edit,
        } = self;
        let icon = (kind == FieldKind::Search).then(|| mount.kit_icon(KitIcon::Search).class(CLASSES.field_icon).id());
        let mut input = if kind == FieldKind::Area {
            mount.text_area()
        } else {
            mount.text_input()
        };
        input
            .class(CLASSES.field_input)
            .edits(value)
            .placeholder(placeholder)
            .state(States::DISABLED, is_disabled.clone())
            .focusable(Focusability::TabStop)
            .accessible_role(kind.accessible_role())
            .accessible_name(label)
            .on_event(move |runtime, event| match event {
                ElementEvent::TextChanged(text) => {
                    runtime.set(value, (*text).to_owned());
                    if let Some(edited) = &mut on_edit {
                        edited(runtime);
                    }
                    Propagation::Stop
                }
                ElementEvent::Key(key) => {
                    if let Some(on_key) = &mut on_key
                        && on_key(runtime, key) == Propagation::Stop
                    {
                        return Propagation::Stop;
                    }
                    match &mut on_submit {
                        Some(submit) if kind.submits(key) => {
                            submit(runtime);
                            Propagation::Stop
                        }
                        _ => Propagation::Continue,
                    }
                }
                ElementEvent::Pointer(_) | ElementEvent::Accessibility(_) => Propagation::Continue,
            });
        let input = input.id();
        let mut field = mount.element();
        field.class(CLASSES.field);
        match kind {
            FieldKind::Line => {}
            FieldKind::Search => {
                field.class(CLASSES.search);
            }
            FieldKind::Area => {
                field.class(CLASSES.area);
            }
        }
        field
            .class_when(CLASSES.invalid, is_invalid)
            .state(States::DISABLED, is_disabled);
        if let Some(icon) = icon {
            field.child(icon);
        }
        let field = field.child(input).id();
        FieldElements { field, input }
    }
}
