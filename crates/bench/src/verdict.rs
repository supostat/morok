use std::fmt::{Display, Formatter, Result};

use crate::check::Check;

const WITHIN_BUDGET: u8 = 0;
const OVER_BUDGET: u8 = 1;

/// What a scenario measured, each value against its budget, and what the process exits with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    scenario: String,
    checks: Vec<Check>,
    /// Observations that are reported but not gated.
    notes: Vec<String>,
}

impl Verdict {
    pub fn new(scenario: &str, checks: Vec<Check>, notes: Vec<String>) -> Self {
        Self {
            scenario: scenario.to_owned(),
            checks,
            notes,
        }
    }

    /// Whether every check holds.
    pub fn holds(&self) -> bool {
        self.checks.iter().all(|check| check.holds)
    }

    pub fn exit_code(&self) -> u8 {
        if self.holds() { WITHIN_BUDGET } else { OVER_BUDGET }
    }

    fn failing_subjects(&self) -> Vec<&'static str> {
        self.checks
            .iter()
            .filter(|check| !check.holds)
            .map(|check| check.subject)
            .collect()
    }
}

impl Display for Verdict {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let scenario = &self.scenario;
        for check in &self.checks {
            let outcome = if check.holds { "ok" } else { "OVER BUDGET" };
            writeln!(
                formatter,
                "{scenario}: {:<16} {:<22} budget {:<10} {outcome}",
                check.subject, check.measured, check.budget
            )?;
        }
        for note in &self.notes {
            writeln!(formatter, "{scenario}: {note}")?;
        }
        if self.holds() {
            write!(formatter, "{scenario}: every budget holds")
        } else {
            write!(
                formatter,
                "{scenario}: over budget: {}",
                self.failing_subjects().join(", ")
            )
        }
    }
}
