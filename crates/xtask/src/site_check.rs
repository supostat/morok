use std::io;
use std::path::Path;

use crate::finding::Finding;
use crate::finding_report;
use crate::site_page;
use crate::site_tokens;

const SUBJECT: &str = "site";

/// Reports whatever keeps the folder of the site of the workspace at `workspace` from being
/// what is published; `false` when there is a finding.
pub(crate) fn is_clean(workspace: &Path) -> bool {
    finding_report::is_clean(SUBJECT, workspace, findings(workspace))
}

fn findings(workspace: &Path) -> io::Result<Vec<Finding>> {
    let mut findings = site_tokens::findings(workspace)?;
    findings.extend(site_page::findings(workspace)?);
    Ok(findings)
}
