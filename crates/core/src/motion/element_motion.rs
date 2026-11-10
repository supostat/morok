use crate::motion::animation::Animation;
use crate::motion::timeline::Timeline;
use crate::motion::transition::Transition;
use crate::style::computed_style::ComputedStyle;
use crate::style::style_lists::StyleLists;

/// The transitions and keyframe animations of one element, with the style they run on: the one
/// the cascade gives the element before any motion.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ElementMotion {
    pub(crate) base: ComputedStyle,
    pub(crate) base_lists: StyleLists,
    pub(crate) transitions: Vec<Transition>,
    /// In the order of `animation-name`: a later animation of a property overrides an earlier one.
    pub(crate) animations: Vec<Animation>,
}

impl ElementMotion {
    pub(crate) fn new(base: &ComputedStyle) -> Self {
        Self {
            base: *base,
            base_lists: StyleLists::default(),
            transitions: Vec::new(),
            animations: Vec::new(),
        }
    }

    /// Whether a frame later than now shows something else.
    pub(crate) fn is_running(&self, timeline: &Timeline) -> bool {
        !self.transitions.is_empty() || self.animations.iter().any(|animation| animation.is_running(timeline))
    }

    pub(crate) fn clear(&mut self) {
        self.transitions.clear();
        self.animations.clear();
    }
}
