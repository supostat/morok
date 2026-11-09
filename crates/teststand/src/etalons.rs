//! The two commands that need Chrome: rendering the etalons of a suite and verifying that
//! the pinned build still renders them the same.

use morok_platform::FrameImage;

use crate::case_rendering::COLOR_SCHEMES;
use crate::case_verdict::CaseVerdict;
use crate::chrome_build::ChromeBuild;
use crate::chrome_side::ChromeSide;
use crate::compiled_suite::CompiledSuite;
use crate::etalon::Etalon;
use crate::etalon_store::EtalonStore;
use crate::geometry_gate;
use crate::mismatch::Mismatch;
use crate::rendering::Rendering;
use crate::stand::Stand;
use crate::stand_error::StandError;
use crate::suite::Suite;
use crate::suite_report::SuiteReport;

const CHANNELS: usize = 4;
/// An etalon is rendered again without any tolerance.
const NO_TOLERANCE: f32 = 0.0;

/// Renders every case of the suite in the pinned Chrome and stores the results as the etalons
/// of the suite; an etalon whose case the suite no longer has is dropped. Returns the number
/// of etalons written.
pub fn render(suite: &Suite, stand: &Stand) -> Result<usize, StandError> {
    let chrome_build = ChromeBuild::read(&stand.chrome_build_file)?;
    let compiled = CompiledSuite::compile(suite)?;
    let store = EtalonStore::new(&stand.etalons_directory);
    let mut chrome = ChromeSide::launch(&chrome_build)?;
    for case in &suite.cases {
        let etalon = Etalon {
            manifest: compiled.manifest(case, &chrome_build),
            rendering: chrome.render_case(&compiled, case)?,
        };
        store.write(suite.name, case.name, &etalon)?;
    }
    chrome.close()?;
    for case_name in caseless_etalons(suite, &store)? {
        store.remove(suite.name, &case_name)?;
    }
    Ok(suite.cases.len())
}

/// Renders every case of the suite in the pinned Chrome again and reports each difference
/// from the stored etalons: a case without an etalon, an etalon without a case, an etalon made
/// of other inputs than the case has now, a pixel or a box that came out differently.
pub fn verify(suite: &Suite, stand: &Stand) -> Result<SuiteReport, StandError> {
    let chrome_build = ChromeBuild::read(&stand.chrome_build_file)?;
    let compiled = CompiledSuite::compile(suite)?;
    let store = EtalonStore::new(&stand.etalons_directory);
    let mut chrome = ChromeSide::launch(&chrome_build)?;
    let mut report = SuiteReport::new(suite.name);
    for case in &suite.cases {
        let Some(etalon) = store.read(suite.name, case.name)? else {
            report.add(case.name, CaseVerdict::MissingEtalon);
            continue;
        };
        let changed = etalon.manifest.changed_inputs(&compiled.manifest(case, &chrome_build));
        if !changed.is_empty() {
            report.add(case.name, CaseVerdict::Stale { changed });
            continue;
        }
        let rendered_again = chrome.render_case(&compiled, case)?;
        let [light, dark] = COLOR_SCHEMES
            .map(|color_scheme| differences(etalon.rendering.under(color_scheme), rendered_again.under(color_scheme)));
        report.add(case.name, CaseVerdict::of_mismatches(light, dark));
    }
    chrome.close()?;
    for case_name in caseless_etalons(suite, &store)? {
        report.add(&case_name, CaseVerdict::NoCase);
    }
    Ok(report)
}

/// The names of the stored etalons of the suite that belong to none of its cases.
fn caseless_etalons(suite: &Suite, store: &EtalonStore) -> Result<Vec<String>, StandError> {
    let mut case_names = store.case_names(suite.name)?;
    case_names.retain(|stored| suite.cases.iter().all(|case| case.name != stored));
    Ok(case_names)
}

/// Every difference between an etalon and the same case rendered again.
fn differences(etalon: &Rendering, rendered_again: &Rendering) -> Vec<Mismatch> {
    let mut differences = geometry_gate::mismatches(&etalon.boxes, &rendered_again.boxes, NO_TOLERANCE);
    let size = |image: &FrameImage| (image.width, image.height);
    if size(&etalon.image) != size(&rendered_again.image) {
        differences.push(Mismatch::ImageSize {
            etalon: size(&etalon.image),
            candidate: size(&rendered_again.image),
        });
        return differences;
    }
    let different_pixels = etalon
        .image
        .pixels
        .chunks_exact(CHANNELS)
        .zip(rendered_again.image.pixels.chunks_exact(CHANNELS))
        .filter(|(etalon_pixel, pixel)| etalon_pixel != pixel)
        .count();
    if different_pixels > 0 {
        differences.push(Mismatch::MismatchingPixels {
            pixels: different_pixels,
            area: etalon.image.width * etalon.image.height,
        });
    }
    differences
}
