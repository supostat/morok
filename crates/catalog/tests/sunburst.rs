use std::f32::consts::{FRAC_PI_2, TAU};

use morok::core::Point;
use morok_catalog::{DiskMap, SUNBURST_SIZE};

/// The share of a full turn the used space of the volume takes.
const USED_SHARE: f32 = 0.85;
/// How far two computed angles may lie apart.
const ANGLE_TOLERANCE: f32 = 1e-4;

/// Whether the piece is a folder of the volume: those show their color whole.
fn is_folder_of_volume(opacity: f32) -> bool {
    (opacity - 1.0).abs() < f32::EPSILON
}

#[test]
fn the_folders_take_the_used_share_of_a_turn_clockwise_from_the_top() {
    let sectors = DiskMap::sunburst().sectors();
    let folders: Vec<_> = sectors
        .iter()
        .filter(|piece| is_folder_of_volume(piece.opacity))
        .collect();

    assert_eq!(folders.len(), 6);
    assert!(
        (folders[0].sector.start_angle - -FRAC_PI_2).abs() < 0.01,
        "the first folder starts at the top: {:?}",
        folders[0].sector
    );
    let last_end = folders[5].sector.end_angle;
    assert!(
        (last_end - (-FRAC_PI_2 + TAU * USED_SHARE)).abs() < 0.01,
        "the folders end where the used share of the turn ends: {last_end}"
    );
    let centers: Vec<Point> = sectors.iter().map(|piece| piece.sector.center).collect();
    assert!(
        centers.iter().all(|center| *center
            == Point {
                x: SUNBURST_SIZE / 2.0,
                y: SUNBURST_SIZE / 2.0
            }),
        "every piece is drawn around the middle of the square of the sunburst"
    );
}

#[test]
fn the_folders_inside_a_folder_share_its_angle_in_the_outer_ring_and_fade() {
    let sectors = DiskMap::sunburst().sectors();
    let first_folder = sectors[0];
    let inside: Vec<_> = sectors[1..]
        .iter()
        .take_while(|piece| !is_folder_of_volume(piece.opacity))
        .collect();

    assert_eq!(inside.len(), 2, "the first folder holds two folders");
    assert!(inside[0].sector.inner_radius > first_folder.sector.outer_radius);
    assert!(
        (inside[0].sector.start_angle - first_folder.sector.start_angle).abs() < ANGLE_TOLERANCE,
        "the first folder inside starts where its folder starts"
    );
    assert!(
        (inside[1].sector.end_angle - first_folder.sector.end_angle).abs() < ANGLE_TOLERANCE,
        "the last folder inside ends where its folder ends"
    );
    assert!(inside[0].opacity > inside[1].opacity);
    assert!(inside.iter().all(|piece| piece.color == first_folder.color));
}
