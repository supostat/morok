//! The catalog of the morok UI kit: an application that shows every family of the components
//! of the kit on a page of its own, in both color schemes and under two sets of token values,
//! and two screens of real applications built from them — a two-pane file manager and a disk
//! map that draws a sunburst on a canvas. It is built on the published crates of the kit alone,
//! with stylesheets and tokens of its own compiled by its build script.

#![forbid(unsafe_code)]

mod catalog;
mod catalog_window;
mod chart_view;
mod disk_map;
mod disk_map_chart;
mod disk_map_painter;
pub mod ember;
mod file_column;
mod file_entry;
mod file_manager;
mod file_pane;
mod file_place;
mod folder_usage;
mod gallery;
mod gallery_picture;
mod page;
mod page_layout;
mod pages;
mod pane_side;
mod picked_folder;
pub mod styles;
mod stylesheets;
mod sunburst;
mod sunburst_sector;
mod token_set;
pub mod tokens;
mod treemap;
mod treemap_tile;
mod volume;

pub use catalog::{Catalog, WINDOW_SIZE, WINDOW_TITLE, styles};
pub use catalog_window::NEW_WINDOW;
pub use chart_view::{CHART_SIZE, ChartView};
pub use disk_map::DiskMap;
pub use disk_map_painter::DiskMapPainter;
pub use file_manager::FileManager;
pub use folder_usage::FolderUsage;
pub use gallery::Gallery;
pub use gallery_picture::GalleryPicture;
pub use page::Page;
pub use pages::collections_page::LIST_ROWS;
pub use pages::table_page::TABLE_ROWS;
pub use picked_folder::PickedFolder;
pub use stylesheets::{EMBER_TOKENS_STYLESHEET, STYLESHEETS, TOKENS_STYLESHEET};
pub use sunburst::Sunburst;
pub use sunburst_sector::SunburstSector;
pub use token_set::TokenSet;
pub use treemap::Treemap;
pub use treemap_tile::TreemapTile;
pub use volume::Volume;
