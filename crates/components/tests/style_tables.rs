use morok_components::styles::{CLASSES, STYLE_SHEET};
use morok_components::tokens::{DARK, LIGHT};
use morok_components::{STYLESHEETS, StylesheetSource};
use morok_css::{SourceFile, compile_stylesheets, compile_tokens};

const TOKENS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/system/tokens.css");

fn kit_stylesheets() -> Vec<SourceFile> {
    STYLESHEETS
        .iter()
        .map(|StylesheetSource { path, text }| SourceFile::new(*path, *text))
        .collect()
}

#[test]
fn the_listed_stylesheets_compile_to_the_tables_of_the_build() {
    let tokens = compile_tokens(&SourceFile::read(TOKENS_FILE).expect("the tokens of the kit are readable"))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the kit compile:\n{diagnostics}"));

    let compiled = compile_stylesheets(&kit_stylesheets(), &tokens)
        .unwrap_or_else(|diagnostics| panic!("the stylesheets of the kit compile:\n{diagnostics}"));

    assert!(
        compiled == STYLE_SHEET,
        "`STYLESHEETS` lists every stylesheet under `src`, in the order of their paths"
    );
    assert!(tokens.light == LIGHT && tokens.dark == DARK);
}

#[test]
fn a_host_that_compiles_its_stylesheets_after_those_of_the_kit_keeps_the_classes_of_the_kit() {
    let tokens = compile_tokens(&SourceFile::read(TOKENS_FILE).expect("the tokens of the kit are readable"))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the kit compile:\n{diagnostics}"));
    let mut stylesheets = kit_stylesheets();
    stylesheets.push(SourceFile::new(
        "host.css",
        ".toolbar { display: flex; } .toolbar > .button { flex: none; }",
    ));

    let compiled = compile_stylesheets(&stylesheets, &tokens)
        .unwrap_or_else(|diagnostics| panic!("the stylesheets compile:\n{diagnostics}"));

    assert_eq!(compiled.class_named("button"), Some(CLASSES.button));
    assert_eq!(compiled.class_named("primary"), Some(CLASSES.primary));
    assert_eq!(
        compiled.class_named("toolbar").map(morok_style::Class::index),
        Some(STYLE_SHEET.classes.len())
    );
}
