use crate::text::caret_line::CaretLine;
use crate::text::caret_stop::CaretStop;

/// The lines of an edited text as a caret moves through them. The buffers are reused from one
/// shaping to the next.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaretLines {
    pub lines: Vec<CaretLine>,
    pub stops: Vec<CaretStop>,
}

impl CaretLines {
    pub fn clear(&mut self) {
        self.lines.clear();
        self.stops.clear();
    }

    pub fn stops_of(&self, line: &CaretLine) -> &[CaretStop] {
        &self.stops[line.stops.as_range()]
    }
}
