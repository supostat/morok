use crate::geometry::point::Point;
use crate::menu::native_menu_item::NativeMenuItem;

/// A menu the system shows for a surface, in its own look: what a surface asks its host for
/// when a component wants the menu of the system instead of one the kit draws.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeMenu {
    /// Where on the surface the menu opens, in CSS pixels: its top left corner.
    pub position: Point,
    pub items: Vec<NativeMenuItem>,
}
