//! The catalog of the morok UI kit: an application that shows every family of the components
//! of the kit on a page of its own, in both color schemes and under two sets of token values,
//! and two screens of real applications built from them — a two-pane file manager and a disk
//! map that draws a sunburst on a canvas. It is built on the published crates of the kit alone,
//! with stylesheets and tokens of its own compiled by its build script.

#![forbid(unsafe_code)]

mod catalog;
mod catalog_window;
mod disk_map;
pub mod ember;
mod file_column;
mod file_entry;
mod file_manager;
mod file_pane;
mod file_place;
mod folder_usage;
mod gallery;
mod page;
mod page_layout;
mod pages;
mod pane_side;
pub mod styles;
mod stylesheets;
mod sunburst;
mod sunburst_painter;
mod sunburst_sector;
mod token_set;
pub mod tokens;

pub use catalog::{Catalog, WINDOW_SIZE, WINDOW_TITLE, styles};
pub use catalog_window::NEW_WINDOW;
pub use disk_map::{DISK_MAP_CANVAS, DiskMap};
pub use file_manager::FileManager;
pub use folder_usage::FolderUsage;
pub use gallery::Gallery;
pub use page::Page;
pub use pages::collections_page::LIST_ROWS;
pub use pages::table_page::TABLE_ROWS;
pub use stylesheets::{EMBER_TOKENS_STYLESHEET, STYLESHEETS, TOKENS_STYLESHEET};
pub use sunburst::{SUNBURST_SIZE, Sunburst};
pub use sunburst_painter::SunburstPainter;
pub use sunburst_sector::SunburstSector;
pub use token_set::TokenSet;
