#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TokenNameError {
    NotLowercaseKebabCase,
    UnknownFamily { family: String },
    NameMissing,
}
