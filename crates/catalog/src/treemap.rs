use morok::core::Rect;

use crate::chart_view::CHART_SIZE;
use crate::folder_usage::{FOLDER_OPACITY, FolderUsage};
use crate::treemap_tile::TreemapTile;

/// How high the band along the top of a folder that shows the folder itself is at most; the
/// folders inside it share the rest of its rectangle.
const FOLDER_BAND: f32 = 8.0;
/// The share of the height of a folder its band takes at most, for a folder that is low.
const FOLDER_BAND_SHARE: f32 = 0.25;

/// The folders of a volume as rectangles that tile a square: each folder of the volume takes
/// its share of the used space, a band along its top shows the folder itself, and the folders
/// inside it share the rest of its rectangle. The rectangles are laid out squarified, as near
/// to squares as their shares let them.
#[derive(Clone, Copy, Debug)]
pub struct Treemap {
    folders: &'static [FolderUsage],
}

impl Treemap {
    pub fn new(folders: &'static [FolderUsage]) -> Self {
        Self { folders }
    }

    /// The rectangle of every folder of the volume in a square of [`CHART_SIZE`] pixels, in
    /// the order of the folders.
    pub fn folder_areas(&self) -> Vec<Rect> {
        let sizes: Vec<f32> = self.folders.iter().map(|folder| folder.gigabytes).collect();
        squarify(&sizes, Rect::new(0.0, 0.0, CHART_SIZE, CHART_SIZE))
    }

    /// The tiles of the treemap in the order they are drawn: the band of every folder of the
    /// volume followed by the folders inside it.
    pub fn tiles(&self) -> Vec<TreemapTile> {
        let mut tiles = Vec::new();
        for (folder, area) in self.folders.iter().zip(self.folder_areas()) {
            let band_height = FOLDER_BAND.min(area.height * FOLDER_BAND_SHARE);
            tiles.push(TreemapTile {
                rect: Rect::new(area.x, area.y, area.width, band_height),
                color: folder.color,
                opacity: FOLDER_OPACITY,
            });
            let inside = Rect::new(area.x, area.y + band_height, area.width, area.height - band_height);
            for (position, rect) in squarify(folder.children, inside).into_iter().enumerate() {
                tiles.push(TreemapTile {
                    rect,
                    color: folder.color,
                    opacity: FolderUsage::child_opacity(position),
                });
            }
        }
        tiles
    }
}

/// Rectangles that tile `bounds`, one for every size in its order, each with the share of
/// the area its size has of all of them. The largest sizes are laid out first, a row at a
/// time along the shorter side of what is left, and a size joins a row while it brings the
/// rectangles of the row nearer to squares.
fn squarify(sizes: &[f32], bounds: Rect) -> Vec<Rect> {
    let total: f32 = sizes.iter().sum();
    let area_per_size = bounds.width * bounds.height / total;
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by(|first, second| sizes[*second].total_cmp(&sizes[*first]));
    let mut rects = vec![Rect::ZERO; sizes.len()];
    let mut remaining = bounds;
    let mut row: Vec<usize> = Vec::new();
    for index in order {
        let side = remaining.width.min(remaining.height);
        let areas_of = |row: &[usize]| {
            row.iter()
                .map(|index| sizes[*index] * area_per_size)
                .collect::<Vec<_>>()
        };
        let mut grown = row.clone();
        grown.push(index);
        if row.is_empty() || worst_ratio(&areas_of(&grown), side) <= worst_ratio(&areas_of(&row), side) {
            row = grown;
        } else {
            remaining = lay_out_row(&row, &areas_of(&row), remaining, &mut rects);
            row = vec![index];
        }
    }
    let areas: Vec<f32> = row.iter().map(|index| sizes[*index] * area_per_size).collect();
    lay_out_row(&row, &areas, remaining, &mut rects);
    rects
}

/// The longest side over the shortest of the most stretched rectangle of a row of `areas`
/// laid out along a side `side` long.
fn worst_ratio(areas: &[f32], side: f32) -> f32 {
    let row_area: f32 = areas.iter().sum();
    let side_squared = side * side;
    let row_area_squared = row_area * row_area;
    areas
        .iter()
        .map(|area| (side_squared * area / row_area_squared).max(row_area_squared / (side_squared * area)))
        .fold(0.0, f32::max)
}

/// Lays the row out along the shorter side of `remaining`, writes the rectangle of every
/// index of the row into `rects` and returns what is left of `remaining`.
fn lay_out_row(row: &[usize], areas: &[f32], remaining: Rect, rects: &mut [Rect]) -> Rect {
    let row_area: f32 = areas.iter().sum();
    if remaining.width >= remaining.height {
        let width = row_area / remaining.height;
        let mut y = remaining.y;
        for (index, area) in row.iter().zip(areas) {
            let height = area / width;
            rects[*index] = Rect::new(remaining.x, y, width, height);
            y += height;
        }
        Rect::new(
            remaining.x + width,
            remaining.y,
            remaining.width - width,
            remaining.height,
        )
    } else {
        let height = row_area / remaining.width;
        let mut x = remaining.x;
        for (index, area) in row.iter().zip(areas) {
            let width = area / height;
            rects[*index] = Rect::new(x, remaining.y, width, height);
            x += width;
        }
        Rect::new(
            remaining.x,
            remaining.y + height,
            remaining.width,
            remaining.height - height,
        )
    }
}
