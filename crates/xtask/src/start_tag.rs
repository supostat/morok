use crate::line_number;

const TAG_OPENING: char = '<';
const TAG_CLOSING: char = '>';
const END_TAG_OPENING: &str = "</";
const COMMENT_OPENING: &str = "<!--";
const COMMENT_CLOSING: &str = "-->";
const VALUE_ASSIGNMENT: char = '=';
const VALUE_QUOTE: char = '"';
const SELF_CLOSING_MARK: char = '/';
/// The elements whose text is not markup: a tag written inside one opens nothing.
const RAW_TEXT_ELEMENTS: [&str; 2] = ["script", "style"];

/// A start tag of a page: the element it opens, the line it starts on and the attributes it
/// writes in double quotes. A value written any other way is not read.
#[derive(Debug)]
pub(crate) struct StartTag<'page> {
    element: &'page str,
    pub(crate) line: usize,
    attributes: Vec<Attribute<'page>>,
}

#[derive(Debug)]
struct Attribute<'page> {
    name: &'page str,
    value: &'page str,
}

impl<'page> StartTag<'page> {
    /// The start tags of the page in its order. A comment and the text of a `script` or a
    /// `style` element are passed over.
    pub(crate) fn all_in(page: &'page str) -> Vec<Self> {
        let mut start_tags = Vec::new();
        let mut unread = page;
        while let Some(opening) = unread.find(TAG_OPENING) {
            unread = &unread[opening..];
            if let Some(comment) = unread.strip_prefix(COMMENT_OPENING) {
                unread = text_after(comment, COMMENT_CLOSING);
            } else if opens_an_element(unread) {
                let line = line_number::at(page, page.len() - unread.len());
                let (start_tag, after_tag) = Self::read(unread, line);
                unread = if start_tag.holds_raw_text() {
                    after_raw_text(start_tag.element, after_tag)
                } else {
                    after_tag
                };
                start_tags.push(start_tag);
            } else {
                unread = &unread[TAG_OPENING.len_utf8()..];
            }
        }
        start_tags
    }

    /// Whether the tag opens the element, whatever the case either is written in.
    pub(crate) fn opens(&self, element: &str) -> bool {
        self.element.eq_ignore_ascii_case(element)
    }

    /// The value of the attribute; none when the tag does not write it in double quotes.
    pub(crate) fn attribute(&self, name: &str) -> Option<&'page str> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case(name))
            .map(|attribute| attribute.value)
    }

    /// Reads the tag `tag_text` starts with; what follows the tag comes back with it.
    fn read(tag_text: &'page str, line: usize) -> (Self, &'page str) {
        let (element, mut unread) = split_where(&tag_text[TAG_OPENING.len_utf8()..], ends_a_name);
        let mut attributes = Vec::new();
        loop {
            unread = unread.trim_start_matches(separates_attributes);
            if let Some(after_tag) = unread.strip_prefix(TAG_CLOSING) {
                unread = after_tag;
                break;
            }
            if unread.is_empty() {
                break;
            }
            let (name, after_name) = split_where(unread, ends_a_name);
            let (quoted_value, after_attribute) = read_value(after_name);
            if let Some(value) = quoted_value {
                attributes.push(Attribute { name, value });
            }
            unread = after_attribute;
        }
        let start_tag = Self {
            element,
            line,
            attributes,
        };
        (start_tag, unread)
    }

    fn holds_raw_text(&self) -> bool {
        RAW_TEXT_ELEMENTS.iter().any(|element| self.opens(element))
    }
}

fn opens_an_element(text: &str) -> bool {
    text[TAG_OPENING.len_utf8()..].starts_with(|first: char| first.is_ascii_alphabetic())
}

fn ends_a_name(character: char) -> bool {
    character.is_ascii_whitespace() || [VALUE_ASSIGNMENT, TAG_CLOSING, SELF_CLOSING_MARK].contains(&character)
}

fn separates_attributes(character: char) -> bool {
    character.is_ascii_whitespace() || character == SELF_CLOSING_MARK
}

fn ends_an_unquoted_value(character: char) -> bool {
    character.is_ascii_whitespace() || character == TAG_CLOSING
}

/// The value an attribute writes in double quotes after its name, and what follows the
/// attribute. An attribute with no value, or with a value outside double quotes, has none.
fn read_value(after_name: &str) -> (Option<&str>, &str) {
    let Some(after_assignment) = after_name.trim_start().strip_prefix(VALUE_ASSIGNMENT) else {
        return (None, after_name);
    };
    let written = after_assignment.trim_start();
    match written.strip_prefix(VALUE_QUOTE) {
        Some(quoted) => {
            let (value, after_value) = quoted.split_once(VALUE_QUOTE).unwrap_or((quoted, ""));
            (Some(value), after_value)
        }
        None => (None, split_where(written, ends_an_unquoted_value).1),
    }
}

/// What follows the text of a raw text element: the page from the name of its end tag on.
fn after_raw_text<'page>(element: &str, text: &'page str) -> &'page str {
    let mut unread = text;
    while let Some((_, after_opening)) = unread.split_once(END_TAG_OPENING) {
        let closes_the_element = after_opening
            .get(..element.len())
            .is_some_and(|name| name.eq_ignore_ascii_case(element));
        if closes_the_element {
            return after_opening;
        }
        unread = after_opening;
    }
    ""
}

fn text_after<'page>(text: &'page str, mark: &str) -> &'page str {
    text.split_once(mark).map_or("", |(_, after_mark)| after_mark)
}

/// The text up to the first character that `ends` it, and the text from that character on.
fn split_where(text: &str, ends: fn(char) -> bool) -> (&str, &str) {
    text.find(ends).map_or((text, ""), |end| text.split_at(end))
}
