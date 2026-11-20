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
    /// Where the light scheme declares the token.
    name_span: Span,
    light: TokenValue,
    dark: TokenValue,
}

/// The tokens a file declares, in its order, with the `:root` selector of its light scheme.
struct DeclaredTokens {
    pairs: Vec<TokenPair>,
    light_root: Span,
}

/// Compiles a tokens file into typed token IDs and the light and dark theme tables.
pub fn compile_tokens(source: &SourceFile) -> Result<CompiledTokens, Diagnostics> {
    declared_tokens(source).map(|declared| tables(declared.pairs))
}

/// Compiles the tokens file of an application over the tokens of the kit. The file defines
/// every token of the kit, and may define tokens of its own: a token of the kit keeps the ID
/// the kit gave it, so the tables of the kit's components read the application's values, and a
/// token of the application's own gets an ID after those of the kit, in the order of the file.
pub fn compile_application_tokens(kit: &CompiledTokens, source: &SourceFile) -> Result<CompiledTokens, Diagnostics> {
    let declared = declared_tokens(source)?;
    let mut diagnostics = Vec::new();
    report_undefined(
        source,
        kit,
        &declared,
        "the tokens file of an application defines every token of the kit",
        &mut diagnostics,
    );
    if diagnostics.is_empty() {
        Ok(tables(in_order_of(kit, declared.pairs)))
    } else {
        Err(Diagnostics::new(diagnostics))
    }
}

/// Compiles a tokens file an application switches to over its first tokens file: it defines
/// exactly the tokens of `first`, which keep their IDs, so the style sheet compiled over the
/// first file is restyled by the values of this one.
pub fn compile_alternate_tokens(first: &CompiledTokens, source: &SourceFile) -> Result<CompiledTokens, Diagnostics> {
    let declared = declared_tokens(source)?;
    let mut diagnostics = Vec::new();
    report_undefined(
        source,
        first,
        &declared,
        "a tokens file the application switches to defines every token of its first tokens file",
        &mut diagnostics,
    );
    for pair in &declared.pairs {
        if first.id(pair.name.as_str()).is_none() {
            diagnostics.push(Diagnostic::new(
                source,
                pair.name_span,
                format!("`{}` is not a token of the first tokens file", pair.name),
                "a tokens file the application switches to defines the tokens of its first one and no other",
            ));
        }
    }
    if diagnostics.is_empty() {
        Ok(tables(in_order_of(first, declared.pairs)))
    } else {
        Err(Diagnostics::new(diagnostics))
    }
}

fn declared_tokens(source: &SourceFile) -> Result<DeclaredTokens, Diagnostics> {
    let mut diagnostics = Vec::new();
    let stylesheet = tokens_stylesheet::parse(source, &mut diagnostics);
    let (Some(light), Some(dark)) = (stylesheet.light, stylesheet.dark) else {
        return Err(Diagnostics::new(diagnostics));
    };
    let light_root = light.root;
    let light = first_declarations(source, ColorScheme::Light, light.declarations, &mut diagnostics);
    let dark = first_declarations(source, ColorScheme::Dark, dark.declarations, &mut diagnostics);
    let pairs = paired(source, &light, &dark, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(DeclaredTokens { pairs, light_root })
    } else {
        Err(Diagnostics::new(diagnostics))
    }
}

/// Reports every token of `base` the file does not define, at its light `:root` block.
fn report_undefined(
    source: &SourceFile,
    base: &CompiledTokens,
    declared: &DeclaredTokens,
    hint: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let defined: HashSet<&str> = declared.pairs.iter().map(|pair| pair.name.as_str()).collect();
    for entry in &base.entries {
        if !defined.contains(entry.name.as_str()) {
            diagnostics.push(Diagnostic::new(
                source,
                declared.light_root,
                format!("`--{}` is not defined", entry.name),
                hint,
            ));
        }
    }
}

/// The pairs with the tokens of `base` first, in the order of `base`, and the others after
/// them in their order. Every token of `base` is among the pairs.
fn in_order_of(base: &CompiledTokens, pairs: Vec<TokenPair>) -> Vec<TokenPair> {
    let mut by_name: HashMap<String, TokenPair> = HashMap::with_capacity(pairs.len());
    let mut own_names = Vec::new();
    for pair in pairs {
        if base.id(pair.name.as_str()).is_none() {
            own_names.push(pair.name.as_str().to_owned());
        }
        by_name.insert(pair.name.as_str().to_owned(), pair);
    }
    base.entries
        .iter()
        .map(|entry| &entry.name)
        .chain(&own_names)
        .map(|name| by_name.remove(name).expect("every token of the base is defined"))
        .collect()
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
                name_span: light_declaration.name_span,
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
