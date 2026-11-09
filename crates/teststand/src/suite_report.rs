use std::fmt::{Display, Formatter, Result};

use crate::case_verdict::CaseVerdict;
use crate::mismatch::Mismatch;

const CLEAN: u8 = 0;
const FAILURES: u8 = 1;

/// The verdicts of one run over a suite, in the order of its cases.
#[derive(Clone, Debug, PartialEq)]
pub struct SuiteReport {
    pub suite: String,
    /// The name of every case with its verdict.
    pub cases: Vec<(String, CaseVerdict)>,
}

impl SuiteReport {
    pub(crate) fn new(suite: &str) -> Self {
        Self {
            suite: suite.to_owned(),
            cases: Vec::new(),
        }
    }

    pub(crate) fn add(&mut self, case_name: &str, verdict: CaseVerdict) {
        self.cases.push((case_name.to_owned(), verdict));
    }

    /// The verdict of the named case.
    pub fn verdict(&self, case_name: &str) -> Option<&CaseVerdict> {
        self.cases
            .iter()
            .find(|(name, _)| name == case_name)
            .map(|(_, verdict)| verdict)
    }

    pub fn failure_count(&self) -> usize {
        self.cases.iter().filter(|(_, verdict)| verdict.is_failure()).count()
    }

    /// What the process of the stand exits with: zero when every case matches, one otherwise.
    pub fn exit_code(&self) -> u8 {
        if self.failure_count() == 0 { CLEAN } else { FAILURES }
    }
}

impl Display for SuiteReport {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        for (case_name, verdict) in &self.cases {
            let name = format!("{}/{case_name}", self.suite);
            match verdict {
                CaseVerdict::Matches => writeln!(formatter, "ok     {name}")?,
                CaseVerdict::MissingEtalon => writeln!(formatter, "FAIL   {name}: Chrome has not rendered the case")?,
                CaseVerdict::NoCase => writeln!(formatter, "FAIL   {name}: the suite has no such case")?,
                CaseVerdict::Stale { changed } => writeln!(
                    formatter,
                    "STALE  {name}: changed since Chrome rendered it: {}",
                    changed.join(", ")
                )?,
                CaseVerdict::Differs { light, dark } => {
                    writeln!(formatter, "FAIL   {name}")?;
                    write_mismatches(formatter, "light", light)?;
                    write_mismatches(formatter, "dark", dark)?;
                }
            }
        }
        write!(
            formatter,
            "{}: {} of {} cases failed",
            self.suite,
            self.failure_count(),
            self.cases.len()
        )
    }
}

fn write_mismatches(formatter: &mut Formatter<'_>, color_scheme: &str, mismatches: &[Mismatch]) -> Result {
    mismatches
        .iter()
        .try_for_each(|mismatch| writeln!(formatter, "         {color_scheme}: {mismatch}"))
}
