use accesskit::{Action, ActionData, ActionRequest, ScrollUnit};
use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};

use super::ElementTree;
use crate::dirty::Dirty;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::geometry::size::Size;
use crate::input::key::Key;
use crate::input::key_event::KeyEvent;
use crate::input::key_phase::KeyPhase;
use crate::input::modifiers::Modifiers;
use crate::motion::spring::Spring;
use crate::scroll::scroll_bars::ScrollBars;
use crate::scroll::scroll_input::ScrollInput;
use crate::scroll::scroll_phase::ScrollPhase;
use crate::scroll::scroll_state::ScrollState;
use crate::scroll::sticky_shift::StickyPlace;
use crate::style::positioning::Positioning;

/// How far an arrow key, or an assistive technology asking for an item, scrolls.
const LINE_LENGTH: f32 = 40.0;
/// The share of the viewport a page key, or an assistive technology asking for a page, scrolls.
const PAGE_SHARE: f32 = 0.875;
/// The shortest a thumb gets however long the content is.
const SHORTEST_THUMB: f32 = 24.0;
/// The spring that takes content pulled past a limit back to it.
const RETURN_SPRING: Spring = Spring {
    stiffness: 180.0,
    precision: 0.5,
};

/// The content of an element of the `Scroll` role moves by its offset as a picture: a scroll
/// runs no layout. The offset stays within limits the last layout found — the extent of the
/// content past the padding box on the axes whose overflow scrolls — except while a trackpad
/// gesture pulls it past them, against the resistance of the edge, after which a spring takes it
/// back.
impl ElementTree {
    /// How far the content of a `Scroll` element is scrolled now: what the next frame shows.
    ///
    /// # Panics
    ///
    /// When the element is not of the `Scroll` role.
    pub fn scroll_offset(&self, element: ElementId) -> Point {
        self.scroll_state(element).offset
    }

    /// The largest offset of each axis of a `Scroll` element as the last layout found it: zero
    /// on an axis that does not scroll or whose content fits.
    ///
    /// # Panics
    ///
    /// When the element is not of the `Scroll` role.
    pub fn scroll_limit(&self, element: ElementId) -> Point {
        self.scroll_state(element).limit
    }

    /// Scrolls the content of a `Scroll` element to `offset`, kept within the limits the next
    /// layout finds; what moved the content before stops.
    ///
    /// # Panics
    ///
    /// When the element is not of the `Scroll` role.
    pub fn scroll_to(&mut self, element: ElementId, offset: Point) {
        self.stop_returns(element);
        let state = self.scroll_state_mut(element);
        state.is_in_gesture = false;
        state.requested = Some(offset);
        self.request_frame();
    }

    /// Names the thumbs of the overlay scroll bars of a `Scroll` element, in place of those it
    /// had: from the next frame on the tree sizes them by the share of the content the viewport
    /// shows, moves them along their bars as the content scrolls and gives them their `showing`
    /// class while it moves.
    ///
    /// # Panics
    ///
    /// When the element is not of the `Scroll` role.
    pub fn set_scroll_bars(&mut self, element: ElementId, bars: ScrollBars) {
        if let Some(previous) = self.scroll_state(element).bars {
            for thumb in [previous.vertical, previous.horizontal].into_iter().flatten() {
                self.elements[thumb].is_scroll_thumb = false;
                self.elements[thumb].shift = Point::default();
            }
        }
        for thumb in [bars.vertical, bars.horizontal].into_iter().flatten() {
            self.elements[thumb].is_scroll_thumb = true;
            self.elements[thumb].dirty.insert(Dirty::PAINT);
        }
        self.scroll_state_mut(element).bars = Some(bars);
        self.paint_order_is_stale = true;
        self.has_unpainted_changes = true;
        self.elements.invalidate_layout(element);
        self.request_frame();
    }

    /// The overlay scroll bars of a `Scroll` element, if it was given any.
    ///
    /// # Panics
    ///
    /// When the element is not of the `Scroll` role.
    pub fn scroll_bars(&self, element: ElementId) -> Option<ScrollBars> {
        self.scroll_state(element).bars
    }

    /// How far the last frame moved a scroll thumb along its bar; none for an element that is
    /// no thumb.
    pub fn scroll_thumb_travel(&self, element: ElementId) -> Option<Point> {
        let element = &self.elements[element];
        element.is_scroll_thumb.then_some(element.shift)
    }

    /// Takes a scroll of the wheel or the trackpad over the surface. A scroll goes to the
    /// innermost `Scroll` element under the pointer that has room to move that way; a gesture
    /// stays with the element it began over until it and its momentum end.
    pub fn scroll_input(&mut self, input: &ScrollInput) {
        let latched = self
            .input
            .scroll_latch
            .filter(|latched| self.elements.contains(*latched));
        let target = match input.phase {
            ScrollPhase::Discrete => self.scroller_at(input.position, input.delta),
            ScrollPhase::GestureBegan => {
                let target = self.scroller_at(input.position, input.delta);
                self.input.scroll_latch = target;
                target
            }
            ScrollPhase::GestureChanged
            | ScrollPhase::GestureEnded
            | ScrollPhase::Momentum
            | ScrollPhase::MomentumEnded => latched,
        };
        if input.phase == ScrollPhase::MomentumEnded {
            self.input.scroll_latch = None;
        }
        let Some(scroller) = target else {
            return;
        };
        if input.phase.is_of_gesture() && self.scroll_state(scroller).has_return() {
            self.stop_returns(scroller);
            self.scroll_state_mut(scroller).resume_gesture();
        }
        let layout = self.elements[scroller].style.layout;
        let state = self.scroll_state_mut(scroller);
        state.scroll(
            input.delta,
            input.phase,
            layout.scrolls_horizontally(),
            layout.scrolls_vertically(),
        );
        state.is_in_gesture = input.phase.is_of_gesture() && !input.phase.ends_motion();
        if input.phase.ends_motion() {
            self.start_returns(scroller);
        }
        self.request_frame();
    }

    /// Scrolls the `Scroll` element around focus by a key nothing else used: an arrow by a
    /// line, Page Up and Page Down, Space and Shift-Space by most of the viewport, Home and End
    /// to the ends. Returns whether the content moved.
    pub(crate) fn scroll_by_key(&mut self, event: &KeyEvent) -> bool {
        if event.phase != KeyPhase::Pressed {
            return false;
        }
        let Some(focused) = self.input.focused else {
            return false;
        };
        let Some(scroller) = self.scroller_around(focused) else {
            return false;
        };
        let state = self.scroll_state(scroller);
        let page = state.viewport.height * PAGE_SHARE;
        let now = state.offset;
        let by = |x: f32, y: f32| Point {
            x: now.x + x,
            y: now.y + y,
        };
        let target = match (event.key, event.modifiers) {
            (Key::ArrowDown, Modifiers::NONE) => by(0.0, LINE_LENGTH),
            (Key::ArrowUp, Modifiers::NONE) => by(0.0, -LINE_LENGTH),
            (Key::ArrowRight, Modifiers::NONE) => by(LINE_LENGTH, 0.0),
            (Key::ArrowLeft, Modifiers::NONE) => by(-LINE_LENGTH, 0.0),
            (Key::PageDown | Key::Space, Modifiers::NONE) => by(0.0, page),
            (Key::PageUp, Modifiers::NONE) | (Key::Space, Modifiers::SHIFT) => by(0.0, -page),
            (Key::Home, Modifiers::NONE) => Point { x: now.x, y: 0.0 },
            (Key::End, Modifiers::NONE) => Point {
                x: now.x,
                y: state.limit.y,
            },
            _ => return false,
        };
        self.move_scroll(scroller, target)
    }

    /// Does what an assistive technology asks of the scroll of an element: a `Scroll` element
    /// scrolls by a page or an item or to an offset, and any element is scrolled into view.
    /// Returns whether the request was one of these.
    pub(crate) fn scroll_by_request(&mut self, target: ElementId, request: &ActionRequest) -> bool {
        if request.action == Action::ScrollIntoView {
            self.reveal(target);
            self.request_frame();
            return true;
        }
        if self.elements[target].scroll.is_none() {
            return false;
        }
        let state = self.scroll_state(target);
        let now = state.offset;
        let unit = |length: f32| match request.data {
            Some(ActionData::ScrollUnit(ScrollUnit::Item)) => LINE_LENGTH,
            _ => length * PAGE_SHARE,
        };
        let goal = match (request.action, &request.data) {
            (Action::ScrollDown, _) => Point {
                x: now.x,
                y: now.y + unit(state.viewport.height),
            },
            (Action::ScrollUp, _) => Point {
                x: now.x,
                y: now.y - unit(state.viewport.height),
            },
            (Action::ScrollRight, _) => Point {
                x: now.x + unit(state.viewport.width),
                y: now.y,
            },
            (Action::ScrollLeft, _) => Point {
                x: now.x - unit(state.viewport.width),
                y: now.y,
            },
            (Action::SetScrollOffset, Some(ActionData::SetScrollOffset(offset))) => Point {
                x: offset.x as f32,
                y: offset.y as f32,
            },
            _ => return false,
        };
        self.move_scroll(target, goal);
        true
    }

    /// Settles the scroll of every `Scroll` element after layout: the limits the layout found,
    /// the offset asked for, the springs, focus the keyboard moved, then sticky elements and
    /// scroll thumbs placed for the offset the frame shows. `laid_out` tells that layout moved
    /// boxes. Returns whether a thumb changed its length, so that it is styled and laid out
    /// again.
    pub(crate) fn update_scrolling(&mut self, laid_out: bool) -> bool {
        for position in 0..self.scrollers.len() {
            let scroller = self.scrollers[position];
            if self.elements[scroller].is_attached {
                self.settle_scroll(scroller);
            }
        }
        if let Some(focused) = self.input.focus_to_reveal.take()
            && self.elements.contains(focused)
            && self.elements[focused].is_attached
        {
            self.reveal(focused);
        }
        let mut lengthens_thumbs = false;
        for position in 0..self.scrollers.len() {
            let scroller = self.scrollers[position];
            if !self.elements[scroller].is_attached {
                continue;
            }
            let state = self.scroll_state_mut(scroller);
            let moved = state.offset != state.shown;
            state.moved_in_last_frame = moved;
            if moved {
                state.shown = state.offset;
                self.elements[scroller].dirty.insert(Dirty::PAINT);
                self.has_unpainted_changes = true;
                self.request_frame();
            }
            if moved || laid_out {
                self.place_sticky_descendants(scroller);
            }
            lengthens_thumbs |= self.place_thumbs(scroller);
        }
        lengthens_thumbs
    }

    /// Gives the thumbs of a `Scroll` element their `showing` class while its content moves
    /// and takes it away in the frame after the content comes to rest.
    pub(crate) fn show_moving_scroll_bars(&mut self) {
        for position in 0..self.scrollers.len() {
            let scroller = self.scrollers[position];
            if !self.elements[scroller].is_attached {
                continue;
            }
            let state = self.scroll_state(scroller);
            let Some(bars) = state.bars else {
                continue;
            };
            let shows = state.offset != state.shown || state.moved_in_last_frame || state.has_return();
            for thumb in [bars.vertical, bars.horizontal].into_iter().flatten() {
                let has_class = self.elements[thumb].classes.contains(&bars.showing);
                if shows && !has_class {
                    self.add_class(thumb, bars.showing);
                } else if !shows && has_class {
                    self.remove_class(thumb, bars.showing);
                }
            }
        }
    }

    fn scroll_state(&self, element: ElementId) -> &ScrollState {
        self.elements[element]
            .scroll
            .as_deref()
            .expect("the element is of the `Scroll` role")
    }

    fn scroll_state_mut(&mut self, element: ElementId) -> &mut ScrollState {
        self.elements[element]
            .scroll
            .as_deref_mut()
            .expect("the element is of the `Scroll` role")
    }

    /// Moves the content of the `Scroll` element to `goal` within its limits at once. Returns
    /// whether it moved.
    fn move_scroll(&mut self, scroller: ElementId, goal: Point) -> bool {
        self.stop_returns(scroller);
        let state = self.scroll_state_mut(scroller);
        let before = state.offset;
        state.is_in_gesture = false;
        state.move_to(goal);
        let moved = state.offset != before;
        if moved {
            self.request_frame();
        }
        moved
    }

    /// The innermost `Scroll` element at or around the element, if any.
    fn scroller_around(&self, element: ElementId) -> Option<ElementId> {
        let mut current = Some(element);
        while let Some(id) = current {
            if self.elements[id].scroll.is_some() {
                return Some(id);
            }
            current = self.elements[id].parent;
        }
        None
    }

    /// The innermost `Scroll` element under a point of the surface with room to move by
    /// `delta`, or with room to move at all when the delta is none.
    fn scroller_at(&mut self, position: Point, delta: Point) -> Option<ElementId> {
        let mut current = self.element_at(position);
        while let Some(id) = current {
            let element = &self.elements[id];
            if let Some(state) = element.scroll.as_deref() {
                let layout = &element.style.layout;
                let room_x = layout.scrolls_horizontally() && state.limit.x > 0.0;
                let room_y = layout.scrolls_vertically() && state.limit.y > 0.0;
                let has_no_delta = delta == Point::default();
                if (room_x && (delta.x != 0.0 || has_no_delta)) || (room_y && (delta.y != 0.0 || has_no_delta)) {
                    return Some(id);
                }
            }
            current = element.parent;
        }
        None
    }

    /// The limits from the last layout, the offset asked for, the springs taking the content
    /// back, and, while nothing pulls it, the offset kept within the limits.
    fn settle_scroll(&mut self, scroller: ElementId) {
        let element = &self.elements[scroller];
        let layout = &element.layout;
        let style = &element.style.layout;
        let viewport = Size {
            width: layout.size.width - layout.border.left - layout.border.right,
            height: layout.size.height - layout.border.top - layout.border.bottom,
        };
        let limit = Point {
            x: if style.scrolls_horizontally() {
                layout.scroll_width()
            } else {
                0.0
            },
            y: if style.scrolls_vertically() {
                layout.scroll_height()
            } else {
                0.0
            },
        };
        let (return_x, return_y) = {
            let state = self.scroll_state_mut(scroller);
            state.viewport = viewport;
            state.limit = limit;
            if let Some(requested) = state.requested.take() {
                state.move_to(requested);
            }
            (state.return_x, state.return_y)
        };
        let returned_x = return_x.map(|spring| self.spring_sample(spring));
        let returned_y = return_y.map(|spring| self.spring_sample(spring));
        let state = self.scroll_state_mut(scroller);
        if let Some(sample) = returned_x {
            state.offset.x = sample.value;
        }
        if let Some(sample) = returned_y {
            state.offset.y = sample.value;
        }
        let settled_x = returned_x.filter(|sample| sample.is_settled).and(return_x);
        let settled_y = returned_y.filter(|sample| sample.is_settled).and(return_y);
        if settled_x.is_some() {
            state.return_x = None;
        }
        if settled_y.is_some() {
            state.return_y = None;
        }
        if !state.is_in_gesture && !state.has_return() {
            let kept = state.clamped(state.offset);
            state.move_to(kept);
        }
        for spring in [settled_x, settled_y].into_iter().flatten() {
            self.stop_spring(spring);
        }
    }

    /// Starts the springs that take content left past a limit back to it.
    fn start_returns(&mut self, scroller: ElementId) {
        let state = self.scroll_state(scroller);
        let offset = state.offset;
        let goal = state.clamped(offset);
        let is_past = |offset: f32, limit: f32| offset < 0.0 || offset > limit;
        let (is_past_x, is_past_y) = (is_past(offset.x, state.limit.x), is_past(offset.y, state.limit.y));
        let return_x = is_past_x.then(|| self.start_spring(RETURN_SPRING, offset.x, goal.x, 0.0));
        let return_y = is_past_y.then(|| self.start_spring(RETURN_SPRING, offset.y, goal.y, 0.0));
        let state = self.scroll_state_mut(scroller);
        state.return_x = return_x;
        state.return_y = return_y;
    }

    fn stop_returns(&mut self, scroller: ElementId) {
        let state = self.scroll_state_mut(scroller);
        let springs = [state.return_x.take(), state.return_y.take()];
        for spring in springs.into_iter().flatten() {
            self.stop_spring(spring);
        }
    }

    /// Scrolls every `Scroll` element around the element as little as brings the element into
    /// its viewport, the innermost first; an element longer than a viewport shows its start.
    fn reveal(&mut self, target: ElementId) {
        let mut shown = self.elements[target].border_box;
        let mut ancestor = self.elements[target].parent;
        while let Some(current) = ancestor {
            if self.elements[current].scroll.is_some() {
                let port = padding_box(&self.elements[current]);
                let state = self.scroll_state_mut(current);
                let goal = Point {
                    x: revealing_offset(shown.x, shown.right(), port.x, port.width, state.offset.x),
                    y: revealing_offset(shown.y, shown.bottom(), port.y, port.height, state.offset.y),
                };
                state.is_in_gesture = false;
                state.move_to(goal);
                let offset = state.offset;
                shown = Rect::new(shown.x - offset.x, shown.y - offset.y, shown.width, shown.height);
                self.stop_returns(current);
            }
            ancestor = self.elements[current].parent;
        }
    }

    /// Shifts the sticky elements whose nearest `Scroll` ancestor is `scroller` for the offset
    /// it shows; an element that is sticky no more goes back to where layout placed it.
    fn place_sticky_descendants(&mut self, scroller: ElementId) {
        let port = padding_box(&self.elements[scroller]);
        let offset = self.scroll_state(scroller).shown;
        let scrollport = Rect::new(port.x + offset.x, port.y + offset.y, port.width, port.height);
        self.pending.clear();
        self.pending.extend(&self.elements[scroller].children);
        while let Some(id) = self.pending.pop() {
            let element = &self.elements[id];
            let shift = if element.style.layout.positioning == Positioning::Sticky {
                let parent = &self.elements[element.parent.expect("a descendant has a parent")];
                StickyPlace {
                    border_box: element.border_box,
                    margin: Sides {
                        top: element.layout.margin.top,
                        right: element.layout.margin.right,
                        bottom: element.layout.margin.bottom,
                        left: element.layout.margin.left,
                    },
                    containing_block: content_box(parent),
                    scrollport,
                }
                .shift(&element.style.layout.inset)
            } else {
                Point::default()
            };
            if element.scroll.is_none() {
                self.pending.extend(&element.children);
            }
            if !element.is_scroll_thumb && element.shift != shift {
                self.elements[id].shift = shift;
                self.elements[id].dirty.insert(Dirty::PAINT);
                self.has_unpainted_changes = true;
            }
        }
    }

    /// Sizes the thumbs of a `Scroll` element by the share of its content the viewport shows
    /// and moves them along their bars by the offset it shows. Returns whether a thumb changed
    /// its length.
    fn place_thumbs(&mut self, scroller: ElementId) -> bool {
        let state = self.scroll_state(scroller);
        let Some(bars) = state.bars else {
            return false;
        };
        let (viewport, limit, offset) = (state.viewport, state.limit, state.shown);
        let mut lengthens = false;
        if let Some(thumb) = bars.vertical {
            lengthens |= self.place_thumb(thumb, ThumbAxis::Vertical, viewport.height, limit.y, offset.y);
        }
        if let Some(thumb) = bars.horizontal {
            lengthens |= self.place_thumb(thumb, ThumbAxis::Horizontal, viewport.width, limit.x, offset.x);
        }
        lengthens
    }

    fn place_thumb(&mut self, thumb: ElementId, axis: ThumbAxis, viewport: f32, limit: f32, offset: f32) -> bool {
        let bar = self.elements[thumb].parent.expect("a thumb lies on its bar");
        let track = content_box(&self.elements[bar]);
        let track_length = match axis {
            ThumbAxis::Vertical => track.height,
            ThumbAxis::Horizontal => track.width,
        };
        let content_length = viewport + limit;
        let (length, travel) = if limit > 0.0 && content_length > 0.0 {
            let length = (track_length * viewport / content_length)
                .max(SHORTEST_THUMB)
                .min(track_length);
            (length, (track_length - length) * (offset / limit).clamp(0.0, 1.0))
        } else {
            (0.0, 0.0)
        };
        let shift = match axis {
            ThumbAxis::Vertical => Point { x: 0.0, y: travel },
            ThumbAxis::Horizontal => Point { x: travel, y: 0.0 },
        };
        if self.elements[thumb].shift != shift {
            self.elements[thumb].shift = shift;
            self.elements[thumb].dirty.insert(Dirty::PAINT);
            self.has_unpainted_changes = true;
        }
        let size = LengthPercentageAuto::LengthPercentage(LengthPercentage {
            pixels: Pixels(length),
            fraction: None,
            tokens: std::borrow::Cow::Borrowed(&[]),
        });
        let declaration = match axis {
            ThumbAxis::Vertical => Declaration::Height(size),
            ThumbAxis::Horizontal => Declaration::Width(size),
        };
        if self.elements[thumb].inline_style.declarations().contains(&declaration) {
            return false;
        }
        self.set_inline(thumb, declaration);
        true
    }
}

/// The axis a thumb moves along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThumbAxis {
    Vertical,
    Horizontal,
}

/// The offset of one axis that brings the extent from `start` to `end` into a viewport of
/// `length` from `port_start` with as little a scroll as it takes.
fn revealing_offset(start: f32, end: f32, port_start: f32, length: f32, offset: f32) -> f32 {
    let visible_start = port_start + offset;
    let visible_end = visible_start + length;
    if start < visible_start {
        offset - (visible_start - start)
    } else if end > visible_end {
        offset + (end - visible_end).min(start - visible_start)
    } else {
        offset
    }
}

/// The padding box of an element as layout placed it.
fn padding_box(element: &crate::element::Element) -> Rect {
    let border = &element.layout.border;
    let border_box = element.border_box;
    Rect::new(
        border_box.x + border.left,
        border_box.y + border.top,
        border_box.width - border.left - border.right,
        border_box.height - border.top - border.bottom,
    )
}

/// The content box of an element as layout placed it.
fn content_box(element: &crate::element::Element) -> Rect {
    let padding = &element.layout.padding;
    let port = padding_box(element);
    Rect::new(
        port.x + padding.left,
        port.y + padding.top,
        port.width - padding.left - padding.right,
        port.height - padding.top - padding.bottom,
    )
}
