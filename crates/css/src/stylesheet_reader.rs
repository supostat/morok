use cssparser::{ParseError, Parser};

use crate::cascade::SourceRule;
use crate::class_table::ClassTable;
use crate::compiled_tokens::CompiledTokens;
use crate::declaration_reader;
use crate::diagnostic::Diagnostic;
use crate::rule_prelude::{self, RuleEnd};
use crate::selector_syntax;
use crate::source_file::SourceFile;
use crate::span::Span;

const AT_RULE_PREFIX: char = '@';

/// Reads the class rules of one component stylesheet, reporting everything outside the subset.
pub(crate) fn read(
    source: &SourceFile,
    tokens: &CompiledTokens,
    classes: &mut ClassTable,
    rules: &mut Vec<SourceRule>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let text = source.text();
    let mut input = Parser::new(text);
    loop {
        let (prelude, rule_end) = rule_prelude::read(&mut input, text);
        let prelude_text = &text[prelude.start..prelude.end];
        if prelude_text.starts_with(AT_RULE_PREFIX) {
            diagnostics.push(at_rule(source, prelude));
        } else if rule_end == RuleEnd::Block {
            let selectors = selector_syntax::parse(text, prelude, classes);
            let declarations = input
                .parse_nested_block(|block| {
                    Ok::<_, ParseError<()>>(declaration_reader::read(source, tokens, block, diagnostics))
                })
                .expect("a rule block is read to its end");
            match selectors {
                Ok(selectors) => rules.extend(selectors.into_iter().map(|parsed| SourceRule {
                    selector: parsed.selector,
                    specificity: parsed.specificity,
                    declarations: declarations.clone(),
                })),
                Err(problem) => diagnostics.push(Diagnostic::new(source, problem.span, problem.message, problem.hint)),
            }
        } else if !prelude.is_empty() {
            diagnostics.push(Diagnostic::new(
                source,
                prelude,
                "expected a rule",
                "write it as `.class { property: value; }`",
            ));
        }
        if rule_end == RuleEnd::EndOfInput {
            return;
        }
    }
}

fn at_rule(source: &SourceFile, prelude: Span) -> Diagnostic {
    let prelude_text = &source.text()[prelude.start..prelude.end];
    let keyword_length = prelude_text
        .find(|character: char| character.is_whitespace() || character == '(')
        .unwrap_or(prelude_text.len());
    let keyword = Span {
        start: prelude.start,
        end: prelude.start + keyword_length,
    };
    Diagnostic::new(
        source,
        keyword,
        format!("`{}` is not supported", &prelude_text[..keyword_length]),
        "a component stylesheet holds class rules only",
    )
}
