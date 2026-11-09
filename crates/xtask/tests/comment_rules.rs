use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const XTASK: &str = env!("CARGO_BIN_EXE_xtask");
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

const CLEAN: Option<i32> = Some(0);
const FINDINGS: Option<i32> = Some(1);
const USAGE_ERROR: Option<i32> = Some(2);

struct SourceTree {
    root: PathBuf,
}

struct CommentCheck {
    exit_code: Option<i32>,
    report: String,
}

impl SourceTree {
    fn named(test_name: &str) -> Self {
        let root = Path::new(SCRATCH_ROOT).join("comment_rules").join(test_name);
        if root.exists() {
            fs::remove_dir_all(&root).expect("the previous source tree is removed");
        }
        fs::create_dir_all(&root).expect("the source tree is created");
        Self { root }
    }

    fn with_file(self, relative_path: &str, contents: &str) -> Self {
        let path = self.root.join(relative_path);
        fs::create_dir_all(path.parent().expect("a source file has a parent directory"))
            .expect("the parent directory is created");
        fs::write(path, contents).expect("the source file is written");
        self
    }

    fn check(&self) -> CommentCheck {
        let output = Command::new(XTASK)
            .arg("comments")
            .arg(&self.root)
            .output()
            .expect("xtask runs");
        CommentCheck {
            exit_code: output.status.code(),
            report: String::from_utf8(output.stderr).expect("the report is UTF-8"),
        }
    }

    fn finding(&self, relative_path: &str, line: usize, reason: &str) -> String {
        format!("{}:{line}: {reason}", self.root.join(relative_path).display())
    }
}

impl CommentCheck {
    fn assert_reports(&self, finding: &str) {
        assert_eq!(self.exit_code, FINDINGS, "{}", self.report);
        assert!(
            self.report.lines().any(|line| line == finding),
            "expected `{finding}` in:\n{}",
            self.report
        );
    }

    fn assert_clean(&self) {
        assert_eq!(self.exit_code, CLEAN, "{}", self.report);
        assert_eq!(self.report, "");
    }
}

#[test]
fn a_comment_with_cyrillic_letters_fails() {
    let tree = SourceTree::named("cyrillic").with_file("kit/src/retry.rs", "fn retry() {}\n// повторить позже\n");

    tree.check().assert_reports(&tree.finding(
        "kit/src/retry.rs",
        2,
        "comments are written in English: // повторить позже",
    ));
}

#[test]
fn a_comment_naming_a_phase_task_or_step_by_number_fails() {
    let numbered_comments = ["// phase 3: retry", "// Task #12 adds this", "/* see steps 2-4 */"];
    for (index, comment) in numbered_comments.into_iter().enumerate() {
        let tree = SourceTree::named(&format!("numbered-{index}")).with_file("retry.rs", &format!("{comment}\n"));

        tree.check().assert_reports(&tree.finding(
            "retry.rs",
            1,
            &format!("comments do not name a phase, task or step by number: {comment}"),
        ));
    }
}

#[test]
fn a_comment_naming_the_spec_or_the_gameplan_fails() {
    let referring_comments = ["// see SPEC-morok", "// the Gameplan asks for this"];
    for (index, comment) in referring_comments.into_iter().enumerate() {
        let tree = SourceTree::named(&format!("spec-{index}")).with_file("retry.rs", &format!("{comment}\n"));

        tree.check().assert_reports(&tree.finding(
            "retry.rs",
            1,
            &format!("comments do not reference the spec or the gameplan: {comment}"),
        ));
    }
}

#[test]
fn metal_and_css_sources_are_scanned() {
    let tree = SourceTree::named("languages")
        .with_file("shaders/rect.metal", "float4 fill;\n// phase 3: retry\n")
        .with_file("styles/button.css", ".button {\n  /* phase 3: retry */\n}\n");

    let check = tree.check();

    check.assert_reports(&tree.finding(
        "shaders/rect.metal",
        2,
        "comments do not name a phase, task or step by number: // phase 3: retry",
    ));
    check.assert_reports(&tree.finding(
        "styles/button.css",
        2,
        "comments do not name a phase, task or step by number: /* phase 3: retry */",
    ));
}

#[test]
fn files_of_other_kinds_are_not_scanned() {
    let tree = SourceTree::named("other-kinds").with_file("notes.md", "// phase 3: retry\n");

    tree.check().assert_clean();
}

#[test]
fn a_string_literal_is_not_a_comment() {
    let tree = SourceTree::named("literals")
        .with_file(
            "greeting.rs",
            "const GREETING: &str = \"привет // phase 3\";\nconst RAW: &str = r#\"/* SPEC-morok \"quoted\" */\"#;\n",
        )
        .with_file("label.css", ".label::after { content: '/* phase 3 */'; }\n")
        .with_file("rect.metal", "const char* label = \"// phase 3\";\n");

    tree.check().assert_clean();
}

#[test]
fn a_css_double_slash_does_not_start_a_comment() {
    let tree = SourceTree::named("css-double-slash")
        .with_file("link.css", ".link { background: url(https://example.com/phase 3); }\n");

    tree.check().assert_clean();
}

#[test]
fn a_finding_names_the_line_inside_a_block_comment() {
    let tree = SourceTree::named("block-comment").with_file(
        "atlas.rs",
        "fn pack() {}\n\n/* Shelf packing keeps rows sorted by height.\n   The rest comes with phase 5.\n*/\n",
    );

    let check = tree.check();

    check.assert_reports(&tree.finding(
        "atlas.rs",
        4,
        "comments do not name a phase, task or step by number: The rest comes with phase 5.",
    ));
    assert!(
        check.report.ends_with("xtask: 1 comment finding(s)\n"),
        "{}",
        check.report
    );
}

#[test]
fn a_comment_breaking_two_rules_is_reported_for_each() {
    let tree = SourceTree::named("two-rules").with_file("retry.rs", "// фаза: phase 3 of SPEC-morok\n");

    let check = tree.check();

    assert!(
        check.report.ends_with("xtask: 3 comment finding(s)\n"),
        "{}",
        check.report
    );
}

#[test]
fn a_clean_tree_exits_with_zero() {
    let tree = SourceTree::named("clean")
        .with_file(
            "kit/src/atlas.rs",
            "/// Packs glyphs by shelves.\nfn pack<'a>(glyphs: &'a [char]) -> char {\n    // A quote is a glyph too.\n    let quote = '\\'';\n    glyphs.first().copied().unwrap_or(quote)\n}\n",
        )
        .with_file(
            "kit/src/rect.metal",
            "// A rounded rectangle as a signed distance field.\n",
        )
        .with_file("kit/src/button.css", "/* The button height is its own geometry. */\n");

    tree.check().assert_clean();
}

#[test]
fn an_unknown_command_exits_with_two() {
    let output = Command::new(XTASK).arg("deploy").output().expect("xtask runs");

    assert_eq!(output.status.code(), USAGE_ERROR);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "usage: cargo xtask <check | comments <directory>>\n"
    );
}
