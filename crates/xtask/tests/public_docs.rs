use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const XTASK: &str = env!("CARGO_BIN_EXE_xtask");
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

const CLEAN: Option<i32> = Some(0);
const FINDINGS: Option<i32> = Some(1);
const USAGE_ERROR: Option<i32> = Some(2);

const PUBLISHED: &str = "";
const UNPUBLISHED: &str = "publish = false\n";
/// The crate of a workspace with one member, and the source the cases write.
const KIT: &str = "kit";
const KIT_SOURCE: &str = "kit/src/lib.rs";

struct Workspace {
    root: PathBuf,
}

struct PublicDocsCheck {
    exit_code: Option<i32>,
    report: String,
}

impl Workspace {
    /// A workspace of the named members, each with the line of its manifest that says whether it
    /// may be published.
    fn of_members(test_name: &str, members: &[(&str, &str)]) -> Self {
        let root = Path::new(SCRATCH_ROOT).join("public_docs").join(test_name);
        if root.exists() {
            fs::remove_dir_all(&root).expect("the previous workspace is removed");
        }
        let member_names: Vec<String> = members.iter().map(|(name, _)| format!("\"{name}\"")).collect();
        let workspace = Self { root }.with_file(
            "Cargo.toml",
            &format!("[workspace]\nmembers = [{}]\n", member_names.join(", ")),
        );
        members.iter().fold(workspace, |workspace, (name, publish)| {
            workspace.with_file(
                &format!("{name}/Cargo.toml"),
                &format!("[package]\nname = \"{name}\"\n{publish}"),
            )
        })
    }

    /// A workspace whose one crate may be published and holds `source`.
    fn of_kit(test_name: &str, source: &str) -> Self {
        Self::of_members(test_name, &[(KIT, PUBLISHED)]).with_file(KIT_SOURCE, source)
    }

    fn with_file(self, relative_path: &str, contents: &str) -> Self {
        let path = self.root.join(relative_path);
        fs::create_dir_all(path.parent().expect("a file of the workspace has a parent directory"))
            .expect("the parent directory is created");
        fs::write(path, contents).expect("the file is written");
        self
    }

    fn check(&self) -> PublicDocsCheck {
        let output = Command::new(XTASK)
            .arg("public-docs")
            .arg(&self.root)
            .output()
            .expect("xtask runs");
        PublicDocsCheck {
            exit_code: output.status.code(),
            report: String::from_utf8(output.stderr).expect("the report is UTF-8"),
        }
    }

    fn undocumented(&self, relative_path: &str, line: usize, item: &str) -> String {
        format!(
            "{}:{line}: a public item carries documentation: {item}",
            self.root.join(relative_path).display()
        )
    }
}

impl PublicDocsCheck {
    fn assert_reports(&self, finding: &str) {
        assert_eq!(self.exit_code, FINDINGS, "{}", self.report);
        assert!(
            self.report.lines().any(|line| line == finding),
            "expected `{finding}` in:\n{}",
            self.report
        );
    }

    fn assert_reports_nothing_of(&self, path: &Path) {
        let named = path.display().to_string();
        assert!(
            !self.report.contains(&named),
            "expected nothing of `{named}` in:\n{}",
            self.report
        );
    }

    fn assert_clean(&self) {
        assert_eq!(self.exit_code, CLEAN, "{}", self.report);
        assert_eq!(self.report, "");
    }
}

#[test]
fn a_public_item_of_each_kind_without_documentation_fails() {
    let undocumented_items = [
        ("pub fn retry() {}\n", "fn retry"),
        ("pub struct Atlas;\n", "struct Atlas"),
        ("pub enum Fit {\n    Cover,\n}\n", "enum Fit"),
        ("pub union Bits {\n    whole: u32,\n}\n", "union Bits"),
        ("pub trait Painter {}\n", "trait Painter"),
        ("pub type Label = String;\n", "type Label"),
        ("pub const LIMIT: u32 = 1;\n", "const LIMIT"),
        ("pub static GREETING: &str = \"hello\";\n", "static GREETING"),
    ];
    for (index, (source, item)) in undocumented_items.into_iter().enumerate() {
        let workspace = Workspace::of_kit(&format!("kind-{index}"), source);

        workspace
            .check()
            .assert_reports(&workspace.undocumented(KIT_SOURCE, 1, item));
    }
}

#[test]
fn a_public_item_of_an_inherent_impl_without_documentation_fails() {
    let workspace = Workspace::of_kit(
        "inherent-impl",
        "/// A page of glyphs.\npub struct Atlas;\n\nimpl Atlas {\n    pub const PAGES: u32 = 4;\n\n    pub fn pack(&self) {}\n}\n",
    );

    let check = workspace.check();

    check.assert_reports(&workspace.undocumented(KIT_SOURCE, 5, "const PAGES"));
    check.assert_reports(&workspace.undocumented(KIT_SOURCE, 7, "fn pack"));
    assert!(
        check.report.ends_with("xtask: 2 public documentation finding(s)\n"),
        "{}",
        check.report
    );
}

#[test]
fn an_item_of_a_public_trait_without_documentation_fails() {
    let workspace = Workspace::of_kit(
        "trait-items",
        "/// Draws a canvas.\npub trait Painter {\n    const LAYERS: u32;\n\n    type Encoder;\n\n    fn paint(&self);\n}\n",
    );

    let check = workspace.check();

    check.assert_reports(&workspace.undocumented(KIT_SOURCE, 3, "const LAYERS"));
    check.assert_reports(&workspace.undocumented(KIT_SOURCE, 5, "type Encoder"));
    check.assert_reports(&workspace.undocumented(KIT_SOURCE, 7, "fn paint"));
}

#[test]
fn documentation_in_each_of_its_forms_passes() {
    let documented_items = [
        "/// Asks again.\npub fn retry() {}\n",
        "/** Asks again. */\npub fn retry() {}\n",
        "#[doc = \"Asks again.\"]\npub fn retry() {}\n",
        "#[doc = include_str!(\"../docs/retry.md\")]\npub fn retry() {}\n",
    ];
    for (index, source) in documented_items.into_iter().enumerate() {
        Workspace::of_kit(&format!("documented-{index}"), source)
            .check()
            .assert_clean();
    }
}

#[test]
fn a_field_and_a_variant_need_no_documentation() {
    Workspace::of_kit(
        "fields-and-variants",
        "/// A size in points.\npub struct Size {\n    pub width: f32,\n    pub height: f32,\n}\n\n/// How a picture fills its box.\npub enum Fit {\n    Cover,\n    Contain,\n}\n",
    )
    .check()
    .assert_clean();
}

#[test]
fn an_item_that_is_not_public_needs_no_documentation() {
    Workspace::of_kit(
        "not-public",
        "fn private() {}\n\npub(crate) fn of_the_crate() {}\n\nmod inner {\n    pub(super) fn of_the_parent() {}\n}\n\npub(crate) trait Painter {\n    fn paint(&self);\n}\n\n/// A page of glyphs.\npub struct Atlas;\n\nimpl Painter for Atlas {\n    fn paint(&self) {}\n}\n",
    )
    .check()
    .assert_clean();
}

#[test]
fn a_reexport_and_a_module_need_no_documentation() {
    Workspace::of_kit("reexport-and-module", "pub mod atlas;\n\npub use atlas::Atlas;\n")
        .with_file("kit/src/atlas.rs", "/// A page of glyphs.\npub struct Atlas;\n")
        .check()
        .assert_clean();
}

#[test]
fn code_inside_a_macro_invocation_is_not_an_item() {
    Workspace::of_kit(
        "macro-invocation",
        "generate! {\n    pub struct Generated;\n}\n\nfn tokens() {\n    let _ = quote! { pub fn written() {} };\n}\n",
    )
    .check()
    .assert_clean();
}

#[test]
fn a_hidden_item_and_an_item_of_the_tests_need_no_documentation() {
    Workspace::of_kit(
        "hidden-and-tests",
        "#[doc(hidden)]\npub fn internal() {}\n\n#[cfg(test)]\npub fn only_in_tests() {}\n\n#[cfg(test)]\nmod tests {\n    pub fn helper() {}\n}\n",
    )
    .check()
    .assert_clean();
}

#[test]
fn an_item_of_an_inline_module_is_checked_as_an_item_of_the_file() {
    let workspace = Workspace::of_kit("inline-module", "pub mod atlas {\n    pub fn pack() {}\n}\n");

    workspace
        .check()
        .assert_reports(&workspace.undocumented(KIT_SOURCE, 2, "fn pack"));
}

#[test]
fn only_the_sources_of_a_crate_that_may_be_published_are_read() {
    let undocumented = "pub fn retry() {}\n";
    let workspace = Workspace::of_members("published-sources", &[("open", PUBLISHED), ("closed", UNPUBLISHED)])
        .with_file("open/src/lib.rs", undocumented)
        .with_file("open/tests/retry.rs", undocumented)
        .with_file("open/build.rs", undocumented)
        .with_file("closed/src/lib.rs", undocumented);

    let check = workspace.check();

    check.assert_reports(&workspace.undocumented("open/src/lib.rs", 1, "fn retry"));
    check.assert_reports_nothing_of(&workspace.root.join("open/tests"));
    check.assert_reports_nothing_of(&workspace.root.join("open/build.rs"));
    check.assert_reports_nothing_of(&workspace.root.join("closed"));
}

#[test]
fn a_source_that_does_not_parse_fails() {
    let workspace = Workspace::of_kit("unparsed", "/// Asks again.\npub fn retry() {\n");

    let check = workspace.check();

    assert_eq!(check.exit_code, FINDINGS, "{}", check.report);
    let unparsed = format!("{}:", workspace.root.join(KIT_SOURCE).display());
    assert!(
        check
            .report
            .lines()
            .any(|line| line.starts_with(&unparsed) && line.contains(": the source does not parse: ")),
        "{}",
        check.report
    );
}

#[test]
fn a_clean_workspace_exits_with_zero() {
    Workspace::of_kit(
        "clean",
        "/// A page of glyphs.\npub struct Atlas;\n\nimpl Atlas {\n    /// Packs the glyphs by shelves.\n    pub fn pack(&self) {}\n}\n",
    )
    .check()
    .assert_clean();
}

#[test]
fn a_missing_workspace_directory_exits_with_two() {
    let output = Command::new(XTASK).arg("public-docs").output().expect("xtask runs");

    assert_eq!(output.status.code(), USAGE_ERROR);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "usage: cargo xtask <check | comments <directory> | public-docs <workspace directory>>\n"
    );
}
