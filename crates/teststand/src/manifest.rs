use std::fmt::Write;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// What an etalon was rendered from, as hashes: a case whose inputs hash differently has
/// changed since, and its etalon no longer says how Chrome draws it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The stylesheets of the suite.
    pub stylesheet: String,
    /// The harness page of the case without its tokens and stylesheets.
    pub markup: String,
    pub tokens: String,
    /// The version of the Chrome build.
    pub chrome: String,
}

impl Manifest {
    const INPUTS: [&'static str; 4] = ["stylesheet", "markup", "tokens", "Chrome build"];

    pub(crate) fn of(stylesheets: &[&str], page_without_styles: &str, tokens: &str, chrome_version: &str) -> Self {
        Self {
            stylesheet: hash(stylesheets),
            markup: hash(&[page_without_styles]),
            tokens: hash(&[tokens]),
            chrome: hash(&[chrome_version]),
        }
    }

    /// The names of the inputs that hash differently in `other`.
    pub fn changed_inputs(&self, other: &Self) -> Vec<&'static str> {
        let hashes = |manifest: &'_ Self| {
            [
                manifest.stylesheet.clone(),
                manifest.markup.clone(),
                manifest.tokens.clone(),
                manifest.chrome.clone(),
            ]
        };
        Self::INPUTS
            .into_iter()
            .zip(hashes(self).into_iter().zip(hashes(other)))
            .filter(|(_, (own, others))| own != others)
            .map(|(input, _)| input)
            .collect()
    }
}

/// The SHA-256 of the parts in hexadecimal. Each part is closed by a zero byte, so that
/// moving text from one part to the next changes the hash.
fn hash(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    let mut hexadecimal = String::new();
    for byte in hasher.finalize() {
        write!(hexadecimal, "{byte:02x}").expect("a string takes whatever is written to it");
    }
    hexadecimal
}
