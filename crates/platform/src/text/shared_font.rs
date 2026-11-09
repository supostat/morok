use objc2_core_foundation::CFRetained;
use objc2_core_text::CTFont;

/// A Core Text font that several threads may hold.
#[derive(Debug)]
pub(crate) struct SharedFont(pub(crate) CFRetained<CTFont>);

// SAFETY: a Core Text font is immutable once created, and Core Text documents its font objects
// as safe to use from several threads at once.
unsafe impl Send for SharedFont {}
