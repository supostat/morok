use std::io;
use std::path::Path;

use crate::finding::Finding;

/// Prints the findings of one check over `scanned` and tells whether there were none. `subject`
/// names what the check looks at, as in "comment".
pub(crate) fn is_clean(subject: &str, scanned: &Path, findings: io::Result<Vec<Finding>>) -> bool {
    match findings {
        Ok(findings) if findings.is_empty() => true,
        Ok(findings) => {
            for finding in &findings {
                eprintln!("{finding}");
            }
            eprintln!("xtask: {} {subject} finding(s)", findings.len());
            false
        }
        Err(error) => {
            eprintln!("xtask: cannot scan {}: {error}", scanned.display());
            false
        }
    }
}
