use std::path::Path;

use crate::finding_report;
use crate::site_tokens;

const SUBJECT: &str = "site";

/// Reports whatever keeps the folder of the site of the workspace at `workspace` from being
/// what is published; `false` when there is a finding.
pub(crate) fn is_clean(workspace: &Path) -> bool {
    finding_report::is_clean(SUBJECT, workspace, site_tokens::findings(workspace))
}
