use morok_catalog::{ChartView, DiskMap, FileManager, STYLESHEETS, TOKENS_STYLESHEET, Volume};
use morok_components::StylesheetSource;
use morok_core::Size;
use morok_css::SourceFile;

use crate::canvas_outline::CanvasOutline;
use crate::canvas_shape::CanvasShape;
use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;

pub(crate) const NAME: &str = "screens";

/// The two screens of the catalog as the catalog builds them: a two-pane file manager and a
/// disk map whose chart the painter of the catalog draws on a canvas, for both volumes and in
/// both views, styled by the stylesheets and the tokens of the catalog over those of the kit.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        tokens: source_file(TOKENS_STYLESHEET),
        component_stylesheets: morok_components::STYLESHEETS.into_iter().map(source_file).collect(),
        stylesheets: STYLESHEETS.into_iter().map(source_file).collect(),
        cases: vec![
            case("file-manager", 900.0, 420.0, file_manager),
            case("disk-map", 900.0, 440.0, disk_map),
            case("disk-map-archive", 900.0, 440.0, disk_map_archive),
            case("disk-map-treemap", 900.0, 440.0, disk_map_treemap),
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

/// The disk map as it opens: the volumes, the sunburst of the folders of the first one and the
/// folder the analysis picked out.
fn disk_map(scene: &mut Scene) {
    show_disk_map(scene, Volume::Macintosh, ChartView::Sunburst);
}

/// The disk map on the second volume: its sunburst, its used space and its folder.
fn disk_map_archive(scene: &mut Scene) {
    show_disk_map(scene, Volume::Archive, ChartView::Sunburst);
}

/// The disk map of the first volume as a treemap.
fn disk_map_treemap(scene: &mut Scene) {
    show_disk_map(scene, Volume::Macintosh, ChartView::Treemap);
}

/// The disk map of `volume` in `view`: Chrome fills its canvas with the pieces of the chart,
/// the kit draws it with the painter of the catalog.
fn show_disk_map(scene: &mut Scene, volume: Volume, view: ChartView) {
    scene.mount(scene.root(), move |mount| DiskMap::showing(volume, view).mount(mount));
    let shapes = match view {
        ChartView::Sunburst => volume
            .sunburst()
            .sectors()
            .into_iter()
            .map(|piece| CanvasShape {
                outline: CanvasOutline::Ring(piece.sector),
                color: piece.color,
                opacity: piece.opacity,
            })
            .collect(),
        ChartView::Treemap => volume
            .treemap()
            .tiles()
            .into_iter()
            .map(|tile| CanvasShape {
                outline: CanvasOutline::Rectangle(tile.rect),
                color: tile.color,
                opacity: tile.opacity,
            })
            .collect(),
    };
    scene.draw_canvas(DiskMap::canvas(volume, view), shapes, DiskMap::painter());
}
