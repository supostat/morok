//! The entry point a `build.rs` calls.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use crate::build_error::BuildError;
use crate::schema_path::SchemaPath;
use crate::source_file::SourceFile;
use crate::style_codegen::generate_styles_source;
use crate::stylesheet_compiler::compile_stylesheets;
use crate::token_codegen::generate_tokens_source;
use crate::token_compiler::compile_tokens;

const GENERATED_TOKENS_FILE: &str = "tokens.rs";
const GENERATED_STYLES_FILE: &str = "styles.rs";
const STYLESHEET_EXTENSION: &str = "css";

/// Compiles `tokens_file` and every `.css` file under `stylesheets_directory` into
/// `$OUT_DIR/tokens.rs` and `$OUT_DIR/styles.rs`, and tells cargo to rerun the build script when
/// any of them changes. On a rejected file it prints the diagnostics and fails the build.
///
/// Stylesheets are compiled in the order of their paths. The crate includes the results with
/// `include!(concat!(env!("OUT_DIR"), "/tokens.rs"));` and
/// `include!(concat!(env!("OUT_DIR"), "/styles.rs"));`.
pub fn generate(tokens_file: impl AsRef<Path>, stylesheets_directory: impl AsRef<Path>, schema: SchemaPath) {
    let tokens_file = tokens_file.as_ref();
    let stylesheets_directory = stylesheets_directory.as_ref();
    println!("cargo::rerun-if-changed={}", tokens_file.display());
    println!("cargo::rerun-if-changed={}", stylesheets_directory.display());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts"));
    if let Err(error) = write_generated_sources(tokens_file, stylesheets_directory, &out_dir, schema) {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn write_generated_sources(
    tokens_file: &Path,
    stylesheets_directory: &Path,
    out_dir: &Path,
    schema: SchemaPath,
) -> Result<(), BuildError> {
    let tokens = compile_tokens(&source_file(tokens_file)?).map_err(BuildError::Rejected)?;
    let stylesheets = stylesheet_files(stylesheets_directory)
        .map_err(|error| BuildError::Unreadable {
            path: stylesheets_directory.to_path_buf(),
            error,
        })?
        .iter()
        .map(|path| source_file(path))
        .collect::<Result<Vec<_>, _>>()?;
    let style_sheet = compile_stylesheets(&stylesheets, &tokens).map_err(BuildError::Rejected)?;
    write(
        out_dir.join(GENERATED_TOKENS_FILE),
        &generate_tokens_source(&tokens, schema),
    )?;
    write(
        out_dir.join(GENERATED_STYLES_FILE),
        &generate_styles_source(&style_sheet, schema),
    )
}

fn source_file(path: &Path) -> Result<SourceFile, BuildError> {
    SourceFile::read(path).map_err(|error| BuildError::Unreadable {
        path: path.to_path_buf(),
        error,
    })
}

fn write(path: PathBuf, generated_source: &str) -> Result<(), BuildError> {
    fs::write(&path, generated_source).map_err(|error| BuildError::Unwritable { path, error })
}

/// Every stylesheet under `directory`, at any depth, ordered by path.
fn stylesheet_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut stylesheets = Vec::new();
    let mut unvisited_directories = vec![directory.to_path_buf()];
    while let Some(unvisited) = unvisited_directories.pop() {
        for entry in fs::read_dir(unvisited)? {
            let path = entry?.path();
            if path.is_dir() {
                unvisited_directories.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == STYLESHEET_EXTENSION)
            {
                stylesheets.push(path);
            }
        }
    }
    stylesheets.sort();
    Ok(stylesheets)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;

    use super::{GENERATED_STYLES_FILE, GENERATED_TOKENS_FILE, write_generated_sources};
    use crate::build_error::BuildError;
    use crate::schema_path::SchemaPath;

    const TOKENS: &str =
        ":root { --space-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --space-1: 4px; } }\n";
    const REJECTED_TOKENS: &str =
        ":root { --glow-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --glow-1: 4px; } }\n";

    struct Scratch {
        directory: PathBuf,
    }

    impl Scratch {
        fn named(test_name: &str) -> Self {
            let directory = std::env::temp_dir().join(format!("morok-css-{test_name}-{}", process::id()));
            fs::create_dir_all(directory.join("styles/nested")).expect("the scratch directory is created");
            Self { directory }
        }

        fn with_file(self, relative_path: &str, contents: &str) -> Self {
            fs::write(self.directory.join(relative_path), contents).expect("the file is written");
            self
        }

        fn generate(&self, schema: SchemaPath) -> Result<(), BuildError> {
            write_generated_sources(
                &self.directory.join("tokens.css"),
                &self.directory.join("styles"),
                &self.directory,
                schema,
            )
        }

        fn generated(&self, file_name: &str) -> String {
            fs::read_to_string(self.directory.join(file_name)).expect("the generated file exists")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    #[test]
    fn writes_tokens_and_styles_into_the_output_directory() {
        let scratch = Scratch::named("writes")
            .with_file("tokens.css", TOKENS)
            .with_file("styles/nested/row.css", ".row { gap: var(--space-1); }\n")
            .with_file("styles/button.css", ".button { padding: var(--space-1); }\n")
            .with_file("styles/notes.txt", ".ignored { float: left; }\n");

        scratch.generate(SchemaPath::Facade).expect("valid sources compile");

        let tokens = scratch.generated(GENERATED_TOKENS_FILE);
        let styles = scratch.generated(GENERATED_STYLES_FILE);
        assert!(
            tokens.contains("pub space_1: ::morok::style::Token<::morok::style::Pixels>"),
            "{tokens}"
        );
        assert!(
            styles.contains("pub button: ::morok::style::Class,\n    pub row: ::morok::style::Class,"),
            "{styles}"
        );
    }

    #[test]
    fn a_rejected_tokens_file_writes_nothing_and_carries_its_diagnostics() {
        let scratch = Scratch::named("rejected-tokens").with_file("tokens.css", REJECTED_TOKENS);

        let error = scratch.generate(SchemaPath::Crate).expect_err("the family is unknown");

        assert!(matches!(error, BuildError::Rejected(_)), "{error}");
        assert!(error.to_string().contains("unknown token family `glow`"), "{error}");
        assert!(!scratch.directory.join(GENERATED_TOKENS_FILE).exists());
    }

    #[test]
    fn a_rejected_stylesheet_writes_nothing_and_names_its_file() {
        let scratch = Scratch::named("rejected-styles")
            .with_file("tokens.css", TOKENS)
            .with_file("styles/button.css", ".button { float: left; }\n");

        let error = scratch
            .generate(SchemaPath::Crate)
            .expect_err("the property is unsupported");

        assert!(
            error.to_string().contains("`float` is not a supported property"),
            "{error}"
        );
        assert!(error.to_string().contains("button.css:1:11"), "{error}");
        assert!(!scratch.directory.join(GENERATED_TOKENS_FILE).exists());
        assert!(!scratch.directory.join(GENERATED_STYLES_FILE).exists());
    }

    #[test]
    fn a_missing_tokens_file_is_named_in_the_error() {
        let scratch = Scratch::named("missing");

        let error = scratch
            .generate(SchemaPath::Crate)
            .expect_err("the file does not exist");

        assert!(matches!(error, BuildError::Unreadable { .. }), "{error}");
        assert!(error.to_string().contains("tokens.css"), "{error}");
    }
}
