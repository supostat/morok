//! The entry point a `build.rs` calls.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use crate::build_error::BuildError;
use crate::kit_sources::KitSources;
use crate::schema_path::SchemaPath;
use crate::source_file::SourceFile;
use crate::style_codegen::generate_styles_source;
use crate::stylesheet_compiler::compile_stylesheets;
use crate::token_codegen::{generate_themes_source, generate_tokens_source};
use crate::token_compiler::{compile_alternate_tokens, compile_application_tokens, compile_tokens};

const GENERATED_TOKENS_FILE: &str = "tokens.rs";
const GENERATED_STYLES_FILE: &str = "styles.rs";
/// The directory of `$OUT_DIR` the values of the tokens files an application switches to are
/// generated into.
const GENERATED_THEMES_DIRECTORY: &str = "themes";
const STYLESHEET_EXTENSION: &str = "css";
const GENERATED_EXTENSION: &str = "rs";

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

/// Compiles an application over the kit. `tokens_file` is the application's own: it defines
/// every token of the kit and may add tokens of its own, and it goes into `$OUT_DIR/tokens.rs`.
/// Every file of `alternate_tokens_files` is a set of values the application switches to: it
/// defines exactly the tokens of `tokens_file`, and its `LIGHT` and `DARK` tables go into
/// `$OUT_DIR/themes/<file stem>.rs`. The stylesheets of the kit, then every `.css` file under
/// `stylesheets_directory` in the order of their paths, go into `$OUT_DIR/styles.rs`: the
/// classes of the kit keep the IDs its components know them by. The generated code reaches the
/// schema through the `morok` facade.
///
/// Tells cargo to rerun the build script when a file of the application changes; on a rejected
/// file it prints the diagnostics and fails the build.
pub fn generate_application(
    kit: &KitSources,
    tokens_file: impl AsRef<Path>,
    alternate_tokens_files: &[impl AsRef<Path>],
    stylesheets_directory: impl AsRef<Path>,
) {
    let tokens_file = tokens_file.as_ref();
    let alternate_tokens_files: Vec<&Path> = alternate_tokens_files.iter().map(AsRef::as_ref).collect();
    let stylesheets_directory = stylesheets_directory.as_ref();
    for watched in [tokens_file, stylesheets_directory]
        .into_iter()
        .chain(alternate_tokens_files.iter().copied())
    {
        println!("cargo::rerun-if-changed={}", watched.display());
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts"));
    let application = ApplicationFiles {
        tokens_file,
        alternate_tokens_files: &alternate_tokens_files,
        stylesheets_directory,
    };
    if let Err(error) = write_application_sources(kit, &application, &out_dir) {
        eprintln!("{error}");
        process::exit(1);
    }
}

/// The files of an application the build compiles.
struct ApplicationFiles<'a> {
    tokens_file: &'a Path,
    alternate_tokens_files: &'a [&'a Path],
    stylesheets_directory: &'a Path,
}

fn write_application_sources(
    kit: &KitSources,
    application: &ApplicationFiles<'_>,
    out_dir: &Path,
) -> Result<(), BuildError> {
    let kit_tokens = compile_tokens(&kit.tokens).map_err(BuildError::Rejected)?;
    let tokens = compile_application_tokens(&kit_tokens, &source_file(application.tokens_file)?)
        .map_err(BuildError::Rejected)?;
    let mut alternates: Vec<GeneratedThemes<'_>> = Vec::with_capacity(application.alternate_tokens_files.len());
    for tokens_file in application.alternate_tokens_files {
        let alternate = compile_alternate_tokens(&tokens, &source_file(tokens_file)?).map_err(BuildError::Rejected)?;
        let path = themes_file(out_dir, tokens_file);
        if let Some(earlier) = alternates.iter().find(|earlier| earlier.path == path) {
            return Err(BuildError::SameThemesFile {
                first: earlier.tokens_file.to_path_buf(),
                second: tokens_file.to_path_buf(),
            });
        }
        alternates.push(GeneratedThemes {
            tokens_file,
            path,
            source: generate_themes_source(&alternate, SchemaPath::Facade),
        });
    }
    let application_stylesheets = stylesheet_files(application.stylesheets_directory)
        .map_err(|error| BuildError::Unreadable {
            path: application.stylesheets_directory.to_path_buf(),
            error,
        })?
        .iter()
        .map(|path| source_file(path))
        .collect::<Result<Vec<_>, _>>()?;
    let stylesheets: Vec<SourceFile> = kit.stylesheets.iter().cloned().chain(application_stylesheets).collect();
    let style_sheet = compile_stylesheets(&stylesheets, &tokens).map_err(BuildError::Rejected)?;
    write(
        out_dir.join(GENERATED_TOKENS_FILE),
        &generate_tokens_source(&tokens, SchemaPath::Facade),
    )?;
    write(
        out_dir.join(GENERATED_STYLES_FILE),
        &generate_styles_source(&style_sheet, SchemaPath::Facade),
    )?;
    let themes_directory = out_dir.join(GENERATED_THEMES_DIRECTORY);
    fs::create_dir_all(&themes_directory).map_err(|error| BuildError::Unwritable {
        path: themes_directory,
        error,
    })?;
    for alternate in alternates {
        write(alternate.path, &alternate.source)?;
    }
    Ok(())
}

/// The values of a tokens file an application switches to, as they are generated.
struct GeneratedThemes<'a> {
    tokens_file: &'a Path,
    path: PathBuf,
    source: String,
}

/// Where the values of an alternate tokens file are generated: named after the file.
fn themes_file(out_dir: &Path, alternate_tokens_file: &Path) -> PathBuf {
    let stem = alternate_tokens_file
        .file_stem()
        .expect("a tokens file that was read has a name");
    out_dir
        .join(GENERATED_THEMES_DIRECTORY)
        .join(stem)
        .with_extension(GENERATED_EXTENSION)
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

    use super::{
        ApplicationFiles, GENERATED_STYLES_FILE, GENERATED_THEMES_DIRECTORY, GENERATED_TOKENS_FILE,
        write_application_sources, write_generated_sources,
    };
    use crate::build_error::BuildError;
    use crate::kit_sources::KitSources;
    use crate::schema_path::SchemaPath;
    use crate::source_file::SourceFile;

    const TOKENS: &str =
        ":root { --space-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --space-1: 4px; } }\n";
    const REJECTED_TOKENS: &str =
        ":root { --glow-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --glow-1: 4px; } }\n";
    const APPLICATION_TOKENS: &str = ":root { --space-2: 8px; --space-1: 4px; }\n@media (prefers-color-scheme: dark) { :root { --space-1: 4px; --space-2: 8px; } }\n";
    const EMBER_TOKENS: &str = ":root { --space-1: 6px; --space-2: 10px; }\n@media (prefers-color-scheme: dark) { :root { --space-1: 6px; --space-2: 10px; } }\n";

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

        fn generate_application(&self, alternate_tokens_files: &[&str]) -> Result<(), BuildError> {
            let kit = KitSources {
                tokens: SourceFile::new("kit/tokens.css", TOKENS),
                stylesheets: vec![SourceFile::new(
                    "kit/button.css",
                    ".button { padding: var(--space-1); }\n",
                )],
            };
            let alternates: Vec<PathBuf> = alternate_tokens_files
                .iter()
                .map(|relative_path| self.directory.join(relative_path))
                .collect();
            let alternates: Vec<&std::path::Path> = alternates.iter().map(PathBuf::as_path).collect();
            write_application_sources(
                &kit,
                &ApplicationFiles {
                    tokens_file: &self.directory.join("tokens.css"),
                    alternate_tokens_files: &alternates,
                    stylesheets_directory: &self.directory.join("styles"),
                },
                &self.directory,
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
    fn an_application_compiles_after_the_kit_and_generates_the_values_it_switches_to() {
        let scratch = Scratch::named("application")
            .with_file("tokens.css", APPLICATION_TOKENS)
            .with_file("ember.css", EMBER_TOKENS)
            .with_file("styles/page.css", ".page { gap: var(--space-2); }\n");

        scratch
            .generate_application(&["ember.css"])
            .expect("the application compiles");

        let tokens = scratch.generated(GENERATED_TOKENS_FILE);
        let styles = scratch.generated(GENERATED_STYLES_FILE);
        let ember = scratch.generated(&format!("{GENERATED_THEMES_DIRECTORY}/ember.rs"));
        assert!(
            tokens.contains("pub space_1: ::morok::style::Token<::morok::style::Pixels>,\n    pub space_2:"),
            "{tokens}"
        );
        assert!(
            styles.contains("pub button: ::morok::style::Class,\n    pub page: ::morok::style::Class,"),
            "{styles}"
        );
        assert!(ember.contains("pub const LIGHT: ::morok::style::Theme"), "{ember}");
        assert!(ember.contains("::morok::style::Pixels(6"), "{ember}");
        assert!(!ember.contains("pub struct Tokens"), "{ember}");
    }

    #[test]
    fn an_application_that_leaves_out_a_token_of_the_kit_writes_nothing() {
        let scratch = Scratch::named("application-rejected")
            .with_file(
                "tokens.css",
                ":root { --space-2: 8px; }\n@media (prefers-color-scheme: dark) { :root { --space-2: 8px; } }\n",
            )
            .with_file("styles/page.css", ".page { gap: var(--space-2); }\n");

        let error = scratch
            .generate_application(&[])
            .expect_err("a token of the kit is left out");

        assert!(error.to_string().contains("`--space-1` is not defined"), "{error}");
        assert!(!scratch.directory.join(GENERATED_STYLES_FILE).exists());
    }

    #[test]
    fn two_tokens_files_an_application_switches_to_have_names_of_their_own() {
        let scratch = Scratch::named("application-same-name");
        fs::create_dir_all(scratch.directory.join("other")).expect("the directory is created");
        let scratch = scratch
            .with_file("tokens.css", APPLICATION_TOKENS)
            .with_file("ember.css", EMBER_TOKENS)
            .with_file("other/ember.css", EMBER_TOKENS)
            .with_file("styles/page.css", ".page { gap: var(--space-2); }\n");

        let error = scratch
            .generate_application(&["ember.css", "other/ember.css"])
            .expect_err("the two files would generate one file");

        assert!(matches!(error, BuildError::SameThemesFile { .. }), "{error}");
        assert!(!scratch.directory.join(GENERATED_TOKENS_FILE).exists());
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
