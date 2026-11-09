use std::env;
use std::path::Path;
use std::process::Command;

use crate::comment_check;
use crate::workspace;

const RUSTFMT_CHECK: [&str; 3] = ["fmt", "--all", "--check"];
const CLIPPY: [&str; 6] = ["clippy", "--workspace", "--all-targets", "--", "--deny", "warnings"];

/// Runs every check even after one fails, so a single run reports everything there is to fix.
pub(crate) fn is_clean() -> bool {
    let workspace_root = workspace::root();
    let steps = [
        ("rustfmt", cargo_succeeds(&workspace_root, &RUSTFMT_CHECK)),
        ("clippy", cargo_succeeds(&workspace_root, &CLIPPY)),
        ("comment rules", comment_check::is_clean(&workspace::crates_directory())),
    ];
    let failed_steps: Vec<&str> = steps
        .into_iter()
        .filter(|(_, step_is_clean)| !step_is_clean)
        .map(|(step, _)| step)
        .collect();
    for step in &failed_steps {
        eprintln!("xtask: {step} failed");
    }
    failed_steps.is_empty()
}

fn cargo_succeeds(workspace_root: &Path, arguments: &[&str]) -> bool {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    match Command::new(cargo).args(arguments).current_dir(workspace_root).status() {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("xtask: cannot run cargo {}: {error}", arguments.join(" "));
            false
        }
    }
}
