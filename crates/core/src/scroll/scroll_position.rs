use crate::geometry::point::Point;

/// How far the content of a `Scroll` element is scrolled and how far it can be.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScrollPosition {
    pub(crate) offset: Point,
    pub(crate) limit: Point,
}

/// What the node of an element tells of scrolling.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NodeScroll {
    /// Whether a `Scroll` element is around the element, which can scroll it into view.
    pub(crate) is_inside_scroll: bool,
    /// For a `Scroll` element: how far its content is scrolled and can be.
    pub(crate) position: Option<ScrollPosition>,
}
