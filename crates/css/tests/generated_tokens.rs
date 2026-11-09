use std::env;
use std::fs;

use morok_css::{CompiledTokens, SchemaPath, SourceFile, TokenId, compile_tokens, generate_tokens_source};

#[rustfmt::skip]
#[allow(dead_code, unreachable_pub)]
#[path = "golden/tokens.rs"]
mod golden;

const BASE_TOKENS_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/system/tokens.css");
const GOLDEN_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/tokens.rs");
const UPDATE_GOLDEN_VARIABLE: &str = "MOROK_UPDATE_GOLDEN";

fn base_theme() -> CompiledTokens {
    let source = SourceFile::read(BASE_TOKENS_FILE).expect("the base tokens file is readable");
    compile_tokens(&source).unwrap_or_else(|diagnostics| panic!("the base tokens compile:\n{diagnostics}"))
}

#[test]
fn the_generated_source_equals_the_golden_file() {
    let generated = generate_tokens_source(&base_theme(), SchemaPath::Crate);
    if env::var_os(UPDATE_GOLDEN_VARIABLE).is_some() {
        fs::write(GOLDEN_FILE, &generated).expect("the golden file is writable");
    }

    let golden = fs::read_to_string(GOLDEN_FILE).expect("the golden file is readable");

    assert!(
        generated == golden,
        "the generated tokens differ from {GOLDEN_FILE}; rerun with {UPDATE_GOLDEN_VARIABLE}=1 to rewrite it"
    );
}

#[test]
fn the_golden_themes_equal_the_compiled_ones() {
    let tokens = base_theme();

    assert_eq!(golden::LIGHT, tokens.light);
    assert_eq!(golden::DARK, tokens.dark);
}

#[test]
fn the_golden_token_ids_equal_the_compiled_ones() {
    let tokens = base_theme();
    let ids = golden::TOKENS;

    assert_eq!(tokens.id("color-bg"), Some(TokenId::Color(ids.color_bg)));
    assert_eq!(tokens.id("color-scrim"), Some(TokenId::Color(ids.color_scrim)));
    assert_eq!(tokens.id("space-12"), Some(TokenId::Length(ids.space_12)));
    assert_eq!(tokens.id("radius-xs"), Some(TokenId::Length(ids.radius_xs)));
    assert_eq!(tokens.id("text-2xl"), Some(TokenId::Length(ids.text_2xl)));
    assert_eq!(tokens.id("font-mono"), Some(TokenId::FontFamilies(ids.font_mono)));
    assert_eq!(tokens.id("text-leading"), Some(TokenId::Leading(ids.text_leading)));
    assert_eq!(
        tokens.id("text-weight-semibold"),
        Some(TokenId::FontWeight(ids.text_weight_semibold))
    );
    assert_eq!(tokens.id("shadow-modal"), Some(TokenId::Shadow(ids.shadow_modal)));
    assert_eq!(tokens.id("motion-slow"), Some(TokenId::Duration(ids.motion_slow)));
    assert_eq!(tokens.id("motion-ease-out"), Some(TokenId::Easing(ids.motion_ease_out)));
}

#[test]
fn the_facade_schema_path_replaces_every_crate_path() {
    let through_the_crate = generate_tokens_source(&base_theme(), SchemaPath::Crate);
    let through_the_facade = generate_tokens_source(&base_theme(), SchemaPath::Facade);

    assert!(through_the_facade.contains("pub const LIGHT: ::morok::style::Theme"));
    assert!(!through_the_facade.contains("::morok_style::"));
    assert_eq!(
        through_the_facade.matches("::morok::style::").count(),
        through_the_crate.matches("::morok_style::").count()
    );
}
