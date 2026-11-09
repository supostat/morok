#![forbid(unsafe_code)]

use std::process::ExitCode;

use morok_bench::{Budgets, scenarios};

const USAGE: &str = "usage: morok-bench <frame | idle>";
/// The command names no scenario, or the harness could not do its work.
const COULD_NOT_RUN: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [name] = arguments.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(COULD_NOT_RUN);
    };
    if !scenarios::NAMES.contains(&name.as_str()) {
        eprintln!(
            "no scenario is named `{name}`; the scenarios are: {}",
            scenarios::NAMES.join(", ")
        );
        return ExitCode::from(COULD_NOT_RUN);
    }
    match Budgets::of_workspace().and_then(|budgets| scenarios::run(name, &budgets)) {
        Ok(verdict) => {
            println!("{verdict}");
            ExitCode::from(verdict.exit_code())
        }
        Err(error) => {
            eprintln!("morok-bench: {error}");
            ExitCode::from(COULD_NOT_RUN)
        }
    }
}
