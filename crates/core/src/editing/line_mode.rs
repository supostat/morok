use std::borrow::Cow;

use crate::role::Role;

/// Whether an edited text is one line or lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LineMode {
    Single,
    Multiple,
}

impl LineMode {
    /// The mode of the edited text of a role; none for a role that edits no text.
    pub(crate) const fn of(role: Role) -> Option<Self> {
        match role {
            Role::TextInput => Some(Self::Single),
            Role::TextArea => Some(Self::Multiple),
            Role::Element | Role::Text | Role::Image | Role::Icon | Role::Canvas | Role::Scroll => None,
        }
    }

    /// The text as the mode holds it. One line holds no line break, as a text field of a
    /// browser has it: the breaks a text ends with are dropped and every other becomes a
    /// space. Lines break at a line feed, whatever the text was written with.
    pub(crate) fn sanitized(self, text: &str) -> Cow<'_, str> {
        if !text.contains(['\n', '\r']) {
            return Cow::Borrowed(text);
        }
        match self {
            Self::Single => {
                let kept = text.trim_end_matches(['\n', '\r']);
                Cow::Owned(kept.replace("\r\n", " ").replace(['\n', '\r'], " "))
            }
            Self::Multiple => Cow::Owned(text.replace("\r\n", "\n").replace('\r', "\n")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineMode;

    #[test]
    fn one_line_drops_the_breaks_at_its_end_and_turns_the_others_into_spaces() {
        assert_eq!(LineMode::Single.sanitized("one\r\ntwo\nthree\n\n"), "one two three");
        assert_eq!(LineMode::Single.sanitized("plain"), "plain");
    }

    #[test]
    fn lines_break_at_line_feeds_only() {
        assert_eq!(LineMode::Multiple.sanitized("one\r\ntwo\rthree\n"), "one\ntwo\nthree\n");
    }
}
