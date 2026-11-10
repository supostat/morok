use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::bench_error::BenchError;
use crate::cold_start_application::{COMMAND, FIRST_FRAME_LINE};

/// Starts the application of the harness as a process of its own and returns how long it took
/// from asking the system for the process to the application saying that its first frame was
/// presented. The clock runs outside the process, so the time holds everything a start costs:
/// loading the binary and its libraries, the application and its renderer, the window and the
/// frame.
pub(crate) fn time_to_first_frame() -> Result<Duration, BenchError> {
    let executable = env::current_exe().map_err(|error| BenchError::NoLaunch {
        executable: "morok-bench".into(),
        error,
    })?;
    let no_launch = |error| BenchError::NoLaunch {
        executable: executable.clone(),
        error,
    };
    let started = Instant::now();
    let mut application = Command::new(&executable)
        .arg(COMMAND)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(no_launch)?;
    let output = application.stdout.take().expect("the output of the process is piped");
    let mut first_line = String::new();
    BufReader::new(output).read_line(&mut first_line).map_err(no_launch)?;
    let elapsed = started.elapsed();
    application.wait().map_err(no_launch)?;
    if first_line.trim_end() == FIRST_FRAME_LINE {
        Ok(elapsed)
    } else {
        Err(BenchError::NoFirstFrame { waited: elapsed })
    }
}
