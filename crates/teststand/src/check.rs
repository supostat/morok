use std::fs;
use std::io;
use std::path::Path;

use crate::case_rendering::COLOR_SCHEMES;
use crate::case_verdict::CaseVerdict;
use crate::chrome_build::ChromeBuild;
use crate::comparison::compare;
use crate::compiled_suite::CompiledSuite;
use crate::diff_image;
use crate::etalon_store::EtalonStore;
use crate::frame_png;
use crate::morok_side::MorokSide;
use crate::stand::Stand;
use crate::stand_error::StandError;
use crate::suite::Suite;
use crate::suite_report::SuiteReport;
use crate::thresholds::Thresholds;

/// Compares what the kit draws of every case of the suite with the etalon Chrome left of it.
/// Chrome is not needed. A case that mismatches leaves a diff image per mismatching color
/// scheme in the diffs directory of the stand, named `<suite>/<case>-<scheme>.png`; the images
/// of an earlier run are dropped first.
pub fn check(suite: &Suite, stand: &Stand) -> Result<SuiteReport, StandError> {
    let thresholds = Thresholds::read(&stand.thresholds_file)?;
    let chrome_build = ChromeBuild::read(&stand.chrome_build_file)?;
    let compiled = CompiledSuite::compile(suite)?;
    let store = EtalonStore::new(&stand.etalons_directory);
    let diffs_directory = stand.diffs_directory.join(suite.name);
    recreate(&diffs_directory)?;
    let mut morok = MorokSide::new()?;
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
        let candidate = morok.render_case(&compiled, case);
        let [light, dark] = COLOR_SCHEMES.map(|color_scheme| {
            compare(
                etalon.rendering.under(color_scheme),
                candidate.under(color_scheme),
                &thresholds,
            )
        });
        for (color_scheme, comparison) in COLOR_SCHEMES.into_iter().zip([&light, &dark]) {
            if !comparison.matches() {
                let path = diffs_directory.join(format!("{}-{color_scheme}.png", case.name));
                let diff = diff_image::of(
                    etalon.rendering.under(color_scheme),
                    candidate.under(color_scheme),
                    comparison,
                );
                fs::write(&path, frame_png::encode(&diff)).map_err(|error| file_error(&path, error))?;
            }
        }
        report.add(case.name, CaseVerdict::of_comparisons(&light, &dark));
    }
    Ok(report)
}

/// Leaves an empty directory at the path.
fn recreate(directory: &Path) -> Result<(), StandError> {
    if directory.exists() {
        fs::remove_dir_all(directory).map_err(|error| file_error(directory, error))?;
    }
    fs::create_dir_all(directory).map_err(|error| file_error(directory, error))
}

fn file_error(path: &Path, error: io::Error) -> StandError {
    StandError::File {
        path: path.to_path_buf(),
        error,
    }
}
