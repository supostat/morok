use cssparser::{Delimiter, ParseError, Parser, Token};
use morok_style::ColorScheme;

use crate::diagnostic::Diagnostic;
use crate::rule_prelude::{self, RuleEnd};
use crate::source_file::SourceFile;
use crate::span::Span;
use crate::token_declaration::TokenDeclaration;
use crate::token_family::TokenFamily;
use crate::token_name::TokenName;
use crate::token_name_error::TokenNameError;
use crate::token_value::TokenValue;

const ROOT_SELECTOR: &str = ":root";
const DARK_MEDIA_QUERY: &str = "@media(prefers-color-scheme:dark)";

/// The token blocks of a tokens file; a block the file does not have is `None` and already reported.
pub(crate) struct TokensStylesheet {
    pub(crate) light: Option<Vec<TokenDeclaration>>,
    pub(crate) dark: Option<Vec<TokenDeclaration>>,
}

pub(crate) fn parse(source: &SourceFile, diagnostics: &mut Vec<Diagnostic>) -> TokensStylesheet {
    let mut reader = Reader {
        source,
        diagnostics,
        light: None,
        dark: None,
    };
    reader.rules(&mut Parser::new(source.text()), Nesting::TopLevel);
    reader.report_missing_blocks();
    TokensStylesheet {
        light: reader.light,
        dark: reader.dark,
    }
}

#[derive(Clone, Copy)]
enum Nesting {
    TopLevel,
    DarkMedia,
}

struct Reader<'a> {
    source: &'a SourceFile,
    diagnostics: &'a mut Vec<Diagnostic>,
    light: Option<Vec<TokenDeclaration>>,
    dark: Option<Vec<TokenDeclaration>>,
}

impl Reader<'_> {
    fn rules(&mut self, input: &mut Parser<'_>, nesting: Nesting) {
        loop {
            let (prelude, rule_end) = rule_prelude::read(input, self.source.text());
            match rule_end {
                RuleEnd::Block => self.block(input, prelude, nesting),
                RuleEnd::Semicolon | RuleEnd::EndOfInput if prelude.is_empty() => {}
                RuleEnd::Semicolon | RuleEnd::EndOfInput => self.report_foreign_rule(prelude),
            }
            if rule_end == RuleEnd::EndOfInput {
                return;
            }
        }
    }

    fn block(&mut self, input: &mut Parser<'_>, prelude: Span, nesting: Nesting) {
        let source = self.source;
        let prelude_text = &source.text()[prelude.start..prelude.end];
        match nesting {
            Nesting::TopLevel if prelude_text == ROOT_SELECTOR => self.tokens_block(input, prelude, ColorScheme::Light),
            Nesting::DarkMedia if prelude_text == ROOT_SELECTOR => self.tokens_block(input, prelude, ColorScheme::Dark),
            Nesting::TopLevel if is_dark_media_query(prelude_text) => input
                .parse_nested_block(|media_block| {
                    self.rules(media_block, Nesting::DarkMedia);
                    Ok::<(), ParseError<()>>(())
                })
                .expect("a media block is read to its end"),
            _ => self.report_foreign_rule(prelude),
        }
    }

    fn tokens_block(&mut self, input: &mut Parser<'_>, prelude: Span, scheme: ColorScheme) {
        let already_declared = match scheme {
            ColorScheme::Light => self.light.is_some(),
            ColorScheme::Dark => self.dark.is_some(),
        };
        if already_declared {
            self.report(
                prelude,
                format!("the {scheme} scheme is declared twice"),
                "declare every token of a scheme in one block",
            );
            return;
        }
        let declarations = input
            .parse_nested_block(|tokens_block| Ok::<_, ParseError<()>>(self.declarations(tokens_block)))
            .expect("a tokens block is read to its end");
        match scheme {
            ColorScheme::Light => self.light = Some(declarations),
            ColorScheme::Dark => self.dark = Some(declarations),
        }
    }

    fn declarations(&mut self, input: &mut Parser<'_>) -> Vec<TokenDeclaration> {
        let mut declarations = Vec::new();
        loop {
            input.skip_whitespace();
            let start = input.position();
            let Some(token) = input.next().ok().cloned() else {
                return declarations;
            };
            let span = Span::between(start, input.position());
            match token {
                Token::Semicolon => {}
                Token::Ident(property) if TokenName::is_custom_property(&property) => {
                    declarations.extend(self.declaration(input, &property, span));
                }
                _ => {
                    self.report(
                        span,
                        "a tokens block holds custom properties only",
                        "declare a token as `--<family>-<name>: <value>;`",
                    );
                    skip_to_declaration_end(input);
                }
            }
        }
    }

    fn declaration(&mut self, input: &mut Parser<'_>, property: &str, name_span: Span) -> Option<TokenDeclaration> {
        let name = match TokenName::parse(property) {
            Ok(name) => Some(name),
            Err(error) => {
                self.report_name(&error, property, name_span);
                None
            }
        };
        if input.expect_colon().is_err() {
            self.report(
                name_span,
                format!("`{property}` has no value"),
                format!("write `{property}: <value>;`"),
            );
            skip_to_declaration_end(input);
            return None;
        }
        let kind = name.as_ref().map(TokenName::kind);
        let text = self.source.text();
        let (value_span, value) = input
            .parse_until_before(Delimiter::Semicolon, |value_input| {
                value_input.skip_whitespace();
                let value_start = value_input.state();
                skip_to_end(value_input);
                let span =
                    Span::between(value_start.position(), value_input.position()).without_surrounding_whitespace(text);
                value_input.reset(&value_start);
                let value = kind.and_then(|kind| {
                    value_input
                        .parse_entirely(|entire_value| TokenValue::parse(kind, entire_value))
                        .ok()
                });
                skip_to_end(value_input);
                Ok::<_, ParseError<()>>((span, value))
            })
            .expect("a declaration value is read to its end");
        let name = name?;
        if value.is_none() {
            self.report(
                value_span,
                format!("`{name}` expects {}", name.kind().expectation()),
                format!("for example `{}`", name.kind().example()),
            );
        }
        Some(TokenDeclaration { name, name_span, value })
    }

    fn report_name(&mut self, error: &TokenNameError, property: &str, span: Span) {
        let (message, hint) = match error {
            TokenNameError::NotLowercaseKebabCase => (
                format!("`{property}` is not lowercase kebab-case"),
                "use lowercase letters, digits and single hyphens".to_owned(),
            ),
            TokenNameError::UnknownFamily { family } => (
                format!("unknown token family `{family}`"),
                format!("token families are {}", TokenFamily::PREFIXES),
            ),
            TokenNameError::NameMissing => (
                format!("`{property}` has a family but no name"),
                format!("name the token, for example `{property}-1`"),
            ),
        };
        self.report(span, message, hint);
    }

    fn report_foreign_rule(&mut self, prelude: Span) {
        self.report(
            prelude,
            "a tokens file holds only `:root` and `@media (prefers-color-scheme: dark)` blocks",
            "declare tokens in `:root` and keep every other rule out of the tokens file",
        );
    }

    fn report_missing_blocks(&mut self) {
        if self.light.is_none() {
            self.report(
                Span::FILE_START,
                "the tokens file has no light scheme",
                "declare it in a top-level `:root { … }` block",
            );
        }
        if self.dark.is_none() {
            self.report(
                Span::FILE_START,
                "the tokens file has no dark scheme",
                "declare it in `@media (prefers-color-scheme: dark) { :root { … } }`",
            );
        }
    }

    fn report(&mut self, span: Span, message: impl Into<String>, hint: impl Into<String>) {
        self.diagnostics.push(Diagnostic::new(self.source, span, message, hint));
    }
}

fn is_dark_media_query(prelude: &str) -> bool {
    let without_whitespace: String = prelude.chars().filter(|character| !character.is_whitespace()).collect();
    without_whitespace.eq_ignore_ascii_case(DARK_MEDIA_QUERY)
}

fn skip_to_end(input: &mut Parser<'_>) {
    while input.next().is_ok() {}
}

fn skip_to_declaration_end(input: &mut Parser<'_>) {
    input
        .parse_until_before(Delimiter::Semicolon, |rest| {
            skip_to_end(rest);
            Ok::<(), ParseError<()>>(())
        })
        .expect("the rest of a declaration is read to its end");
}
