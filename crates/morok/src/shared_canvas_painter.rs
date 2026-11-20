use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::CanvasId;
use morok_platform::{CanvasEncoder, CanvasPainter};

/// The painter the application names, as every window of the application and every picture
/// of a window drawn off screen reaches it: until the application names one, the canvases are
/// left as they are.
#[derive(Clone, Default)]
pub(crate) struct SharedCanvasPainter {
    painter: Rc<RefCell<Option<Box<dyn CanvasPainter>>>>,
}

impl SharedCanvasPainter {
    /// Names who draws the canvases from now on, in place of a painter named before.
    pub(crate) fn replace(&self, painter: Box<dyn CanvasPainter>) {
        *self.painter.borrow_mut() = Some(painter);
    }
}

impl CanvasPainter for SharedCanvasPainter {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        if let Some(painter) = self.painter.borrow_mut().as_mut() {
            painter.draw(canvas, encoder);
        }
    }
}

impl Debug for SharedCanvasPainter {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SharedCanvasPainter")
            .field("is_named", &self.painter.borrow().is_some())
            .finish()
    }
}
