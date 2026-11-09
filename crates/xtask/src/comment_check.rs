use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::comment::Comment;
use crate::comment_rule::CommentRule;
use crate::comment_scan;
use crate::finding::Finding;
use crate::source_language::SourceLanguage;

/// Reports every comment under `directory` that breaks a comment rule; `false` when there is one.
pub(crate) fn is_clean(directory: &Path) -> bool {
    match findings(directory) {
        Ok(findings) if findings.is_empty() => true,
        Ok(findings) => {
            for finding in &findings {
                eprintln!("{finding}");
            }
            eprintln!("xtask: {} comment finding(s)", findings.len());
            false
        }
        Err(error) => {
            eprintln!("xtask: cannot scan {}: {error}", directory.display());
            false
        }
    }
}

fn findings(directory: &Path) -> io::Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for (path, language) in source_files(directory)? {
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

fn source_files(directory: &Path) -> io::Result<Vec<(PathBuf, SourceLanguage)>> {
    let mut source_files = Vec::new();
    let mut unvisited_directories = vec![directory.to_path_buf()];
    while let Some(unvisited) = unvisited_directories.pop() {
        for entry in fs::read_dir(unvisited)? {
            let path = entry?.path();
            if path.is_dir() {
                unvisited_directories.push(path);
            } else if let Some(language) = SourceLanguage::of(&path) {
                source_files.push((path, language));
            }
        }
    }
    source_files.sort_by(|(left, _), (right, _)| left.cmp(right));
    Ok(source_files)
}
