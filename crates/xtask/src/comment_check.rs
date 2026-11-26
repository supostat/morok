use std::fs;
use std::io;
use std::path::Path;

use crate::comment::Comment;
use crate::comment_rule::CommentRule;
use crate::comment_scan;
use crate::finding::Finding;
use crate::finding_report;
use crate::source_files;

const SUBJECT: &str = "comment";

/// Reports every comment under `directory` that breaks a comment rule; `false` when there is one.
pub(crate) fn is_clean(directory: &Path) -> bool {
    finding_report::is_clean(SUBJECT, directory, findings(directory))
}

fn findings(directory: &Path) -> io::Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for (path, language) in source_files::under(directory)? {
        let source = fs::read_to_string(&path)?;
        for comment in comment_scan::comments(&source, language) {
            findings.extend(findings_in_comment(&path, &comment));
        }
    }
    Ok(findings)
}

fn findings_in_comment(path: &Path, comment: &Comment) -> Vec<Finding> {
    CommentRule::violations(&comment.text)
        .into_iter()
        .map(|violation| {
            let lines_before = comment.text[..violation.offset].matches('\n').count();
            let offending_line = comment.text.lines().nth(lines_before).unwrap_or_default().trim();
            Finding {
                path: path.to_path_buf(),
                line: comment.line + lines_before,
                reason: format!("{}: {offending_line}", violation.rule.requirement()),
            }
        })
        .collect()
}
