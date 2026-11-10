use cssparser::{Parser, Token};
use morok_style::States;

/// A class of the kit holds single hyphens only, so no stylesheet names a class that starts
/// like this.
const MARKER_PREFIX: &str = "state--";
/// The class an element carries while it shows its placeholder. The placeholder of an edited
/// text is the text of its element on the page, so the rules of `::placeholder` style that
/// element; no other rule of the stylesheets of the kit sets what they set on it.
pub(crate) const PLACEHOLDER_MARKER: &str = "pseudo--placeholder";
const PLACEHOLDER_PSEUDO_ELEMENT: &str = "placeholder";

/// The class that stands for a state pseudo-class on the Chrome side.
fn marker_class(pseudo_class: &str) -> String {
    format!("{MARKER_PREFIX}{pseudo_class}")
}

/// The marker classes of the states an element is in.
pub(crate) fn of_states(states: States) -> impl Iterator<Item = String> {
    States::PSEUDO_CLASSES
        .into_iter()
        .filter(move |(_, state)| states.contains(*state))
        .map(|(pseudo_class, _)| marker_class(pseudo_class))
}

/// The stylesheet with every state pseudo-class of a selector turned into its marker class,
/// and `::placeholder` into the class of an element that shows its placeholder. A class weighs
/// as much as a pseudo-class, so the cascade stays as it was, and a marker can stand on any
/// element. Everything else, `::selection` and the declarations among it, is copied byte for
/// byte.
pub(crate) fn in_place_of_states(stylesheet: &str) -> String {
    let mut rewritten = String::with_capacity(stylesheet.len());
    let mut copied_up_to = 0;
    // Where the colons before the token being read began, and how many there are.
    let mut colons: Option<(usize, usize)> = None;
    let mut input = Parser::new(stylesheet);
    loop {
        let start = input.position().byte_index();
        // Outside the blocks of the rules every token belongs to a selector: the parser steps
        // over a block it is not asked to enter.
        let Some(token) = input.next_including_whitespace().ok().cloned() else {
            break;
        };
        let marker = match (&token, colons) {
            (Token::Colon, None) => {
                colons = Some((start, 1));
                continue;
            }
            (Token::Colon, Some((first, 1))) => {
                colons = Some((first, 2));
                continue;
            }
            (Token::Ident(name), Some((first, 1))) => States::PSEUDO_CLASSES
                .into_iter()
                .find(|(pseudo_class, _)| name.eq_ignore_ascii_case(pseudo_class))
                .map(|(pseudo_class, _)| (first, marker_class(pseudo_class))),
            (Token::Ident(name), Some((first, 2))) if name.eq_ignore_ascii_case(PLACEHOLDER_PSEUDO_ELEMENT) => {
                Some((first, PLACEHOLDER_MARKER.to_owned()))
            }
            _ => None,
        };
        colons = None;
        if let Some((first_colon, class)) = marker {
            rewritten.push_str(&stylesheet[copied_up_to..first_colon]);
            rewritten.push('.');
            rewritten.push_str(&class);
            copied_up_to = input.position().byte_index();
        }
    }
    rewritten.push_str(&stylesheet[copied_up_to..]);
    rewritten
}

#[cfg(test)]
mod tests {
    use morok_style::States;

    use super::{in_place_of_states, of_states};

    #[test]
    fn a_state_pseudo_class_becomes_its_marker_class_and_declarations_stay_as_they_are() {
        let stylesheet = ".button:hover,\n.button.primary:focus-visible { color: var(--color-text); }\n\
                          .toolbar:disabled > .button:checked { gap: 4px }\n";

        assert_eq!(
            in_place_of_states(stylesheet),
            ".button.state--hover,\n.button.primary.state--focus-visible { color: var(--color-text); }\n\
             .toolbar.state--disabled > .button.state--checked { gap: 4px }\n"
        );
    }

    #[test]
    fn a_pseudo_element_and_a_colon_inside_a_block_are_left_alone() {
        let stylesheet = ".field::selection { color: var(--color-text); }\n\
                          @keyframes hover { from { opacity: 0 } }\n.a { margin: 0 }";

        assert_eq!(in_place_of_states(stylesheet), stylesheet);
    }

    #[test]
    fn a_placeholder_becomes_the_class_of_an_element_that_shows_it() {
        let stylesheet = ".field-input::placeholder { color: var(--color-text-3); }\n\
                          .field:disabled > .field-input::placeholder { color: var(--color-text); }";

        assert_eq!(
            in_place_of_states(stylesheet),
            ".field-input.pseudo--placeholder { color: var(--color-text-3); }\n\
             .field.state--disabled > .field-input.pseudo--placeholder { color: var(--color-text); }"
        );
    }

    #[test]
    fn an_element_carries_the_markers_of_its_states() {
        let markers: Vec<String> = of_states(States::HOVER.union(States::CHECKED)).collect();

        assert_eq!(markers, ["state--hover", "state--checked"]);
    }
}
