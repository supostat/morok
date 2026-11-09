use std::borrow::Cow;

use cssparser::{Parser, Token};
use morok_style::{CompoundSelector, Selector, States};

use crate::class_table::ClassTable;
use crate::problem::Problem;
use crate::span::Span;

const RESERVED_CLASS_NAMES: [&str; 3] = ["self", "super", "crate"];
const OWN_CLASS_HINT: &str = "give the element its own class";

/// A selector with the number of classes and pseudo-classes it names.
#[derive(Debug)]
pub(crate) struct ParsedSelector {
    pub(crate) selector: Selector,
    pub(crate) specificity: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Combinator {
    Child,
    Descendant,
}

#[derive(Default)]
struct CompoundDraft {
    class_names: Vec<String>,
    states: States,
    pseudo_classes: u32,
}

#[derive(Default)]
struct SelectorDraft {
    compounds: Vec<CompoundDraft>,
    combinators: Vec<Combinator>,
    span: Option<Span>,
}

/// Reads the comma-separated selectors of a rule prelude and registers the classes they name.
pub(crate) fn parse(text: &str, prelude: Span, classes: &mut ClassTable) -> Result<Vec<ParsedSelector>, Problem> {
    let drafts = Reader::new(text, prelude).drafts()?;
    let selectors = drafts
        .iter()
        .map(|draft| draft.checked())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(selectors.into_iter().map(|draft| draft.selector(classes)).collect())
}

struct Reader<'i> {
    parser: Parser<'i>,
    prelude_start: usize,
    drafts: Vec<SelectorDraft>,
    selector: SelectorDraft,
    compound: Option<CompoundDraft>,
    /// The combinator read since the last compound ended.
    combinator: Option<Combinator>,
}

impl<'i> Reader<'i> {
    fn new(text: &'i str, prelude: Span) -> Self {
        Self {
            parser: Parser::new(&text[prelude.start..prelude.end]),
            prelude_start: prelude.start,
            drafts: Vec::new(),
            selector: SelectorDraft::default(),
            compound: None,
            combinator: None,
        }
    }

    fn offset(&self) -> usize {
        self.prelude_start + self.parser.position().byte_index()
    }

    fn drafts(mut self) -> Result<Vec<SelectorDraft>, Problem> {
        loop {
            let start = self.offset();
            let Some(token) = self.parser.next_including_whitespace().ok().cloned() else {
                break;
            };
            match token {
                Token::WhiteSpace(_) => {
                    if self.end_compound() {
                        self.combinator = Some(Combinator::Descendant);
                    }
                    continue;
                }
                Token::Comma => {
                    self.end_selector(start)?;
                    continue;
                }
                Token::Delim('>') => {
                    self.end_compound();
                    if self.selector.compounds.is_empty() {
                        return Err(self.problem(start, "a selector cannot start with `>`", OWN_CLASS_HINT));
                    }
                    self.combinator = Some(Combinator::Child);
                }
                Token::Delim('.') => {
                    let class_name = self.class_name(start)?;
                    self.compound_mut().class_names.push(class_name);
                }
                Token::Colon => self.pseudo(start)?,
                other => return Err(self.unsupported(start, &other)),
            }
            self.selector.span = Some(Span {
                start: self.selector.span.map_or(start, |span| span.start),
                end: self.offset(),
            });
        }
        let end = self.offset();
        self.end_selector(end)?;
        Ok(self.drafts)
    }

    /// The compound being read, started now if the previous one has ended.
    fn compound_mut(&mut self) -> &mut CompoundDraft {
        if self.compound.is_none()
            && let Some(combinator) = self.combinator.take()
        {
            self.selector.combinators.push(combinator);
        }
        self.compound.get_or_insert_with(CompoundDraft::default)
    }

    fn end_compound(&mut self) -> bool {
        match self.compound.take() {
            Some(compound) => {
                self.selector.compounds.push(compound);
                true
            }
            None => false,
        }
    }

    fn end_selector(&mut self, position: usize) -> Result<(), Problem> {
        self.end_compound();
        if self.combinator.take() == Some(Combinator::Child) {
            return Err(self.problem(position, "a selector cannot end with `>`", OWN_CLASS_HINT));
        }
        let selector = std::mem::take(&mut self.selector);
        if selector.compounds.is_empty() {
            return Err(self.problem(
                position,
                "expected a class selector",
                "a rule starts with a class, for example `.button`",
            ));
        }
        self.drafts.push(selector);
        Ok(())
    }

    fn class_name(&mut self, start: usize) -> Result<String, Problem> {
        let Some(Token::Ident(name)) = self.parser.next_including_whitespace().ok().cloned() else {
            return Err(self.problem(start, "expected a class name after `.`", "write it as `.button`"));
        };
        if !is_lowercase_kebab_case(&name) {
            return Err(self.problem(
                start,
                format!("`.{name}` is not a lowercase kebab-case class name"),
                "use lowercase letters, digits and single hyphens, starting with a letter",
            ));
        }
        if RESERVED_CLASS_NAMES.contains(&name.as_ref()) {
            return Err(self.problem(
                start,
                format!("`.{name}` cannot name a class"),
                "the name is reserved in Rust: rename the class",
            ));
        }
        Ok(name.as_ref().to_owned())
    }

    fn pseudo(&mut self, start: usize) -> Result<(), Problem> {
        match self.parser.next_including_whitespace().ok().cloned() {
            Some(Token::Ident(name)) => {
                let Some((_, state)) = States::PSEUDO_CLASSES
                    .into_iter()
                    .find(|(pseudo_class, _)| name.eq_ignore_ascii_case(pseudo_class))
                else {
                    return Err(self.problem(
                        start,
                        format!("the pseudo-class `:{name}` is not supported"),
                        "states are :hover, :active, :focus, :focus-visible, :focus-within, :disabled and :checked",
                    ));
                };
                let compound = self.compound_mut();
                compound.states = compound.states.union(state);
                compound.pseudo_classes += 1;
                Ok(())
            }
            Some(Token::Function(name)) => {
                let _ = self.parser.parse_nested_block(|arguments| {
                    while arguments.next().is_ok() {}
                    Ok::<(), cssparser::ParseError<()>>(())
                });
                Err(self.problem(start, format!("`:{name}()` is not supported"), OWN_CLASS_HINT))
            }
            Some(Token::Colon) => {
                let name = match self.parser.next_including_whitespace().ok().cloned() {
                    Some(Token::Ident(name)) => name.as_ref().to_owned(),
                    _ => String::new(),
                };
                Err(self.problem(
                    start,
                    format!("the pseudo-element `::{name}` is not supported"),
                    "draw it as an element of its own",
                ))
            }
            _ => Err(self.problem(start, "expected a pseudo-class after `:`", "write it as `:hover`")),
        }
    }

    fn unsupported(&mut self, start: usize, token: &Token<'_>) -> Problem {
        let (message, hint) = match token {
            Token::Delim('*') => ("the universal selector `*` is not supported".to_owned(), OWN_CLASS_HINT),
            Token::Ident(name) => (
                format!("type selectors are not supported: `{name}`"),
                "select the element by a class",
            ),
            Token::Hash(_) | Token::IDHash(_) => (
                "ID selectors are not supported".to_owned(),
                "select the element by a class",
            ),
            Token::SquareBracketBlock => (
                "attribute selectors are not supported".to_owned(),
                "express the attribute as a class or a state",
            ),
            Token::Delim('+' | '~') => ("sibling combinators are not supported".to_owned(), OWN_CLASS_HINT),
            Token::Delim('&') => (
                "nested rules are not supported".to_owned(),
                "write the rule at the top level",
            ),
            _ => (
                "this is not a part of a supported selector".to_owned(),
                "a selector is made of classes, states and one `>`",
            ),
        };
        if matches!(token, Token::SquareBracketBlock) {
            let _ = self.parser.parse_nested_block(|attribute| {
                while attribute.next().is_ok() {}
                Ok::<(), cssparser::ParseError<()>>(())
            });
        }
        self.problem(start, message, hint)
    }

    fn problem(&self, start: usize, message: impl Into<String>, hint: impl Into<String>) -> Problem {
        Problem {
            span: Span {
                start,
                end: self.offset().max(start),
            },
            message: message.into(),
            hint: hint.into(),
        }
    }
}

impl SelectorDraft {
    fn checked(&self) -> Result<&Self, Problem> {
        let span = self.span.unwrap_or(Span::FILE_START);
        let problem = |message: &str, hint: String| Problem {
            span,
            message: message.to_owned(),
            hint,
        };
        if self.compounds.iter().any(|compound| compound.class_names.is_empty()) {
            return Err(problem(
                "a state needs a class to belong to",
                "write the class before the state, for example `.button:hover`".to_owned(),
            ));
        }
        if self.combinators.contains(&Combinator::Descendant) {
            return Err(problem("descendant selectors are not supported", self.own_class_hint()));
        }
        if self.combinators.len() > 1 {
            return Err(problem("only one child step is supported", self.own_class_hint()));
        }
        Ok(self)
    }

    /// Suggests a class made of the first and the last class the selector names.
    fn own_class_hint(&self) -> String {
        let first = self.compounds.first().and_then(|compound| compound.class_names.first());
        let last = self.compounds.last().and_then(|compound| compound.class_names.last());
        match (first, last) {
            (Some(first), Some(last)) => format!("{OWN_CLASS_HINT}: .{first}-{last}"),
            _ => OWN_CLASS_HINT.to_owned(),
        }
    }

    fn selector(&self, classes: &mut ClassTable) -> ParsedSelector {
        let mut compounds = self.compounds.iter().map(|compound| CompoundSelector {
            classes: Cow::Owned(compound.class_names.iter().map(|name| classes.class(name)).collect()),
            states: compound.states,
        });
        let first = compounds.next();
        let second = compounds.next();
        let (parent, subject) = match (first, second) {
            (Some(parent), Some(subject)) => (Some(parent), subject),
            (Some(subject), None) => (None, subject),
            (None, _) => unreachable!("a checked selector has a compound"),
        };
        ParsedSelector {
            selector: Selector { subject, parent },
            specificity: self
                .compounds
                .iter()
                .map(|compound| compound.class_names.len() as u32 + compound.pseudo_classes)
                .sum(),
        }
    }
}

fn is_lowercase_kebab_case(name: &str) -> bool {
    name.starts_with(|first: char| first.is_ascii_lowercase())
        && name.split('-').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}
