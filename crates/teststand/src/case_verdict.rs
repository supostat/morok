use crate::comparison::Comparison;
use crate::mismatch::Mismatch;

/// What a run of the stand found about one case.
#[derive(Clone, Debug, PartialEq)]
pub enum CaseVerdict {
    Matches,
    /// Chrome has never rendered the case.
    MissingEtalon,
    /// The store holds an etalon of this name and the suite no case.
    NoCase,
    /// The case changed since Chrome rendered it: the named inputs of the etalon hash
    /// differently now. Nothing was compared.
    Stale {
        changed: Vec<&'static str>,
    },
    /// The mismatches under each color scheme; at least one scheme has some.
    Differs {
        light: Vec<Mismatch>,
        dark: Vec<Mismatch>,
    },
}

impl CaseVerdict {
    pub(crate) fn of_comparisons(light: &Comparison, dark: &Comparison) -> Self {
        Self::of_mismatches(light.mismatches.clone(), dark.mismatches.clone())
    }

    pub(crate) fn of_mismatches(light: Vec<Mismatch>, dark: Vec<Mismatch>) -> Self {
        if light.is_empty() && dark.is_empty() {
            Self::Matches
        } else {
            Self::Differs { light, dark }
        }
    }

    pub fn is_failure(&self) -> bool {
        *self != Self::Matches
    }
}
