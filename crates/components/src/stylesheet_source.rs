/// A stylesheet of the kit as the build compiled it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StylesheetSource {
    /// Where the file lies, from the root of the workspace.
    pub path: &'static str,
    pub text: &'static str,
}
