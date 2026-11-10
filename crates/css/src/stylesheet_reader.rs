use cssparser::{ParseError, Parser};

use crate::cascade::SourceRule;
use crate::class_table::ClassTable;
use crate::declaration_reader::{self, DeclarationBlock};
use crate::diagnostic::Diagnostic;
use crate::keyframes_rules::KeyframesRules;
use crate::keyframes_syntax;
use crate::problem::Problem;
use crate::rule_prelude::{self, RuleEnd};
use crate::selector_syntax::{self, ParsedSelector};
use crate::source_file::SourceFile;
use crate::span::Span;
use crate::value_context::ValueContext;

const AT_RULE_PREFIX: char = '@';

/// What the rules of the stylesheets compiled together are read into.
#[derive(Debug)]
pub(crate) struct ReadRules<'a> {
    pub(crate) classes: &'a mut ClassTable,
    pub(crate) rules: &'a mut Vec<SourceRule>,
    pub(crate) keyframes: &'a mut KeyframesRules,
    pub(crate) diagnostics: &'a mut Vec<Diagnostic>,
}

/// Reads the class rules and the `@keyframes` rules of one component stylesheet, reporting
/// everything outside the subset.
pub(crate) fn read(source: &SourceFile, context: &ValueContext<'_>, read_rules: &mut ReadRules<'_>) {
    let text = source.text();
    let mut input = Parser::new(text);
    loop {
        let (prelude, rule_end) = rule_prelude::read(&mut input, text);
        let prelude_text = &text[prelude.start..prelude.end];
        if let Some(name) = keyframes_syntax::name(text, prelude) {
            match name {
                Ok(name) if rule_end == RuleEnd::Block => {
                    read_keyframes(source, context, &name, &mut input, read_rules);
                }
                Ok(name) => read_rules.diagnostics.push(Diagnostic::new(
                    source,
                    name.span,
                    format!("`@keyframes {}` has no block", name.text),
                    "write its keyframes in braces: `@keyframes fade { from { opacity: 0; } }`",
                )),
                Err(problem) => {
                    read_rules
                        .diagnostics
                        .push(Diagnostic::new(source, problem.span, problem.message, problem.hint));
                }
            }
        } else if prelude_text.starts_with(AT_RULE_PREFIX) {
            read_rules.diagnostics.push(at_rule(source, prelude));
        } else if rule_end == RuleEnd::Block {
            let selectors = selector_syntax::parse(text, prelude, read_rules.classes)
                .and_then(|selectors| Ok((declaration_block(&selectors, prelude)?, selectors)));
            let block = selectors.as_ref().map_or(DeclarationBlock::Rule, |(block, _)| *block);
            let declarations = input
                .parse_nested_block(|declarations| {
                    Ok::<_, ParseError<()>>(declaration_reader::read(
                        source,
                        context,
                        block,
                        declarations,
                        read_rules.diagnostics,
                    ))
                })
                .expect("a rule block is read to its end");
            match selectors {
                Ok((_, selectors)) => read_rules.rules.extend(selectors.into_iter().map(|parsed| SourceRule {
                    selector: parsed.selector,
                    specificity: parsed.specificity,
                    declarations: declarations.clone(),
                })),
                Err(problem) => {
                    read_rules
                        .diagnostics
                        .push(Diagnostic::new(source, problem.span, problem.message, problem.hint));
                }
            }
        } else if !prelude.is_empty() {
            read_rules.diagnostics.push(Diagnostic::new(
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

/// What the block of a rule with these selectors sets: the properties of an element, or those
/// of the one pseudo-element all its selectors name. A block is read once for every selector
/// of its rule, so the selectors of a pseudo-element do not share a rule with others.
fn declaration_block(selectors: &[ParsedSelector], prelude: Span) -> Result<DeclarationBlock, Problem> {
    let (first, rest) = selectors.split_first().expect("a rule that was read has a selector");
    if rest
        .iter()
        .any(|selector| selector.pseudo_element != first.pseudo_element)
    {
        return Err(Problem {
            span: prelude,
            message: "a rule of a pseudo-element is shared with other selectors".to_owned(),
            hint: "write the rule of each pseudo-element apart from the rules of elements".to_owned(),
        });
    }
    Ok(first
        .pseudo_element
        .map_or(DeclarationBlock::Rule, DeclarationBlock::PseudoElement))
}

/// Reads the keyframes in the block of a `@keyframes` rule, which is open.
fn read_keyframes(
    source: &SourceFile,
    context: &ValueContext<'_>,
    name: &keyframes_syntax::KeyframesName,
    input: &mut Parser<'_>,
    read_rules: &mut ReadRules<'_>,
) {
    let keyframes = context
        .keyframes
        .id(&name.text)
        .expect("the keyframes table holds every well-formed name");
    if read_rules.keyframes.is_declared(keyframes) {
        read_rules.diagnostics.push(Diagnostic::new(
            source,
            name.span,
            format!("`@keyframes {}` is declared twice", name.text),
            "give one of them another name",
        ));
        return;
    }
    read_rules.keyframes.declare(keyframes, &name.text);
    let text = source.text();
    input
        .parse_nested_block(|block| {
            loop {
                let (prelude, rule_end) = rule_prelude::read(block, text);
                if rule_end == RuleEnd::Block {
                    let offsets = keyframes_syntax::offsets(text, prelude);
                    let declarations = block
                        .parse_nested_block(|declarations| {
                            Ok::<_, ParseError<()>>(declaration_reader::read(
                                source,
                                context,
                                DeclarationBlock::Keyframe,
                                declarations,
                                read_rules.diagnostics,
                            ))
                        })
                        .expect("a keyframe block is read to its end");
                    match offsets {
                        Ok(offsets) => {
                            for offset in offsets {
                                read_rules
                                    .keyframes
                                    .add_keyframe(keyframes, offset, declarations.clone());
                            }
                        }
                        Err(problem) => read_rules.diagnostics.push(Diagnostic::new(
                            source,
                            problem.span,
                            problem.message,
                            problem.hint,
                        )),
                    }
                } else if !prelude.is_empty() {
                    read_rules.diagnostics.push(Diagnostic::new(
                        source,
                        prelude,
                        "expected a keyframe",
                        "write it as `50% { property: value; }`",
                    ));
                }
                if rule_end == RuleEnd::EndOfInput {
                    return Ok::<_, ParseError<()>>(());
                }
            }
        })
        .expect("a keyframes block is read to its end");
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
        "a component stylesheet holds class rules and `@keyframes` rules only",
    )
}
