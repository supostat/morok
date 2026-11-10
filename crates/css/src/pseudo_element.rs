use morok_style::Declaration;

/// A part of an element that a rule styles in place of the element itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PseudoElement {
    /// The text an edited text shows while it is empty.
    Placeholder,
    /// The selected part of an edited text. Selected text keeps its color: a rule gives the
    /// selection its background only.
    Selection,
}

impl PseudoElement {
    pub(crate) const HINT: &'static str =
        "the pseudo-elements are `::placeholder` and `::selection`; draw anything else as an element of its own";

    pub(crate) fn named(name: &str) -> Option<Self> {
        [Self::Placeholder, Self::Selection]
            .into_iter()
            .find(|pseudo_element| name.eq_ignore_ascii_case(pseudo_element.name()))
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Placeholder => "placeholder",
            Self::Selection => "selection",
        }
    }

    /// What a rule of the pseudo-element may set; finishes the sentence "it takes …".
    pub(crate) const fn expectation(self) -> &'static str {
        match self {
            Self::Placeholder => "`color`",
            Self::Selection => "`background-color`",
        }
    }

    /// The declaration of the element that carries what a rule gives the pseudo-element: the
    /// style of a pseudo-element is a part of the style of its element. None for a property
    /// the pseudo-element does not take.
    pub(crate) fn declaration(self, of_rule: Declaration) -> Option<Declaration> {
        match (self, of_rule) {
            (Self::Placeholder, Declaration::Color(color)) => Some(Declaration::PlaceholderColor(color)),
            (Self::Selection, Declaration::BackgroundColor(color)) => {
                Some(Declaration::SelectionBackgroundColor(color))
            }
            _ => None,
        }
    }
}
