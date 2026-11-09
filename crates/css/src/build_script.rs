//! The entry point a `build.rs` calls.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::schema_path::SchemaPath;
use crate::source_file::SourceFile;
use crate::token_codegen::generate_tokens_source;
use crate::token_compiler::compile_tokens;
use crate::tokens_build_error::TokensBuildError;

const GENERATED_TOKENS_FILE: &str = "tokens.rs";

/// Compiles `tokens_file` into `$OUT_DIR/tokens.rs` and tells cargo to rerun the build script when
/// the file changes. On a rejected file it prints the diagnostics and fails the build.
///
/// The crate includes the result with `include!(concat!(env!("OUT_DIR"), "/tokens.rs"));`.
pub fn generate_tokens(tokens_file: impl AsRef<Path>, schema: SchemaPath) {
    let tokens_file = tokens_file.as_ref();
    println!("cargo::rerun-if-changed={}", tokens_file.display());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts"));
    if let Err(error) = write_tokens(tokens_file, &out_dir, schema) {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn write_tokens(tokens_file: &Path, out_dir: &Path, schema: SchemaPath) -> Result<(), TokensBuildError> {
    let source = SourceFile::read(tokens_file).map_err(|error| TokensBuildError::Unreadable {
        path: tokens_file.to_path_buf(),
        error,
    })?;
    let tokens = compile_tokens(&source).map_err(TokensBuildError::Rejected)?;
    let generated_file = out_dir.join(GENERATED_TOKENS_FILE);
    fs::write(&generated_file, generate_tokens_source(&tokens, schema)).map_err(|error| TokensBuildError::Unwritable {
        path: generated_file,
        error,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process;

    use super::{GENERATED_TOKENS_FILE, write_tokens};
    use crate::schema_path::SchemaPath;
    use crate::tokens_build_error::TokensBuildError;

    const VALID_TOKENS: &str =
        ":root { --space-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --space-1: 4px; } }\n";
    const REJECTED_TOKENS: &str =
        ":root { --glow-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --glow-1: 4px; } }\n";

    fn scratch_directory(test_name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("morok-css-{test_name}-{}", process::id()));
        fs::create_dir_all(&directory).expect("the scratch directory is created");
        directory
    }

    #[test]
    fn writes_the_generated_tokens_into_the_output_directory() {
        let directory = scratch_directory("writes");
        let tokens_file = directory.join("tokens.css");
        fs::write(&tokens_file, VALID_TOKENS).expect("the tokens file is written");

        write_tokens(&tokens_file, &directory, SchemaPath::Facade).expect("valid tokens compile");

        let generated = fs::read_to_string(directory.join(GENERATED_TOKENS_FILE)).expect("the generated file exists");
        assert!(
            generated.contains("pub space_1: ::morok::style::Token<::morok::style::Pixels>"),
            "{generated}"
        );
        fs::remove_dir_all(&directory).expect("the scratch directory is removed");
    }

    #[test]
    fn a_rejected_tokens_file_writes_nothing_and_carries_its_diagnostics() {
        let directory = scratch_directory("rejects");
        let tokens_file = directory.join("tokens.css");
        fs::write(&tokens_file, REJECTED_TOKENS).expect("the tokens file is written");

        let error = write_tokens(&tokens_file, &directory, SchemaPath::Crate).expect_err("the family is unknown");

        assert!(matches!(error, TokensBuildError::Rejected(_)), "{error}");
        assert!(error.to_string().contains("unknown token family `glow`"), "{error}");
        assert!(!directory.join(GENERATED_TOKENS_FILE).exists());
        fs::remove_dir_all(&directory).expect("the scratch directory is removed");
    }

    #[test]
    fn a_missing_tokens_file_is_named_in_the_error() {
        let directory = scratch_directory("missing");
        let tokens_file = directory.join("absent.css");

        let error = write_tokens(&tokens_file, &directory, SchemaPath::Crate).expect_err("the file does not exist");

        assert!(matches!(error, TokensBuildError::Unreadable { .. }), "{error}");
        assert!(error.to_string().contains("absent.css"), "{error}");
        fs::remove_dir_all(&directory).expect("the scratch directory is removed");
    }
}
