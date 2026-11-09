mod support;

use std::env;
use std::fs;

use morok_css::{SchemaPath, SourceFile, compile_stylesheets, generate_styles_source};
use morok_style::StyleSheet;
use support::base_tokens;

#[rustfmt::skip]
#[allow(dead_code, unreachable_pub)]
#[path = "golden/styles.rs"]
mod golden;

const FIXTURE_STYLESHEETS: [&str; 2] = [
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/styles/button.css"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/styles/panel.css"),
];
const GOLDEN_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/styles.rs");
const UPDATE_GOLDEN_VARIABLE: &str = "MOROK_UPDATE_GOLDEN";

fn fixture_style_sheet() -> StyleSheet {
    let stylesheets = FIXTURE_STYLESHEETS.map(|path| SourceFile::read(path).expect("the fixture is readable"));
    compile_stylesheets(&stylesheets, &base_tokens())
        .unwrap_or_else(|diagnostics| panic!("the fixtures compile:\n{diagnostics}"))
}

#[test]
fn the_generated_source_equals_the_golden_file() {
    let generated = generate_styles_source(&fixture_style_sheet(), SchemaPath::Crate);
    if env::var_os(UPDATE_GOLDEN_VARIABLE).is_some() {
        fs::write(GOLDEN_FILE, &generated).expect("the golden file is writable");
    }

    let golden = fs::read_to_string(GOLDEN_FILE).expect("the golden file is readable");

    assert!(
        generated == golden,
        "the generated styles differ from {GOLDEN_FILE}; rerun with {UPDATE_GOLDEN_VARIABLE}=1 to rewrite it"
    );
}

#[test]
fn the_golden_style_sheet_equals_the_compiled_one() {
    assert_eq!(golden::STYLE_SHEET, fixture_style_sheet());
}

#[test]
fn the_golden_class_ids_equal_the_compiled_ones() {
    let style_sheet = fixture_style_sheet();
    let classes = golden::CLASSES;

    assert_eq!(style_sheet.class_named("button"), Some(classes.button));
    assert_eq!(style_sheet.class_named("primary"), Some(classes.primary));
    assert_eq!(style_sheet.class_named("toolbar"), Some(classes.toolbar));
    assert_eq!(style_sheet.class_named("panel"), Some(classes.panel));
    assert_eq!(style_sheet.class_named("badge"), Some(classes.badge));
    assert_eq!(style_sheet.classes.len(), 5);
}

#[test]
fn the_facade_schema_path_replaces_every_crate_path() {
    let through_the_crate = generate_styles_source(&fixture_style_sheet(), SchemaPath::Crate);
    let through_the_facade = generate_styles_source(&fixture_style_sheet(), SchemaPath::Facade);

    assert!(through_the_facade.contains("pub const STYLE_SHEET: ::morok::style::StyleSheet"));
    assert!(!through_the_facade.contains("::morok_style::"));
    assert_eq!(
        through_the_facade.matches("::morok::style::").count(),
        through_the_crate.matches("::morok_style::").count()
    );
}

#[test]
fn two_compilations_of_the_same_stylesheets_generate_identical_source() {
    let first = generate_styles_source(&fixture_style_sheet(), SchemaPath::Crate);
    let second = generate_styles_source(&fixture_style_sheet(), SchemaPath::Crate);

    assert_eq!(first, second);
}

#[test]
fn a_class_named_as_a_rust_keyword_becomes_a_raw_identifier() {
    let source = SourceFile::new("box.css", ".box > .col-2 { display: block; }");
    let style_sheet = compile_stylesheets(&[source], &base_tokens()).expect("the stylesheet compiles");

    let generated = generate_styles_source(&style_sheet, SchemaPath::Crate);

    assert!(
        generated.contains("pub r#box: ::morok_style::Class,\n    pub col_2: ::morok_style::Class,"),
        "{generated}"
    );
}
