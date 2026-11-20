use morok::components::StylesheetSource;
use morok_catalog::styles::STYLE_SHEET;
use morok_catalog::{EMBER_TOKENS_STYLESHEET, STYLESHEETS, TOKENS_STYLESHEET, ember, tokens};
use morok_components::{KIT_TOKENS, tokens as kit_tokens};
use morok_css::{
    CompiledTokens, SourceFile, compile_alternate_tokens, compile_application_tokens, compile_stylesheets,
    compile_tokens,
};

fn source_file(source: &StylesheetSource) -> SourceFile {
    SourceFile::new(source.path, source.text)
}

fn catalog_tokens() -> CompiledTokens {
    let kit = compile_tokens(&source_file(&KIT_TOKENS))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the kit compile:\n{diagnostics}"));
    compile_application_tokens(&kit, &source_file(&TOKENS_STYLESHEET))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the catalog compile:\n{diagnostics}"))
}

#[test]
fn the_listed_stylesheets_compile_after_those_of_the_kit_to_the_tables_of_the_build() {
    let tokens = catalog_tokens();
    let stylesheets: Vec<SourceFile> = morok_components::STYLESHEETS
        .iter()
        .chain(&STYLESHEETS)
        .map(source_file)
        .collect();

    let compiled = compile_stylesheets(&stylesheets, &tokens)
        .unwrap_or_else(|diagnostics| panic!("the stylesheets compile:\n{diagnostics}"));

    assert!(
        compiled == STYLE_SHEET,
        "`STYLESHEETS` lists every stylesheet of the catalog, in the order of their paths"
    );
    assert!(tokens.light == tokens::LIGHT && tokens.dark == tokens::DARK);
}

#[test]
fn the_tokens_of_the_kit_keep_their_ids_in_the_tokens_of_the_catalog() {
    assert_eq!(tokens::TOKENS.color_accent, kit_tokens::TOKENS.color_accent);
    assert_eq!(tokens::TOKENS.motion_ease_out, kit_tokens::TOKENS.motion_ease_out);
    assert_eq!(tokens::TOKENS.color_chart_1.index(), kit_tokens::LIGHT.colors.len());
}

#[test]
fn the_ember_tokens_compile_over_the_tokens_of_the_catalog_to_the_values_of_the_build() {
    let ember_tokens = compile_alternate_tokens(&catalog_tokens(), &source_file(&EMBER_TOKENS_STYLESHEET))
        .unwrap_or_else(|diagnostics| panic!("the ember tokens compile:\n{diagnostics}"));

    assert!(ember_tokens.light == ember::LIGHT && ember_tokens.dark == ember::DARK);
    assert!(ember::LIGHT.resolve(tokens::TOKENS.color_accent) != tokens::LIGHT.resolve(tokens::TOKENS.color_accent));
}
