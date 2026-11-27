use std::path::PathBuf;

use morok_style::ColorScheme;

use crate::case::Case;

/// One frame a list asks for: a case of a suite of the stand under a color scheme, and the
/// file that takes the frame.
#[derive(Clone, Debug)]
pub(crate) struct ListedFrame {
    pub(crate) suite_name: &'static str,
    pub(crate) case: Case,
    pub(crate) color_scheme: ColorScheme,
    pub(crate) path: PathBuf,
}
