use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};
use std::ops::Range;
use std::time::Duration;

use morok_core::accesskit::ActionRequest;
use morok_core::{
    DragOutcome, ElementId, ElementTree, IconId, KeyEvent, Modifiers, NativeMenu, Payload, Point, PointerInput,
    ScrollInput, TextCommand, TimerId,
};
use morok_reactive::{Runtime, Scope, Signal};
use morok_style::{StyleSheet, Theme};

use crate::kit_icon::KitIcon;
use crate::view::action::Action;
use crate::view::child_fragments::ChildFragments;
use crate::view::event_dispatch::EventDispatch;
use crate::view::event_handlers::EventHandlers;
use crate::view::native_menu_choice::NativeMenuChoice;
use crate::view::overlay_attachment::OverlayAttachment;
use crate::view::scroll_area::ScrollArea;
use crate::view::surfaces::Surfaces;
use crate::view::timer_handler::TimerHandler;

/// One surface as the components see it: its element tree and what they registered for its
/// elements. The host of the surface gives it the input of the surface and runs the stages of
/// its tree.
pub struct Surface {
    pub(crate) tree: ElementTree,
    /// Owns everything mounted on the surface.
    pub(crate) scope: Scope,
    pub(crate) event_handlers: EventHandlers,
    pub(crate) child_fragments: ChildFragments,
    /// The IDs the host registered the icons of the kit under, by [`KitIcon`]; empty when it
    /// registered none.
    pub(crate) kit_icon_ids: Vec<IconId>,
    /// The elements that show an icon of the kit.
    kit_icon_elements: HashMap<ElementId, KitIcon>,
    /// Scratch space for the elements of a subtree that is removed.
    removed_elements: Vec<ElementId>,
    /// The `Scroll` elements whose scroll components follow, with the signals they follow it by.
    scroll_followers: Vec<(ElementId, Signal<ScrollArea>)>,
    /// What ties the surface to the surface it is opened over; none for the surface of a window.
    pub(crate) overlay: Option<OverlayAttachment>,
    timer_handlers: HashMap<TimerId, TimerHandler>,
    /// Scratch space for the timers that fall due in a frame.
    due_timers: Vec<TimerId>,
    /// The menu of the system a component asked for and the host has not shown yet.
    native_menu: Option<NativeMenu>,
    /// What hears of the line chosen in the menu of the system asked for last.
    native_menu_choice: Option<NativeMenuChoice>,
}

impl Surface {
    pub(crate) fn new(style_sheet: StyleSheet, theme: Theme, scope: Scope, kit_icon_ids: Vec<IconId>) -> Self {
        Self {
            tree: ElementTree::new(style_sheet, theme),
            scope,
            event_handlers: EventHandlers::default(),
            child_fragments: ChildFragments::default(),
            kit_icon_ids,
            kit_icon_elements: HashMap::new(),
            removed_elements: Vec::new(),
            scroll_followers: Vec::new(),
            overlay: None,
            timer_handlers: HashMap::new(),
            due_timers: Vec::new(),
            native_menu: None,
            native_menu_choice: None,
        }
    }

    /// Starts a timer that runs `run` in the frame it falls due in: once `delay` has passed
    /// since the next frame of the surface. It goes with `owner`.
    pub(crate) fn start_timer(&mut self, owner: ElementId, delay: Duration, run: Action) -> TimerId {
        let timer = self.tree.start_timer(owner, delay);
        self.timer_handlers.insert(timer, TimerHandler { owner, run });
        timer
    }

    /// Stops the timer; one that fell due already is gone and stays so.
    pub(crate) fn cancel_timer(&mut self, timer: TimerId) {
        self.tree.cancel_timer(timer);
        self.timer_handlers.remove(&timer);
    }

    /// Runs what the timers that fall due by `now`, the time of the frame that begins, were
    /// started for. Called at the start of every frame, before its flush.
    pub(crate) fn run_due_timers(&mut self, runtime: &mut Runtime<Surfaces>, now: Duration) {
        self.tree.take_due_timers(now, &mut self.due_timers);
        for timer in self.due_timers.drain(..) {
            let mut handler = self
                .timer_handlers
                .remove(&timer)
                .expect("a timer of a surface is started with its handler");
            (handler.run)(runtime);
        }
    }

    /// Asks the host for a menu of the system; `on_choice` hears of the line the user chooses
    /// in it. A menu asked for before and not shown yet is given up.
    pub(crate) fn request_native_menu(&mut self, menu: NativeMenu, on_choice: NativeMenuChoice) {
        self.native_menu = Some(menu);
        self.native_menu_choice = Some(on_choice);
        self.tree.request_frame();
    }

    /// The menu of the system a component asked for since the host took the last one. The
    /// host shows it and tells the line the user chose through
    /// [`native_menu_command_chosen`](Self::native_menu_command_chosen).
    pub fn take_native_menu(&mut self) -> Option<NativeMenu> {
        self.native_menu.take()
    }

    /// The user chose the command `id` of the menu of the system the host took last.
    pub fn native_menu_command_chosen(&mut self, runtime: &mut Runtime<Surfaces>, id: usize) {
        if let Some(on_choice) = &mut self.native_menu_choice {
            on_choice(runtime, id);
        }
    }

    /// The user chose the line `id` of the menu bar, as the tree of the surface listed it: the
    /// handler of its shortcut runs.
    pub fn menu_command_chosen(&mut self, runtime: &mut Runtime<Surfaces>, id: usize) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.perform_menu_command(id, &mut dispatch);
    }

    /// The icon of the kit the element shows, for a host that draws it from its SVG.
    pub fn kit_icon_of(&self, element: ElementId) -> Option<KitIcon> {
        self.kit_icon_elements.get(&element).copied()
    }

    /// Notes that the element shows an icon of the kit, and shows it when the host
    /// registered the icons.
    pub(crate) fn show_kit_icon(&mut self, element: ElementId, icon: KitIcon) {
        self.kit_icon_elements.insert(element, icon);
        if let Some(id) = self.kit_icon_ids.get(icon.index()) {
            self.tree.set_icon(element, *id);
        }
    }

    /// Inserts what was typed, dictated or settled on by an input method into the text the
    /// focused element edits; with no such focus it does nothing.
    pub fn insert_text(&mut self, runtime: &mut Runtime<Surfaces>, text: &str, replacement: Option<Range<usize>>) {
        let Some(focused) = self.tree.focused_text_input() else {
            return;
        };
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.insert_text(focused, text, replacement, &mut dispatch);
    }

    /// Shows what an input method composes in the text the focused element edits.
    pub fn set_marked_text(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        text: &str,
        selected: Range<usize>,
        replacement: Option<Range<usize>>,
    ) {
        let Some(focused) = self.tree.focused_text_input() else {
            return;
        };
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree
            .set_marked_text(focused, text, selected, replacement, &mut dispatch);
    }

    /// Settles the text an input method composes in the focused text as it stands.
    pub fn unmark_text(&mut self) {
        if let Some(focused) = self.tree.focused_text_input() {
            self.tree.unmark_text(focused);
        }
    }

    /// Does a command of the Edit menu to the focused text. Returns whether a text took it.
    pub fn text_command(&mut self, runtime: &mut Runtime<Surfaces>, command: TextCommand) -> bool {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.perform_text_command(command, &mut dispatch)
    }

    pub fn tree(&self) -> &ElementTree {
        &self.tree
    }

    /// The tree itself, for the stages of a frame and for whoever drives its elements directly.
    pub fn tree_mut(&mut self) -> &mut ElementTree {
        &mut self.tree
    }

    /// Takes what the pointer did on the surface and lets the components hear of it.
    pub(crate) fn pointer_input(&mut self, runtime: &mut Runtime<Surfaces>, input: &PointerInput) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.pointer_input(input, &mut dispatch);
    }

    /// Works out again what a resting pointer is over once a frame moved the boxes under it:
    /// called at the start of every frame.
    pub(crate) fn refresh_hover(&mut self, runtime: &mut Runtime<Surfaces>) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.refresh_hover(&mut dispatch);
    }

    /// Writes how far each followed `Scroll` element is scrolled, and how large its viewport is,
    /// to the signal that follows it: what the input since the last frame and the last frame
    /// itself did to them. Called at the start of every frame, before its flush.
    pub(crate) fn follow_scrolls(&mut self, runtime: &mut Runtime<Surfaces>) {
        for (scroller, area) in &self.scroll_followers {
            let now = ScrollArea {
                offset: self.tree.scroll_offset(*scroller),
                viewport: self.tree.scroll_viewport(*scroller),
            };
            if runtime.untracked(|runtime| runtime.get(*area)) != now {
                runtime.set(*area, now);
            }
        }
    }

    /// Starts to follow the scroll of a `Scroll` element by a signal.
    pub(crate) fn follow_scroll(&mut self, runtime: &mut Runtime<Surfaces>, scroller: ElementId) -> Signal<ScrollArea> {
        let area = runtime.signal(ScrollArea {
            offset: self.tree.scroll_offset(scroller),
            viewport: self.tree.scroll_viewport(scroller),
        });
        self.scroll_followers.push((scroller, area));
        area
    }

    /// Takes a scroll of the wheel or the trackpad over the surface.
    pub fn scroll_input(&mut self, input: &ScrollInput) {
        self.tree.scroll_input(input);
    }

    /// Takes a key that went down or up. Returns whether a component, a shortcut or the focus
    /// of the tree used it.
    pub(crate) fn key_input(&mut self, runtime: &mut Runtime<Surfaces>, event: &KeyEvent) -> bool {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.key_input(event, &mut dispatch)
    }

    /// Takes what an assistive technology asks of a node of the accessibility tree.
    pub fn accessibility_action(&mut self, runtime: &mut Runtime<Surfaces>, request: &ActionRequest) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.accessibility_action(request, &mut dispatch);
    }

    /// The drag the host took from the tree of the surface for the system is over: the
    /// component of `source`, the element it began on, hears how it ended.
    pub fn system_drag_ended(&mut self, runtime: &mut Runtime<Surfaces>, source: ElementId, outcome: DragOutcome) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.system_drag_ended(source, outcome, &mut dispatch);
    }

    /// The drag the pointer carries from a component of the surface ended over another
    /// surface, dropped there or cancelled: the component hears how it ended.
    pub(crate) fn end_pointer_drag(&mut self, runtime: &mut Runtime<Surfaces>, outcome: DragOutcome) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.end_pointer_drag(outcome, &mut dispatch);
    }

    /// A drag that began outside the surface — a drag of the system, or of another surface of
    /// the application — came over it at `position`, carrying `payload`. Returns whether a
    /// drop target takes it there.
    pub fn system_drag_entered(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        payload: Payload,
        position: Point,
        modifiers: Modifiers,
    ) -> bool {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree
            .system_drag_entered(payload, position, modifiers, &mut dispatch)
    }

    /// The drag that began outside the surface moved to `position`. Returns whether a drop
    /// target takes it there.
    pub fn system_drag_moved(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        position: Point,
        modifiers: Modifiers,
    ) -> bool {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.system_drag_moved(position, modifiers, &mut dispatch)
    }

    /// The drag that began outside the surface left it, or ended somewhere else.
    pub fn system_drag_exited(&mut self, runtime: &mut Runtime<Surfaces>) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.system_drag_exited(&mut dispatch);
    }

    /// The drag that began outside the surface was dropped at `position`. Returns whether a
    /// drop target took it.
    pub fn system_drag_dropped(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        position: Point,
        modifiers: Modifiers,
    ) -> bool {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.system_drag_dropped(position, modifiers, &mut dispatch)
    }

    /// Removes the element with everything inside it from the tree, and with them what the
    /// components registered for them. The scope that owns their effects is disposed first.
    pub(crate) fn remove_subtree(&mut self, root: ElementId) {
        self.removed_elements.clear();
        self.removed_elements.push(root);
        let mut visited = 0;
        while let Some(element) = self.removed_elements.get(visited).copied() {
            self.removed_elements.extend(self.tree.children(element));
            visited += 1;
        }
        for element in &self.removed_elements {
            self.event_handlers.forget_element(*element);
            self.child_fragments.forget_parent(*element);
            self.kit_icon_elements.remove(element);
        }
        self.tree.remove(root);
        self.event_handlers.forget_shortcuts_of_removed_elements(&self.tree);
        let tree = &self.tree;
        self.scroll_followers.retain(|(scroller, _)| tree.contains(*scroller));
        self.timer_handlers.retain(|_, handler| tree.contains(handler.owner));
    }
}

impl Debug for Surface {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Surface")
            .field("tree", &self.tree)
            .field("scope", &self.scope)
            .field("event_handlers", &self.event_handlers)
            .field("overlay", &self.overlay)
            .field("timer_handlers", &self.timer_handlers)
            .field("native_menu", &self.native_menu)
            .finish_non_exhaustive()
    }
}
