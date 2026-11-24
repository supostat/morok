#![forbid(unsafe_code)]

use std::process::ExitCode;

use morok_bench::{Budgets, cold_start_application, scenarios};

const USAGE: &str = "usage: morok-bench <frame | idle | cold-start | input-latency | scroll-1m | scroll-list | all>";
const WITHIN_BUDGET: u8 = 0;
/// The command names no scenario, or the harness could not do its work.
const COULD_NOT_RUN: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [name] = arguments.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(COULD_NOT_RUN);
    };
    if name == cold_start_application::COMMAND {
        return cold_start_application::run();
    }
    let Some(scenario_names) = scenarios::named(name) else {
        eprintln!(
            "no scenario is named `{name}`; the scenarios are: {}, or `{}` for all of them",
            scenarios::NAMES.join(", "),
            scenarios::ALL
        );
        return ExitCode::from(COULD_NOT_RUN);
    };
    let budgets = match Budgets::of_workspace() {
        Ok(budgets) => budgets,
        Err(error) => {
            eprintln!("morok-bench: {error}");
            return ExitCode::from(COULD_NOT_RUN);
        }
    };
    let mut worst = WITHIN_BUDGET;
    for scenario in scenario_names {
        match scenarios::run(scenario, &budgets) {
            Ok(verdicts) => {
                for verdict in &verdicts {
                    println!("{verdict}");
                    worst = worst.max(verdict.exit_code());
                }
            }
            Err(error) => {
                eprintln!("morok-bench: {scenario}: {error}");
                worst = COULD_NOT_RUN;
            }
        }
    }
    ExitCode::from(worst)
}
