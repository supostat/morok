use std::f32::consts::{FRAC_PI_2, TAU};

use morok::core::Point;
use morok_catalog::{CHART_SIZE, SunburstSector, Volume};

/// The share of a full turn the used space of a volume takes.
const USED_SHARE: f32 = 0.85;
/// How far two computed angles may lie apart: a piece gives up a little of each end so that
/// neighbours stand apart.
const ANGLE_TOLERANCE: f32 = 0.01;

/// Whether the piece is a folder of the volume: those show their color whole.
fn is_folder_of_volume(piece: &SunburstSector) -> bool {
    (piece.opacity - 1.0).abs() < f32::EPSILON
}

fn are_near(first: f32, second: f32) -> bool {
    (first - second).abs() < ANGLE_TOLERANCE
}

#[test]
fn the_folders_of_each_volume_take_their_shares_of_the_used_turn_clockwise_from_the_top() {
    for volume in Volume::ALL {
        let sectors = volume.sunburst().sectors();
        let folders: Vec<_> = sectors.iter().filter(|piece| is_folder_of_volume(piece)).collect();
        let used: f32 = volume.folders().iter().map(|folder| folder.gigabytes).sum();

        assert_eq!(folders.len(), volume.folders().len(), "{volume:?}");
        assert!(
            are_near(folders[0].sector.start_angle, -FRAC_PI_2),
            "the first folder of {volume:?} starts at the top: {:?}",
            folders[0].sector
        );
        let mut start = -FRAC_PI_2;
        for (piece, folder) in folders.iter().zip(volume.folders()) {
            start += folder.gigabytes / used * TAU * USED_SHARE;
            assert!(
                are_near(piece.sector.end_angle, start) && piece.color == folder.color,
                "{} of {volume:?} ends where its share of the used turn ends: {:?}",
                folder.name,
                piece.sector
            );
        }
        assert!(
            are_near(start, -FRAC_PI_2 + TAU * USED_SHARE),
            "the folders of {volume:?} end where the used share of the turn ends"
        );
        let middle = Point {
            x: CHART_SIZE / 2.0,
            y: CHART_SIZE / 2.0,
        };
        assert!(
            sectors.iter().all(|piece| piece.sector.center == middle),
            "every piece is drawn around the middle of the square of the chart"
        );
    }
}

#[test]
fn the_folders_inside_a_folder_share_its_angle_in_the_outer_ring_and_fade() {
    for volume in Volume::ALL {
        let sectors = volume.sunburst().sectors();
        let mut position = 0;
        for folder in volume.folders() {
            let outer = sectors[position];
            let inside = &sectors[position + 1..=position + folder.children.len()];
            position += 1 + folder.children.len();

            assert!(is_folder_of_volume(&outer), "{} of {volume:?}", folder.name);
            assert!(inside.iter().all(|piece| !is_folder_of_volume(piece)
                && piece.color == outer.color
                && piece.sector.inner_radius > outer.sector.outer_radius));
            assert!(
                are_near(inside[0].sector.start_angle, outer.sector.start_angle)
                    && are_near(inside[inside.len() - 1].sector.end_angle, outer.sector.end_angle),
                "the folders inside {} of {volume:?} share its angle",
                folder.name
            );
            assert!(
                inside.windows(2).all(|pair| pair[0].opacity > pair[1].opacity),
                "every next folder inside {} shows less of its color",
                folder.name
            );
        }
        assert_eq!(position, sectors.len());
    }
}
