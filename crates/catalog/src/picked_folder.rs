/// The folder the analysis of a volume picks out, as the disk map writes it beside the chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickedFolder {
    pub path: &'static str,
    /// Its size and how many files it holds.
    pub size: &'static str,
    /// The share of the used space of the volume it takes, from 0 to 1.
    pub share: f64,
    /// What the local model says of it.
    pub note: &'static str,
}
