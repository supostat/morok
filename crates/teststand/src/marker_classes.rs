use cssparser::{Parser, Token};
use morok_style::States;

/// A class of the kit holds single hyphens only, so no stylesheet names a class that starts
/// like this.
const MARKER_PREFIX: &str = "state--";

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

/// The stylesheet with every state pseudo-class of a selector turned into its marker class.
/// A class weighs as much as a pseudo-class, so the cascade stays as it was, and a marker can
/// stand on any element. Everything else, the declarations among it, is copied byte for byte.
pub(crate) fn in_place_of_states(stylesheet: &str) -> String {
    let mut rewritten = String::with_capacity(stylesheet.len());
    let mut copied_up_to = 0;
    let mut pseudo_class_colon = None;
    let mut input = Parser::new(stylesheet);
    loop {
        let start = input.position().byte_index();
        // Outside the blocks of the rules every token belongs to a selector: the parser steps
        // over a block it is not asked to enter.
        let Some(token) = input.next_including_whitespace().ok().cloned() else {
            break;
        };
        match token {
            Token::Colon if pseudo_class_colon.is_none() => pseudo_class_colon = Some(start),
            Token::Ident(name) => {
                let state = States::PSEUDO_CLASSES
                    .into_iter()
                    .find(|(pseudo_class, _)| name.eq_ignore_ascii_case(pseudo_class));
                if let (Some(colon), Some((pseudo_class, _))) = (pseudo_class_colon.take(), state) {
                    rewritten.push_str(&stylesheet[copied_up_to..colon]);
                    rewritten.push('.');
                    rewritten.push_str(&marker_class(pseudo_class));
                    copied_up_to = input.position().byte_index();
                }
            }
            // A second colon opens a pseudo-element, which is no state.
            _ => pseudo_class_colon = None,
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
    fn an_element_carries_the_markers_of_its_states() {
        let markers: Vec<String> = of_states(States::HOVER.union(States::CHECKED)).collect();

        assert_eq!(markers, ["state--hover", "state--checked"]);
    }
}
