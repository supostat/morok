use morok_core::Rect;
use serde::{Deserialize, Serialize};

/// The border box of one element of a case, in CSS pixels from the top left of the surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElementBox {
    /// Names the element in the tree of its case: its position in document order and its classes.
    pub node: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ElementBox {
    pub fn new(node: impl Into<String>, rect: Rect) -> Self {
        Self {
            node: node.into(),
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}
