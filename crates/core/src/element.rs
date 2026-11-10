use morok_style::{Class, States};
use taffy::{Cache, DetailedLayoutInfo, Layout};

use crate::accessibility::semantics::Semantics;
use crate::dirty::Dirty;
use crate::editing::editable_text::EditableText;
use crate::editing::line_mode::LineMode;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::input::focusability::Focusability;
use crate::media::media_content::MediaContent;
use crate::motion::element_motion::ElementMotion;
use crate::paint::painted_element::PaintedElement;
use crate::role::Role;
use crate::scroll::scroll_state::ScrollState;
use crate::style::computed_style::ComputedStyle;
use crate::style::inline_style::InlineStyle;
use crate::style::positioning::Positioning;
use crate::style::selector_target::SelectorTarget;
use crate::style::style_lists::StyleLists;
use crate::text::text_layout::TextLayout;

/// One node of an element tree with everything the frame stages keep about it.
#[derive(Debug)]
pub(crate) struct Element {
    pub(crate) role: Role,
    pub(crate) parent: Option<ElementId>,
    pub(crate) children: Vec<ElementId>,
    /// Whether the element is the root or one of its descendants.
    pub(crate) is_attached: bool,
    pub(crate) classes: Vec<Class>,
    pub(crate) states: States,
    pub(crate) focusability: Focusability,
    /// What the element is to an assistive technology; none for an element without a node in
    /// the accessibility tree.
    pub(crate) semantics: Option<Box<Semantics>>,
    pub(crate) inline_style: InlineStyle,
    pub(crate) text: String,
    /// The picture, icon or canvas of a leaf of those roles.
    pub(crate) media: MediaContent,
    pub(crate) dirty: Dirty,
    /// The style the element shows: the cascaded one with the values of its motion.
    pub(crate) style: ComputedStyle,
    /// The lists of the style the element shows.
    pub(crate) lists: StyleLists,
    /// The transitions and animations of an element that has run one; none for the others.
    pub(crate) motion: Option<Box<ElementMotion>>,
    /// Whether the last style stage left the element rendered: attached, and neither it nor an
    /// ancestor with `display: none`. Only then has it a style a transition can start from.
    pub(crate) is_rendered: bool,
    pub(crate) layout_cache: Cache,
    /// The box layout gave the element, relative to the element that positions it.
    pub(crate) layout: Layout,
    /// The containing block of an out-of-flow element: the ancestor its box is relative to.
    pub(crate) containing_block: Option<ElementId>,
    pub(crate) grid_info: DetailedLayoutInfo<String>,
    /// The border box on the surface.
    pub(crate) border_box: Rect,
    pub(crate) painted: PaintedElement,
    /// The lines of the text; empty unless the element is a text.
    pub(crate) text_layout: TextLayout,
    /// The text the user edits in an element of an editable role; none for the others.
    pub(crate) editable: Option<Box<EditableText>>,
    /// How far its content is scrolled, for an element of the `Scroll` role; none for the others.
    pub(crate) scroll: Option<Box<ScrollState>>,
    /// How far the frame moves the element from where layout placed it: the shift of a sticky
    /// element, or the travel of a scroll thumb along its bar.
    pub(crate) shift: Point,
    /// Whether the element is the thumb of a scroll bar, which the tree moves along its bar.
    pub(crate) is_scroll_thumb: bool,
}

impl Element {
    pub(crate) fn new(role: Role) -> Self {
        Self {
            role,
            parent: None,
            children: Vec::new(),
            is_attached: false,
            classes: Vec::new(),
            states: States::NONE,
            focusability: Focusability::NotFocusable,
            semantics: None,
            inline_style: InlineStyle::default(),
            text: String::new(),
            media: MediaContent::None,
            dirty: Dirty::STYLE.union(Dirty::PAINT).union(Dirty::TEXT),
            style: ComputedStyle::INITIAL,
            lists: StyleLists::default(),
            motion: None,
            is_rendered: false,
            layout_cache: Cache::new(),
            layout: Layout::new(),
            containing_block: None,
            grid_info: DetailedLayoutInfo::None,
            border_box: Rect::ZERO,
            painted: PaintedElement::default(),
            text_layout: TextLayout::default(),
            editable: LineMode::of(role).map(|line_mode| Box::new(EditableText::new(line_mode))),
            scroll: (role == Role::Scroll).then(Box::default),
            shift: Point::default(),
            is_scroll_thumb: false,
        }
    }

    /// Whether the frame may move the element from where layout placed it; such an element
    /// paints as a group, so that it moves as one.
    pub(crate) fn is_shifted(&self) -> bool {
        self.is_scroll_thumb || self.style.layout.positioning == Positioning::Sticky
    }

    pub(crate) fn selector_target(&self) -> SelectorTarget<'_> {
        SelectorTarget {
            classes: &self.classes,
            states: self.states,
        }
    }
}
