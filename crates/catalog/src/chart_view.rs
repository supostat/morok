/// The side of the square a chart of the disk map is drawn in, in CSS pixels.
pub const CHART_SIZE: f32 = 300.0;

/// How the disk map draws the folders of a volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartView {
    /// Rectangles that tile the chart, a folder's own shared by the folders inside it.
    Treemap,
    /// Rings around a hole, the folders inside a folder around it.
    Sunburst,
}

impl ChartView {
    pub const ALL: [Self; 2] = [Self::Treemap, Self::Sunburst];

    pub fn title(self) -> &'static str {
        match self {
            Self::Treemap => "Treemap",
            Self::Sunburst => "Sunburst",
        }
    }
}
