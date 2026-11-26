use std::env;
use std::fs;

use morok_css::subset_reference;

const REFERENCE_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/css-subset.md");
const UPDATE_GOLDEN_VARIABLE: &str = "MOROK_UPDATE_GOLDEN";

#[test]
fn the_committed_reference_is_what_the_tables_of_the_compiler_render() {
    let rendered = subset_reference();
    if env::var_os(UPDATE_GOLDEN_VARIABLE).is_some() {
        fs::write(REFERENCE_FILE, &rendered).expect("the reference file is writable");
    }

    let committed = fs::read_to_string(REFERENCE_FILE).expect("the reference file is readable");

    assert!(
        rendered == committed,
        "the rendered reference differs from {REFERENCE_FILE}; rerun with {UPDATE_GOLDEN_VARIABLE}=1 to rewrite it"
    );
}
