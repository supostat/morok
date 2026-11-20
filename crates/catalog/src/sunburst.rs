use std::f32::consts::{FRAC_PI_2, TAU};

use morok::RingSector;
use morok::core::Point;

use crate::folder_usage::FolderUsage;
use crate::sunburst_sector::SunburstSector;

/// The side of the square a sunburst is drawn in, in CSS pixels.
pub const SUNBURST_SIZE: f32 = 300.0;
/// The ring of the folders of the volume.
const FOLDER_RADII: (f32, f32) = (62.0, 100.0);
/// The ring of the folders inside them, around it.
const CHILD_RADII: (f32, f32) = (103.0, 140.0);
/// The share of a full turn the used space takes: the rest of the turn is the free space.
const USED_SHARE: f32 = 0.85;
/// The angle cut off each end of a piece, so that neighbours stand apart.
const SEPARATION: f32 = 0.004;
/// The opacity of the first folder inside a folder; every next one shows less.
const FIRST_CHILD_OPACITY: f32 = 0.75;
const CHILD_OPACITY_STEP: f32 = 0.15;

/// The folders of a volume as rings around a hole: the folders of the volume in the inner ring,
/// each taking its share of the used space, and the folders inside each in the outer ring,
/// within the angle of their folder. The pieces run clockwise from the top.
#[derive(Clone, Copy, Debug)]
pub struct Sunburst {
    folders: &'static [FolderUsage],
}

impl Sunburst {
    pub fn new(folders: &'static [FolderUsage]) -> Self {
        Self { folders }
    }

    /// The pieces of the sunburst in a square of [`SUNBURST_SIZE`] pixels, in the order they
    /// are drawn: every folder of the volume followed by the folders inside it.
    pub fn sectors(&self) -> Vec<SunburstSector> {
        let center = Point {
            x: SUNBURST_SIZE / 2.0,
            y: SUNBURST_SIZE / 2.0,
        };
        let used: f32 = self.folders.iter().map(|folder| folder.gigabytes).sum();
        let turn_per_gigabyte = TAU * USED_SHARE / used;
        let mut sectors = Vec::new();
        let mut start_angle = -FRAC_PI_2;
        for folder in self.folders {
            let span = folder.gigabytes * turn_per_gigabyte;
            sectors.push(SunburstSector {
                sector: piece(center, FOLDER_RADII, start_angle, span),
                color: folder.color,
                opacity: 1.0,
            });
            let children_total: f32 = folder.children.iter().sum();
            let mut child_start = start_angle;
            for (position, child) in folder.children.iter().enumerate() {
                let child_span = child / children_total * span;
                sectors.push(SunburstSector {
                    sector: piece(center, CHILD_RADII, child_start, child_span),
                    color: folder.color,
                    opacity: FIRST_CHILD_OPACITY - position as f32 * CHILD_OPACITY_STEP,
                });
                child_start += child_span;
            }
            start_angle += span;
        }
        sectors
    }
}

fn piece(center: Point, (inner_radius, outer_radius): (f32, f32), start_angle: f32, span: f32) -> RingSector {
    RingSector {
        center,
        inner_radius,
        outer_radius,
        start_angle: start_angle + SEPARATION,
        end_angle: start_angle + span - SEPARATION,
    }
}
