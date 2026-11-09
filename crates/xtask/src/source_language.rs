use std::path::Path;

/// A language whose sources the comment rules cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceLanguage {
    Rust,
    Metal,
    Css,
}

impl SourceLanguage {
    pub(crate) fn of(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "metal" => Some(Self::Metal),
            "css" => Some(Self::Css),
            _ => None,
        }
    }

    pub(crate) fn has_line_comments(self) -> bool {
        matches!(self, Self::Rust | Self::Metal)
    }

    pub(crate) fn nests_block_comments(self) -> bool {
        self == Self::Rust
    }

    pub(crate) fn has_raw_strings(self) -> bool {
        self == Self::Rust
    }

    pub(crate) fn quotes_strings_with_apostrophes(self) -> bool {
        self == Self::Css
    }
}
