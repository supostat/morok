use crate::drag::drag_candidate::DragCandidate;
use crate::drag::drag_session::DragSession;
use crate::drag::system_drag::SystemDrag;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::input::focus_trap::FocusTrap;
use crate::input::input_modality::InputModality;
use crate::input::modifiers::Modifiers;
use crate::input::shortcut::Shortcut;

/// What a tree remembers of the pointer and the keyboard between two inputs.
#[derive(Debug)]
pub(crate) struct InputState {
    /// Where the pointer rests on the surface; none while it is outside.
    pub(crate) pointer: Option<Point>,
    pub(crate) pointer_modifiers: Modifiers,
    /// The innermost element the pointer is over; it and its ancestors are hovered. While a
    /// drag is under way it is the drop target the drag is over.
    pub(crate) hovered: Option<ElementId>,
    /// The element the primary button went down on. Until the button is released it and its
    /// ancestors are active and the pointer belongs to it.
    pub(crate) pressed: Option<ElementId>,
    /// Whether the boxes may have moved under a resting pointer since hover was worked out.
    pub(crate) hover_is_stale: bool,
    pub(crate) focused: Option<ElementId>,
    pub(crate) modality: InputModality,
    /// The traps set and not released, the innermost last.
    pub(crate) focus_traps: Vec<FocusTrap>,
    pub(crate) shortcuts: Vec<Shortcut>,
    /// The serial number of the last shortcut registered.
    pub(crate) last_shortcut: u64,
    /// How many times the lines the shortcuts give the menu bar changed.
    pub(crate) menu_bar_revision: u64,
    /// The element whose edited text the pointer selects in, from a press on it to the release.
    pub(crate) text_drag: Option<ElementId>,
    /// Scratch space for the elements the pointer comes over, the innermost first.
    pub(crate) entered: Vec<ElementId>,
    /// Scratch space of the hit test: the point in the coordinates of every enclosing group.
    pub(crate) group_points: Vec<Option<Point>>,
    /// The `Scroll` element a trackpad gesture began over: it takes the gesture and its
    /// momentum to their end.
    pub(crate) scroll_latch: Option<ElementId>,
    /// The element the keyboard focused since the last frame; the frame scrolls it into view.
    pub(crate) focus_to_reveal: Option<ElementId>,
    /// The press on a drag source that has not travelled far enough to be a drag yet.
    pub(crate) drag_candidate: Option<DragCandidate>,
    /// The drag under way over the surface.
    pub(crate) drag: Option<DragSession>,
    /// The drag a source handed to the system and the host has not taken yet.
    pub(crate) system_drag: Option<SystemDrag>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pointer: None,
            pointer_modifiers: Modifiers::NONE,
            hovered: None,
            pressed: None,
            hover_is_stale: false,
            focused: None,
            // Focus given before any input is shown, as it is to a user who has not touched
            // the pointer.
            modality: InputModality::Keyboard,
            focus_traps: Vec::new(),
            shortcuts: Vec::new(),
            last_shortcut: 0,
            menu_bar_revision: 0,
            text_drag: None,
            entered: Vec::new(),
            group_points: Vec::new(),
            scroll_latch: None,
            focus_to_reveal: None,
            drag_candidate: None,
            drag: None,
            system_drag: None,
        }
    }
}
