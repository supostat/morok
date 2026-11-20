use morok_catalog::Page;
use morok_platform::Renderer;

use crate::bench_error::BenchError;
use crate::budgets::Budgets;
use crate::frame_gate;
use crate::reference_scene::ReferenceScene;
use crate::screen_frames::ScreenFrames;
use crate::verdict::Verdict;
use crate::{catalog_idle, cold_start_scenario, frame_scenario, input_latency_scenario, scroll_scenario};

const FRAME: &str = "frame";
const IDLE: &str = "idle";
const COLD_START: &str = "cold-start";
const INPUT_LATENCY: &str = "input-latency";
const SCROLL_1M: &str = "scroll-1m";

/// The name that stands for every scenario, run one after another.
pub const ALL: &str = "all";
/// The scenarios the harness runs by name.
pub const NAMES: [&str; 5] = [FRAME, IDLE, COLD_START, INPUT_LATENCY, SCROLL_1M];

/// Runs the scenario of one of [`NAMES`] against the budgets: a verdict for every scene it
/// measures. The scenarios run on the scenes of the catalog; `frame` measures the reference
/// scene besides.
///
/// # Panics
///
/// When the name is none of them: the command line is checked against the names first.
pub fn run(name: &str, budgets: &Budgets) -> Result<Vec<Verdict>, BenchError> {
    match name {
        FRAME => frame(budgets),
        IDLE => Ok(vec![catalog_idle::run(IDLE, &budgets.idle)?]),
        COLD_START => Ok(vec![cold_start_scenario::run(COLD_START, &budgets.cold_start)?]),
        INPUT_LATENCY => Ok(vec![input_latency_scenario::run(
            INPUT_LATENCY,
            &budgets.input_latency,
        )?]),
        SCROLL_1M => Ok(vec![scroll_scenario::run(SCROLL_1M, &Renderer::new()?, budgets)]),
        _ => panic!("no scenario is named `{name}`"),
    }
}

/// The scenarios a name on the command line stands for: [`ALL`] for every one of them, in the
/// order of [`NAMES`]; none for a name the harness does not know.
pub fn named(name: &str) -> Option<Vec<&'static str>> {
    if name == ALL {
        return Some(NAMES.to_vec());
    }
    NAMES.iter().find(|known| **known == name).map(|known| vec![*known])
}

/// The frames of the reference scene, then those of the two screens of the catalog.
fn frame(budgets: &Budgets) -> Result<Vec<Verdict>, BenchError> {
    let renderer = Renderer::new()?;
    let mut verdicts = vec![frame_scenario::run(
        FRAME,
        &mut ReferenceScene::changing(),
        &renderer,
        budgets,
    )?];
    for (scene, screen) in [
        ("frame/file-manager", Page::FileManager),
        ("frame/disk-map", Page::DiskMap),
    ] {
        let measurements = ScreenFrames::new(&renderer, screen).measure(&budgets.frame);
        verdicts.push(frame_gate::judge(scene, &measurements, budgets));
    }
    Ok(verdicts)
}
