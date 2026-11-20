use std::process::ExitCode;
use std::time::{Duration, Instant};

use morok_catalog::{Catalog, Page};

/// The command of the harness binary that runs the application instead of a scenario.
pub const COMMAND: &str = "cold-start-application";
/// What the application writes to its standard output once its first frame is presented.
pub(crate) const FIRST_FRAME_LINE: &str = "first frame presented";

/// How long the window gets to present its first frame.
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(5);
/// The event loop is given this long at a time while the first frame is waited for.
const FIRST_FRAME_POLL: Duration = Duration::from_millis(1);
const NO_FIRST_FRAME: u8 = 1;
const COULD_NOT_RUN: u8 = 2;

/// The application whose start the `cold-start` scenario measures: the catalog, which opens
/// its window on its first page; it says so on its standard output once the first frame of the
/// window is presented, and ends.
pub fn run() -> ExitCode {
    let catalog = match Catalog::launch(Page::Buttons) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("morok-bench: {error}");
            return ExitCode::from(COULD_NOT_RUN);
        }
    };
    let opened = Instant::now();
    while catalog.windows()[0].frame_statistics().presented_frames == 0 {
        if opened.elapsed() >= FIRST_FRAME_TIMEOUT {
            return ExitCode::from(NO_FIRST_FRAME);
        }
        catalog.application().run_for(FIRST_FRAME_POLL);
    }
    println!("{FIRST_FRAME_LINE}");
    ExitCode::SUCCESS
}
