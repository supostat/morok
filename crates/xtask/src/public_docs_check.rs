use std::fs;
use std::io;
use std::path::Path;

use crate::finding::Finding;
use crate::finding_report;
use crate::public_crates;
use crate::source_files;
use crate::source_language::SourceLanguage;
use crate::undocumented_item::UndocumentedItem;

const SUBJECT: &str = "public documentation";
const REQUIREMENT: &str = "a public item carries documentation";
const UNREADABLE_SOURCE: &str = "the source does not parse";

/// Reports every public item without documentation in the crates of the workspace at
/// `workspace` that may be published; `false` when there is one.
pub(crate) fn is_clean(workspace: &Path) -> bool {
    finding_report::is_clean(SUBJECT, workspace, findings(workspace))
}

fn findings(workspace: &Path) -> io::Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for source_directory in public_crates::source_directories(workspace)? {
        for (path, language) in source_files::under(&source_directory)? {
            if language == SourceLanguage::Rust {
                findings.extend(findings_in_source(&path)?);
            }
        }
    }
    Ok(findings)
}

fn findings_in_source(path: &Path) -> io::Result<Vec<Finding>> {
    let source = fs::read_to_string(path)?;
    Ok(match UndocumentedItem::all_in(&source) {
        Ok(undocumented) => undocumented
            .into_iter()
            .map(|undocumented| Finding {
                path: path.to_path_buf(),
                line: undocumented.line,
                reason: format!("{REQUIREMENT}: {}", undocumented.item),
            })
            .collect(),
        Err(error) => vec![Finding {
            path: path.to_path_buf(),
            line: error.span().start().line,
            reason: format!("{UNREADABLE_SOURCE}: {error}"),
        }],
    })
}
