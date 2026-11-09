use crate::element_box::ElementBox;
use crate::mismatch::Mismatch;

/// The boxes of the candidate that lie further than `tolerance` from the etalon. The boxes of
/// both sides are in document order; trees of different sizes cannot be told apart box by box
/// and mismatch as a whole.
pub(crate) fn mismatches(etalon: &[ElementBox], candidate: &[ElementBox], tolerance: f32) -> Vec<Mismatch> {
    if etalon.len() != candidate.len() {
        return vec![Mismatch::ElementCount {
            etalon: etalon.len(),
            candidate: candidate.len(),
        }];
    }
    etalon
        .iter()
        .zip(candidate)
        .filter(|(etalon, candidate)| !is_within(etalon, candidate, tolerance))
        .map(|(etalon, candidate)| Mismatch::ElementBox {
            etalon: etalon.clone(),
            candidate: candidate.clone(),
        })
        .collect()
}

/// Whether each of the four edges lies no further than `tolerance` from the same edge of the etalon.
fn is_within(etalon: &ElementBox, candidate: &ElementBox, tolerance: f32) -> bool {
    let edges = |element_box: &ElementBox| {
        let rect = element_box.rect();
        [rect.x, rect.y, rect.right(), rect.bottom()]
    };
    edges(etalon)
        .into_iter()
        .zip(edges(candidate))
        .all(|(etalon_edge, candidate_edge)| (etalon_edge - candidate_edge).abs() <= tolerance)
}
