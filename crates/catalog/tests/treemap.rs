use morok::core::Rect;
use morok_catalog::{CHART_SIZE, TreemapTile, Volume};

/// How far a computed area may stray, in square CSS pixels.
const AREA_TOLERANCE: f32 = 0.5;
/// How far a computed edge may stray, in CSS pixels.
const EDGE_TOLERANCE: f32 = 0.01;
/// How far a computed share may stray.
const SHARE_TOLERANCE: f32 = 0.001;

fn area(rect: &Rect) -> f32 {
    rect.width * rect.height
}

/// The area two rectangles cover together.
fn overlap(first: &Rect, second: &Rect) -> f32 {
    let width = first.right().min(second.right()) - first.x.max(second.x);
    let height = first.bottom().min(second.bottom()) - first.y.max(second.y);
    width.max(0.0) * height.max(0.0)
}

fn lies_inside(inner: &Rect, outer: &Rect) -> bool {
    inner.x >= outer.x - EDGE_TOLERANCE
        && inner.y >= outer.y - EDGE_TOLERANCE
        && inner.right() <= outer.right() + EDGE_TOLERANCE
        && inner.bottom() <= outer.bottom() + EDGE_TOLERANCE
}

/// The tile of every folder of the volume with the tiles of the folders inside it.
fn tiles_by_folder(volume: Volume) -> Vec<(TreemapTile, Vec<TreemapTile>)> {
    let tiles = volume.treemap().tiles();
    let mut position = 0;
    volume
        .folders()
        .iter()
        .map(|folder| {
            let band = tiles[position];
            let inside = tiles[position + 1..=position + folder.children.len()].to_vec();
            position += 1 + folder.children.len();
            (band, inside)
        })
        .collect()
}

#[test]
fn the_tiles_of_each_volume_tile_the_chart_without_overlap() {
    let chart = Rect::new(0.0, 0.0, CHART_SIZE, CHART_SIZE);
    for volume in Volume::ALL {
        let tiles = volume.treemap().tiles();
        let covered: f32 = tiles.iter().map(|tile| area(&tile.rect)).sum();

        assert!(
            (covered - area(&chart)).abs() < AREA_TOLERANCE,
            "the tiles of {volume:?} cover the chart: {covered}"
        );
        for (position, tile) in tiles.iter().enumerate() {
            assert!(lies_inside(&tile.rect, &chart), "{volume:?}: {:?}", tile.rect);
            for other in &tiles[position + 1..] {
                assert!(
                    overlap(&tile.rect, &other.rect) < AREA_TOLERANCE,
                    "{volume:?}: {:?} and {:?} overlap",
                    tile.rect,
                    other.rect
                );
            }
        }
    }
}

#[test]
fn the_area_of_a_folder_is_its_share_of_the_used_space() {
    for volume in Volume::ALL {
        let used: f32 = volume.folders().iter().map(|folder| folder.gigabytes).sum();
        let areas = volume.treemap().folder_areas();

        assert_eq!(areas.len(), volume.folders().len());
        for (folder, folder_area) in volume.folders().iter().zip(&areas) {
            let share = area(folder_area) / (CHART_SIZE * CHART_SIZE);
            assert!(
                (share - folder.gigabytes / used).abs() < SHARE_TOLERANCE,
                "{} of {volume:?} takes {share} of the chart",
                folder.name
            );
        }
    }
}

#[test]
fn the_folders_inside_a_folder_share_its_rectangle_in_proportion() {
    for volume in Volume::ALL {
        let areas = volume.treemap().folder_areas();
        for ((folder, folder_area), (band, inside)) in volume.folders().iter().zip(&areas).zip(tiles_by_folder(volume))
        {
            assert!(lies_inside(&band.rect, folder_area), "{} of {volume:?}", folder.name);
            let shared: f32 = inside.iter().map(|tile| area(&tile.rect)).sum();
            let children: f32 = folder.children.iter().sum();
            for (tile, child) in inside.iter().zip(folder.children) {
                assert!(lies_inside(&tile.rect, folder_area), "{} of {volume:?}", folder.name);
                assert!(
                    (area(&tile.rect) / shared - child / children).abs() < SHARE_TOLERANCE,
                    "a folder inside {} of {volume:?} takes its share of the rest of the rectangle",
                    folder.name
                );
            }
            assert!(
                (area(&band.rect) + shared - area(folder_area)).abs() < AREA_TOLERANCE,
                "the band and the folders inside {} of {volume:?} fill its rectangle",
                folder.name
            );
        }
    }
}

#[test]
fn the_tiles_have_the_colors_and_the_opacities_of_the_sunburst() {
    for volume in Volume::ALL {
        let tiles: Vec<_> = volume
            .treemap()
            .tiles()
            .iter()
            .map(|tile| (tile.color, tile.opacity.to_bits()))
            .collect();
        let sectors: Vec<_> = volume
            .sunburst()
            .sectors()
            .iter()
            .map(|piece| (piece.color, piece.opacity.to_bits()))
            .collect();

        assert_eq!(tiles, sectors, "{volume:?}");
    }
}
