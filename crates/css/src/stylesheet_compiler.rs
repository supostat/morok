use morok_style::StyleSheet;

use crate::cascade;
use crate::class_table::ClassTable;
use crate::compiled_tokens::CompiledTokens;
use crate::diagnostics::Diagnostics;
use crate::source_file::SourceFile;
use crate::stylesheet_reader;

/// Compiles component stylesheets against compiled tokens into one style sheet.
///
/// The order of `sources` is the source order of the cascade and the order classes get their IDs in.
pub fn compile_stylesheets(sources: &[SourceFile], tokens: &CompiledTokens) -> Result<StyleSheet, Diagnostics> {
    let mut diagnostics = Vec::new();
    let mut classes = ClassTable::default();
    let mut rules = Vec::new();
    for source in sources {
        stylesheet_reader::read(source, tokens, &mut classes, &mut rules, &mut diagnostics);
    }
    if diagnostics.is_empty() {
        Ok(cascade::style_sheet(rules, classes.into_names()))
    } else {
        Err(Diagnostics::new(diagnostics))
    }
}
