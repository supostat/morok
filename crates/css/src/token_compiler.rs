use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use morok_style::ColorScheme;

use crate::compiled_tokens::CompiledTokens;
use crate::diagnostic::Diagnostic;
use crate::diagnostics::Diagnostics;
use crate::source_file::SourceFile;
use crate::span::Span;
use crate::theme_builder::ThemeBuilder;
use crate::token_declaration::TokenDeclaration;
use crate::token_entry::TokenEntry;
use crate::token_name::TokenName;
use crate::token_value::TokenValue;
use crate::tokens_stylesheet;

/// A token with a value of the same kind in both schemes.
struct TokenPair {
    name: TokenName,
    light: TokenValue,
    dark: TokenValue,
}

/// Compiles a tokens file into typed token IDs and the light and dark theme tables.
pub fn compile_tokens(source: &SourceFile) -> Result<CompiledTokens, Diagnostics> {
    let mut diagnostics = Vec::new();
    let stylesheet = tokens_stylesheet::parse(source, &mut diagnostics);
    let (Some(light), Some(dark)) = (stylesheet.light, stylesheet.dark) else {
        return Err(Diagnostics::new(diagnostics));
    };
    let light = first_declarations(source, ColorScheme::Light, light, &mut diagnostics);
    let dark = first_declarations(source, ColorScheme::Dark, dark, &mut diagnostics);
    let pairs = paired(source, &light, &dark, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(tables(pairs))
    } else {
        Err(Diagnostics::new(diagnostics))
    }
}

fn first_declarations(
    source: &SourceFile,
    scheme: ColorScheme,
    declarations: Vec<TokenDeclaration>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<TokenDeclaration> {
    let mut first_spans: HashMap<TokenName, Span> = HashMap::new();
    let mut first_declarations = Vec::new();
    for declaration in declarations {
        match first_spans.entry(declaration.name.clone()) {
            Entry::Occupied(first_span) => diagnostics.push(Diagnostic::new(
                source,
                declaration.name_span,
                format!("`{}` is declared twice in the {scheme} scheme", declaration.name),
                format!(
                    "its first declaration is on line {}",
                    source.line_number(first_span.get().start)
                ),
            )),
            Entry::Vacant(unseen) => {
                unseen.insert(declaration.name_span);
                first_declarations.push(declaration);
            }
        }
    }
    first_declarations
}

fn paired(
    source: &SourceFile,
    light: &[TokenDeclaration],
    dark: &[TokenDeclaration],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<TokenPair> {
    let dark_by_name: HashMap<&TokenName, &TokenDeclaration> = dark
        .iter()
        .map(|declaration| (&declaration.name, declaration))
        .collect();
    let light_names: HashSet<&TokenName> = light.iter().map(|declaration| &declaration.name).collect();
    let mut pairs = Vec::new();
    for light_declaration in light {
        let Some(dark_declaration) = dark_by_name.get(&light_declaration.name) else {
            diagnostics.push(defined_for_one_scheme(source, light_declaration, ColorScheme::Light));
            continue;
        };
        let (Some(light_value), Some(dark_value)) = (&light_declaration.value, &dark_declaration.value) else {
            continue;
        };
        if light_value.kind() == dark_value.kind() {
            pairs.push(TokenPair {
                name: light_declaration.name.clone(),
                light: light_value.clone(),
                dark: dark_value.clone(),
            });
        } else {
            diagnostics.push(Diagnostic::new(
                source,
                dark_declaration.name_span,
                format!(
                    "`{}` is {} in the light scheme and {} in the dark scheme",
                    dark_declaration.name,
                    light_value.kind().description(),
                    dark_value.kind().description()
                ),
                "a token holds one kind of value in both schemes",
            ));
        }
    }
    for dark_declaration in dark {
        if !light_names.contains(&dark_declaration.name) {
            diagnostics.push(defined_for_one_scheme(source, dark_declaration, ColorScheme::Dark));
        }
    }
    pairs
}

fn defined_for_one_scheme(source: &SourceFile, declaration: &TokenDeclaration, scheme: ColorScheme) -> Diagnostic {
    let hint = match scheme {
        ColorScheme::Light => "define it for the dark scheme in `@media (prefers-color-scheme: dark) { :root { … } }`",
        ColorScheme::Dark => "define it for the light scheme in the top-level `:root` block",
    };
    Diagnostic::new(
        source,
        declaration.name_span,
        format!("`{}` is defined for the {scheme} scheme only", declaration.name),
        hint,
    )
}

fn tables(pairs: Vec<TokenPair>) -> CompiledTokens {
    let mut light = ThemeBuilder::default();
    let mut dark = ThemeBuilder::default();
    let entries = pairs
        .into_iter()
        .map(|pair| {
            let id = light.push(pair.light);
            let dark_id = dark.push(pair.dark);
            debug_assert_eq!(id, dark_id, "both schemes hold a value of the same kind");
            TokenEntry {
                name: pair.name.into_string(),
                id,
            }
        })
        .collect();
    CompiledTokens {
        entries,
        light: light.build(),
        dark: dark.build(),
    }
}
