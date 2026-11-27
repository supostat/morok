#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;

use morok_teststand::{Stand, StandError, Suite, SuiteReport, check, etalons, frames, suites};

const USAGE: &str = "usage: morok-teststand <etalons render <suite> | etalons verify <suite> | check <suite> | \
                     frames <list>>, `all` for every suite";
const CLEAN: u8 = 0;
/// The command names nothing the stand has, or the stand could not do its work.
const COULD_NOT_RUN: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let stand = Stand::of_workspace();
    let outcome = match arguments.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["etalons", "render", suite] => on_suite(suite, |suite| {
            let rendered = etalons::render(suite, &stand)?;
            println!("{}: {rendered} etalons rendered", suite.name);
            Ok(CLEAN)
        }),
        ["etalons", "verify", suite] => on_suite(suite, |suite| {
            etalons::verify(suite, &stand).map(|report| reported(&report))
        }),
        ["check", suite] => on_suite(suite, |suite| check(suite, &stand).map(|report| reported(&report))),
        ["frames", list] => frames::write(Path::new(list)).map(|written| {
            println!("{written} frames written");
            CLEAN
        }),
        _ => {
            eprintln!("{USAGE}");
            Ok(COULD_NOT_RUN)
        }
    };
    match outcome {
        Ok(exit_code) => ExitCode::from(exit_code),
        Err(error) => {
            eprintln!("morok-teststand: {error}");
            ExitCode::from(COULD_NOT_RUN)
        }
    }
}

/// Runs the command on the named suite, or on every suite one after another for
/// [`suites::ALL`]: the process exits with the worst of their codes. A name that is no suite is
/// a command the stand cannot run.
fn on_suite(name: &str, mut command: impl FnMut(&Suite) -> Result<u8, StandError>) -> Result<u8, StandError> {
    if name == suites::ALL {
        let mut worst = CLEAN;
        for suite_name in suites::NAMES {
            let suite = suites::named(suite_name).expect("every name of a suite names a suite");
            worst = worst.max(command(&suite)?);
        }
        return Ok(worst);
    }
    if let Some(suite) = suites::named(name) {
        command(&suite)
    } else {
        eprintln!(
            "no suite is named `{name}`; the suites are: {}, or `{}` for all of them",
            suites::NAMES.join(", "),
            suites::ALL
        );
        Ok(COULD_NOT_RUN)
    }
}

/// Prints the report and returns what the process exits with.
fn reported(report: &SuiteReport) -> u8 {
    println!("{report}");
    report.exit_code()
}
