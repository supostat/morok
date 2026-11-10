use morok_style::StyleSheet;

use crate::cascade;
use crate::class_table::ClassTable;
use crate::compiled_tokens::CompiledTokens;
use crate::diagnostics::Diagnostics;
use crate::keyframes_rules::KeyframesRules;
use crate::keyframes_table::KeyframesTable;
use crate::source_file::SourceFile;
use crate::stylesheet_reader::{self, ReadRules};
use crate::value_context::ValueContext;

/// Compiles component stylesheets against compiled tokens into one style sheet.
///
/// The order of `sources` is the source order of the cascade and the order classes and
/// keyframes get their IDs in.
pub fn compile_stylesheets(sources: &[SourceFile], tokens: &CompiledTokens) -> Result<StyleSheet, Diagnostics> {
    let keyframes_table = KeyframesTable::of(sources);
    let context = ValueContext {
        tokens,
        keyframes: &keyframes_table,
    };
    let mut diagnostics = Vec::new();
    let mut classes = ClassTable::default();
    let mut rules = Vec::new();
    let mut keyframes = KeyframesRules::new(&keyframes_table);
    let mut read_rules = ReadRules {
        classes: &mut classes,
        rules: &mut rules,
        keyframes: &mut keyframes,
        diagnostics: &mut diagnostics,
    };
    for source in sources {
        stylesheet_reader::read(source, &context, &mut read_rules);
    }
    if diagnostics.is_empty() {
        Ok(cascade::style_sheet(
            rules,
            classes.into_names(),
            keyframes.into_rules(),
        ))
    } else {
        Err(Diagnostics::new(diagnostics))
    }
}
