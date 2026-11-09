use morok_platform::Renderer;

use crate::bench_error::BenchError;
use crate::budgets::Budgets;
use crate::frame_gate;
use crate::frame_pipeline::FramePipeline;
use crate::scene::Scene;
use crate::verdict::Verdict;

/// Measures the frames of the scene through the pipeline of the kit and holds them against
/// the frame budgets.
pub fn run(
    scenario: &str,
    scene: &mut dyn Scene,
    renderer: &Renderer,
    budgets: &Budgets,
) -> Result<Verdict, BenchError> {
    let mut pipeline = FramePipeline::new(renderer, scene)?;
    let measurements = pipeline.measure(scene, &budgets.frame);
    Ok(frame_gate::judge(scenario, &measurements, budgets))
}
