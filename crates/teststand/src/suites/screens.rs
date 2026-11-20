use morok_catalog::{DISK_MAP_CANVAS, DiskMap, FileManager, STYLESHEETS, TOKENS_STYLESHEET};
use morok_components::StylesheetSource;
use morok_core::Size;
use morok_css::SourceFile;

use crate::canvas_shape::CanvasShape;
use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;

pub(crate) const NAME: &str = "screens";

/// The two screens of the catalog as the catalog builds them: a two-pane file manager and a
/// disk map whose sunburst is drawn on a canvas, styled by the stylesheets and the tokens of
/// the catalog over those of the kit.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        tokens: source_file(TOKENS_STYLESHEET),
        component_stylesheets: morok_components::STYLESHEETS.into_iter().map(source_file).collect(),
        stylesheets: STYLESHEETS.into_iter().map(source_file).collect(),
        cases: vec![
            case("file-manager", 900.0, 420.0, file_manager),
            case("disk-map", 900.0, 440.0, disk_map),
        ],
    }
}

fn source_file(source: StylesheetSource) -> SourceFile {
    SourceFile::new(source.path, source.text)
}

fn case(name: &'static str, width: f32, height: f32, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size { width, height },
        build,
        moment: Moment::AtRest,
    }
}

/// The file manager: the projects beside the archive, a row selected in each.
fn file_manager(scene: &mut Scene) {
    scene.mount(scene.root(), FileManager::mount);
}

/// The disk map: the volumes, the sunburst of the folders of the first one and the folder the
/// analysis picked out.
fn disk_map(scene: &mut Scene) {
    scene.mount(scene.root(), DiskMap::mount);
    let shapes = DiskMap::sunburst()
        .sectors()
        .into_iter()
        .map(|piece| CanvasShape {
            sector: piece.sector,
            color: piece.color,
            opacity: piece.opacity,
        })
        .collect();
    scene.draw_canvas(DISK_MAP_CANVAS, shapes);
}
