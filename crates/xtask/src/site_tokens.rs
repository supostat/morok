use std::fs;
use std::io;
use std::path::Path;

use crate::dark_tokens::DarkTokens;
use crate::derived_file::DerivedFile;
use crate::finding::Finding;
use crate::line_number;

const TOKENS: &str = "design/system/tokens.css";
/// The dark tokens for the etalons of the pages, which link them beside the tokens.
const DESIGN_MIRROR: &str = "design/system/tokens-dark.css";
const SITE_TOKENS: &str = "site/tokens.css";
const SITE_MIRROR: &str = "site/tokens-dark.css";

const NO_DARK_BLOCK: &str = "the tokens file has no dark block to render the dark tokens of a page from: \
                             declare it in `@media (prefers-color-scheme: dark) { :root { … } }`";

/// Writes the files derived from the tokens of the workspace at `workspace`: the copy of the
/// tokens the site publishes and the two mirrors of their dark block. `false` when the tokens
/// render nothing or a file could not be written.
pub(crate) fn write(workspace: &Path) -> bool {
    match written(workspace) {
        Ok(None) => true,
        Ok(Some(finding)) => {
            eprintln!("{finding}");
            false
        }
        Err(error) => {
            eprintln!(
                "xtask: cannot write the token files of {}: {error}",
                workspace.display()
            );
            false
        }
    }
}

/// Reports every derived file of the workspace that is missing or fell behind its tokens.
pub(crate) fn findings(workspace: &Path) -> io::Result<Vec<Finding>> {
    match derived_files(workspace)? {
        Ok(derived_files) => derived_files
            .iter()
            .filter_map(|derived_file| derived_file.finding().transpose())
            .collect(),
        Err(nothing_rendered) => Ok(vec![nothing_rendered]),
    }
}

/// The finding that kept the files from being written; none when all of them were.
fn written(workspace: &Path) -> io::Result<Option<Finding>> {
    match derived_files(workspace)? {
        Ok(derived_files) => {
            for derived_file in &derived_files {
                derived_file.write()?;
            }
            Ok(None)
        }
        Err(nothing_rendered) => Ok(Some(nothing_rendered)),
    }
}

/// The derived files as the tokens of the workspace render them now, or the finding that the
/// tokens render nothing.
fn derived_files(workspace: &Path) -> io::Result<Result<Vec<DerivedFile>, Finding>> {
    let tokens_file = workspace.join(TOKENS);
    let tokens_text = fs::read_to_string(&tokens_file)?;
    let Some(dark_tokens) = DarkTokens::of(&tokens_text) else {
        return Ok(Err(Finding {
            path: tokens_file,
            line: line_number::FIRST,
            reason: NO_DARK_BLOCK.to_owned(),
        }));
    };
    let mirror = dark_tokens.mirror();
    Ok(Ok(vec![
        DerivedFile {
            path: workspace.join(DESIGN_MIRROR),
            text: mirror.clone(),
        },
        DerivedFile {
            path: workspace.join(SITE_TOKENS),
            text: tokens_text,
        },
        DerivedFile {
            path: workspace.join(SITE_MIRROR),
            text: mirror,
        },
    ]))
}
