use crate::case_rendering::CaseRendering;
use crate::manifest::Manifest;

/// How Chrome draws a case, with what the case was made of when Chrome drew it.
#[derive(Clone, Debug, PartialEq)]
pub struct Etalon {
    pub manifest: Manifest,
    pub rendering: CaseRendering,
}
