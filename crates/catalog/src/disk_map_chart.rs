use crate::sunburst_sector::SunburstSector;
use crate::treemap_tile::TreemapTile;

/// The pieces of one chart of the disk map, worked out once and drawn in every frame.
#[derive(Clone, Debug)]
pub(crate) enum DiskMapChart {
    Sunburst(Vec<SunburstSector>),
    Treemap(Vec<TreemapTile>),
}
