use morok_css::SourceFile;

use crate::case::Case;

/// A set of cases styled by the same stylesheets.
#[derive(Clone, Debug)]
pub struct Suite {
    /// Lowercase kebab-case; names the directory of the etalons.
    pub name: &'static str,
    /// In the order of the cascade.
    pub stylesheets: Vec<SourceFile>,
    pub cases: Vec<Case>,
}
