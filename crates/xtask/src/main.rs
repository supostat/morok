#![forbid(unsafe_code)]

mod check;
mod comment;
mod comment_check;
mod comment_rule;
mod comment_scan;
mod finding;
mod finding_report;
mod public_crates;
mod public_docs_check;
mod source_files;
mod source_language;
mod undocumented_item;
mod workspace;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask <check | comments <directory> | public-docs <workspace directory>>";
const USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let is_clean = match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["check"] => check::is_clean(),
        ["comments", directory] => comment_check::is_clean(Path::new(directory)),
        ["public-docs", workspace] => public_docs_check::is_clean(Path::new(workspace)),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(USAGE_ERROR);
        }
    };
    if is_clean { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
