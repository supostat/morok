use std::ops::RangeInclusive;

const CYRILLIC: RangeInclusive<char> = '\u{0400}'..='\u{052F}';
const NUMBERED_WORK_ITEMS: [&str; 3] = ["phase", "task", "step"];
const NUMBER_SEPARATORS: [char; 4] = [' ', '\t', '#', '-'];
const SPEC_SLUG_PREFIX: &str = "spec-";
const GAMEPLAN: &str = "gameplan";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommentRule {
    WrittenInEnglish,
    NamesNoNumberedWorkItem,
    NamesNoSpec,
}

/// A broken rule and the byte offset in the comment text where it is broken.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Violation {
    pub(crate) rule: CommentRule,
    pub(crate) offset: usize,
}

impl CommentRule {
    const ALL: [Self; 3] = [Self::WrittenInEnglish, Self::NamesNoNumberedWorkItem, Self::NamesNoSpec];

    pub(crate) fn violations(comment_text: &str) -> Vec<Violation> {
        Self::ALL
            .into_iter()
            .filter_map(|rule| {
                rule.first_violation(comment_text)
                    .map(|offset| Violation { rule, offset })
            })
            .collect()
    }

    pub(crate) fn requirement(self) -> &'static str {
        match self {
            Self::WrittenInEnglish => "comments are written in English",
            Self::NamesNoNumberedWorkItem => "comments do not name a phase, task or step by number",
            Self::NamesNoSpec => "comments do not reference the spec or the gameplan",
        }
    }

    fn first_violation(self, comment_text: &str) -> Option<usize> {
        let lowercase_text = comment_text.to_ascii_lowercase();
        match self {
            Self::WrittenInEnglish => comment_text
                .char_indices()
                .find(|(_, character)| CYRILLIC.contains(character))
                .map(|(offset, _)| offset),
            Self::NamesNoNumberedWorkItem => NUMBERED_WORK_ITEMS
                .into_iter()
                .filter_map(|work_item| numbered_mention(&lowercase_text, work_item))
                .min(),
            Self::NamesNoSpec => [spec_slug(&lowercase_text), lowercase_text.find(GAMEPLAN)]
                .into_iter()
                .flatten()
                .min(),
        }
    }
}

fn numbered_mention(lowercase_text: &str, work_item: &str) -> Option<usize> {
    word_starts(lowercase_text, work_item).find(|offset| {
        let after_word = &lowercase_text[offset + work_item.len()..];
        let after_plural = after_word.strip_prefix('s').unwrap_or(after_word);
        after_plural
            .trim_start_matches(NUMBER_SEPARATORS)
            .starts_with(|character: char| character.is_ascii_digit())
    })
}

fn spec_slug(lowercase_text: &str) -> Option<usize> {
    word_starts(lowercase_text, SPEC_SLUG_PREFIX).find(|offset| {
        lowercase_text[offset + SPEC_SLUG_PREFIX.len()..]
            .starts_with(|character: char| character.is_ascii_alphanumeric())
    })
}

fn word_starts<'a>(text: &'a str, word: &'a str) -> impl Iterator<Item = usize> + 'a {
    text.match_indices(word)
        .map(|(offset, _)| offset)
        .filter(|offset| !text[..*offset].ends_with(|character: char| character.is_alphanumeric()))
}
