use morok_css::SourceFile;

use crate::case::Case;

/// A set of cases styled by the same stylesheets.
#[derive(Clone, Debug)]
pub struct Suite {
    /// Lowercase kebab-case; names the directory of the etalons.
    pub name: &'static str,
    /// The stylesheets of the components of the kit the cases mount, as the kit ships them;
    /// none for a suite of primitives. They come before the stylesheets of the suite in the
    /// cascade.
    pub component_stylesheets: Vec<SourceFile>,
    /// The stylesheets of the suite itself, in the order of the cascade.
    pub stylesheets: Vec<SourceFile>,
    pub cases: Vec<Case>,
}
