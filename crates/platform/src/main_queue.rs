use std::cell::Cell;

use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::MainThreadMarker;

/// Runs `work` on the main thread on a later turn of the run loop, once what the application
/// is in the middle of now is over.
pub(crate) fn run_soon(work: Box<dyn FnOnce()>, main_thread: MainThreadMarker) {
    let work = MainThreadBound::new(Cell::new(Some(work)), main_thread);
    DispatchQueue::main().exec_async(move || {
        let main_thread = MainThreadMarker::new().expect("the main queue runs on the main thread");
        if let Some(work) = work.get(main_thread).take() {
            work();
        }
    });
}
