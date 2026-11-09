#![forbid(unsafe_code)]

use std::process::ExitCode;

use morok_teststand::{Stand, StandError, Suite, SuiteReport, check, etalons, suites};

const USAGE: &str = "usage: morok-teststand <etalons render <suite> | etalons verify <suite> | check <suite>>";
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

/// Runs the command on the named suite; a name that is no suite is a command the stand cannot run.
fn on_suite(name: &str, command: impl FnOnce(&Suite) -> Result<u8, StandError>) -> Result<u8, StandError> {
    if let Some(suite) = suites::named(name) {
        command(&suite)
    } else {
        eprintln!(
            "no suite is named `{name}`; the suites are: {}",
            suites::NAMES.join(", ")
        );
        Ok(COULD_NOT_RUN)
    }
}

/// Prints the report and returns what the process exits with.
fn reported(report: &SuiteReport) -> u8 {
    println!("{report}");
    report.exit_code()
}
