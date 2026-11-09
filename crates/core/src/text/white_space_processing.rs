use morok_style::WhiteSpace;

/// The text as its `white-space` has it laid out. Where white space collapses, every run of
/// spaces, tabs and line breaks becomes one space, and the text neither starts nor ends with
/// one: the text of an element is a block of its own, and CSS drops the spaces at the edges of
/// a line. `scratch` holds the result when the text had to change.
pub(crate) fn processed<'a>(text: &'a str, white_space: WhiteSpace, scratch: &'a mut String) -> &'a str {
    if !white_space.collapses() {
        return text;
    }
    scratch.clear();
    for word in text.split(is_collapsible).filter(|word| !word.is_empty()) {
        if !scratch.is_empty() {
            scratch.push(' ');
        }
        scratch.push_str(word);
    }
    scratch
}

fn is_collapsible(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r' | '\u{C}')
}

#[cfg(test)]
mod tests {
    use morok_style::WhiteSpace;

    use super::processed;

    #[test]
    fn collapsing_white_space_leaves_one_space_between_words() {
        let mut scratch = String::new();

        assert_eq!(
            processed("  one \t two\n\nthree ", WhiteSpace::Normal, &mut scratch),
            "one two three"
        );
        assert_eq!(processed(" a  b ", WhiteSpace::Nowrap, &mut scratch), "a b");
    }

    #[test]
    fn preserved_white_space_is_kept_as_written() {
        let mut scratch = String::new();

        assert_eq!(
            processed("  one \n two ", WhiteSpace::Pre, &mut scratch),
            "  one \n two "
        );
        assert_eq!(processed(" a\tb ", WhiteSpace::PreWrap, &mut scratch), " a\tb ");
    }

    #[test]
    fn a_text_of_white_space_alone_collapses_to_nothing() {
        let mut scratch = String::new();

        assert_eq!(processed(" \n\t ", WhiteSpace::Normal, &mut scratch), "");
    }
}
