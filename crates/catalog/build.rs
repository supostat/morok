use morok_components::{KIT_TOKENS, STYLESHEETS};
use morok_css::{KitSources, SourceFile, build_script};

/// The tokens of the catalog: every token of the kit and the colors of its charts.
const TOKENS_FILE: &str = "tokens/base.css";
/// The values the catalog switches its tokens to.
const ALTERNATE_TOKENS_FILES: [&str; 1] = ["tokens/ember.css"];
const STYLESHEETS_DIRECTORY: &str = "styles";

fn main() {
    let kit = KitSources {
        tokens: SourceFile::new(KIT_TOKENS.path, KIT_TOKENS.text),
        stylesheets: STYLESHEETS
            .iter()
            .map(|stylesheet| SourceFile::new(stylesheet.path, stylesheet.text))
            .collect(),
    };
    build_script::generate_application(&kit, TOKENS_FILE, &ALTERNATE_TOKENS_FILES, STYLESHEETS_DIRECTORY);
}
