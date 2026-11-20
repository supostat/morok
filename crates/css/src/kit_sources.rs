use crate::source_file::SourceFile;

/// What an application is compiled over, as the kit ships it: the tokens of the kit and the
/// stylesheets of its components, in the order the kit compiles them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KitSources {
    pub tokens: SourceFile,
    pub stylesheets: Vec<SourceFile>,
}
