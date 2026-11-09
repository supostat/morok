use std::fmt::{Debug, Formatter, Result};

use morok_core::IconId;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Node, Options, Paint, Tree};

use crate::media::icon_raster::IconRaster;

/// The color `currentColor` resolves to while the icon is parsed: a single-color icon is
/// drawn in it and tinted by the shader, so the icon needs one rasterization for every color.
const CURRENT_COLOR: &str = "#ffffff";
const SVG_TAG: &str = "<svg";

/// The icons of a renderer, parsed once and rasterized at every size they are drawn at.
#[derive(Default)]
pub(crate) struct IconLibrary {
    icons: Vec<Icon>,
}

struct Icon {
    tree: Tree,
    /// Whether every paint of the icon is `currentColor`.
    is_single_color: bool,
}

impl IconLibrary {
    /// Parses the SVG; `currentColor` in it stands for the color of the element that shows it.
    pub(crate) fn add(&mut self, svg: &str) -> std::result::Result<IconId, resvg::usvg::Error> {
        let with_color = svg.replacen(SVG_TAG, &format!("{SVG_TAG} color=\"{CURRENT_COLOR}\""), 1);
        let tree = Tree::from_str(&with_color, &Options::default())?;
        let is_single_color = paints_only_white(tree.root());
        self.icons.push(Icon { tree, is_single_color });
        Ok(IconId(self.icons.len() as u32 - 1))
    }

    /// Draws the icon scaled to `width` by `height` device pixels.
    pub(crate) fn rasterize(&self, icon: IconId, width: u32, height: u32) -> Option<IconRaster> {
        let held = &self.icons[icon.0 as usize];
        let size = held.tree.size();
        let mut pixmap = Pixmap::new(width, height)?;
        let transform = Transform::from_scale(width as f32 / size.width(), height as f32 / size.height());
        resvg::render(&held.tree, transform, &mut pixmap.as_mut());
        let pixels = if held.is_single_color {
            pixmap.pixels().iter().map(|pixel| pixel.alpha()).collect()
        } else {
            pixmap
                .pixels()
                .iter()
                .flat_map(|pixel| [pixel.blue(), pixel.green(), pixel.red(), pixel.alpha()])
                .collect()
        };
        Some(IconRaster {
            width,
            height,
            is_single_color: held.is_single_color,
            pixels,
        })
    }
}

/// Whether every fill and stroke of the subtree is the color `currentColor` resolved to.
fn paints_only_white(group: &resvg::usvg::Group) -> bool {
    group.children().iter().all(|node| match node {
        Node::Group(group) => paints_only_white(group),
        Node::Path(path) => {
            let is_white = |paint: &Paint| match paint {
                Paint::Color(color) => color.red == 255 && color.green == 255 && color.blue == 255,
                _ => false,
            };
            path.fill().is_none_or(|fill| is_white(fill.paint()))
                && path.stroke().is_none_or(|stroke| is_white(stroke.paint()))
        }
        Node::Image(_) | Node::Text(_) => false,
    })
}

impl Debug for IconLibrary {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("IconLibrary")
            .field("icons", &self.icons.len())
            .finish()
    }
}
