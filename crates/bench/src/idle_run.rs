use std::time::Duration;

use morok_platform::{Application, Window, WindowOptions};

use crate::bench_error::BenchError;
use crate::idle_observation::IdleObservation;
use crate::scene::Scene;
use crate::scene_content::SceneContent;

/// How long a window gets to come on screen and present its first frame.
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(5);
/// The run loop is given this long at a time while the first frame is waited for.
const FIRST_FRAME_POLL: Duration = Duration::from_millis(100);
/// After the first frame, the time the window gets to finish opening before it counts as idle.
const SETTLING: Duration = Duration::from_secs(1);

/// Opens a window on the scene, lets it settle — whatever motion it opens with ends in that
/// time — then leaves it without events for `idle` and reports what its frame loop did.
pub(crate) fn observe(
    application: &Application,
    scene: Box<dyn Scene>,
    idle: Duration,
) -> Result<IdleObservation, BenchError> {
    let options = WindowOptions {
        title: String::from("morok: bench"),
        content_size: scene.viewport(),
    };
    let mut content = SceneContent::new(scene, application.surface_settings())?;
    let window = Window::open(application, &options, |frame_scheduler, _| {
        content.set_frame_scheduler(frame_scheduler);
        content
    });
    wait_for_first_frame(application, &window)?;
    application.run_for(SETTLING);
    let settled = window.frame_statistics();
    application.run_for(idle);
    let after_idle = window.frame_statistics();
    Ok(IdleObservation {
        settling_frames: settled.presented_frames,
        duration: idle,
        presented_frames: after_idle.presented_frames - settled.presented_frames,
        display_link_callbacks: after_idle.display_link_callbacks - settled.display_link_callbacks,
    })
}

fn wait_for_first_frame(application: &Application, window: &Window<SceneContent>) -> Result<(), BenchError> {
    let mut waited = Duration::ZERO;
    while window.frame_statistics().presented_frames == 0 {
        if waited >= FIRST_FRAME_TIMEOUT {
            return Err(BenchError::NoFirstFrame { waited });
        }
        application.run_for(FIRST_FRAME_POLL);
        waited += FIRST_FRAME_POLL;
    }
    Ok(())
}
