use morok_css::{SchemaPath, build_script};

/// The canonical tokens of the kit.
const TOKENS_FILE: &str = "../../design/system/tokens.css";
/// Every component keeps its stylesheet next to its source.
const STYLESHEETS_DIRECTORY: &str = "src";

fn main() {
    build_script::generate(TOKENS_FILE, STYLESHEETS_DIRECTORY, SchemaPath::Crate);
}
