use std::collections::HashMap;
use std::ops::Range;

use morok_core::accesskit::ActionRequest;
use morok_core::{ElementId, ElementTree, IconId, KeyEvent, PointerInput, ScrollInput, TextCommand};
use morok_reactive::{Runtime, Scope, Signal};
use morok_style::{StyleSheet, Theme};

use crate::kit_icon::KitIcon;
use crate::view::child_fragments::ChildFragments;
use crate::view::event_dispatch::EventDispatch;
use crate::view::event_handlers::EventHandlers;
use crate::view::scroll_area::ScrollArea;
use crate::view::surfaces::Surfaces;

/// One surface as the components see it: its element tree and what they registered for its
/// elements. The host of the surface gives it the input of the surface and runs the stages of
/// its tree.
#[derive(Debug)]
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
        }
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
    pub fn pointer_input(&mut self, runtime: &mut Runtime<Surfaces>, input: &PointerInput) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.pointer_input(input, &mut dispatch);
    }

    /// Works out again what a resting pointer is over once a frame moved the boxes under it:
    /// called at the start of every frame.
    pub fn refresh_hover(&mut self, runtime: &mut Runtime<Surfaces>) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.refresh_hover(&mut dispatch);
    }

    /// Writes how far each followed `Scroll` element is scrolled, and how large its viewport is,
    /// to the signal that follows it: what the input since the last frame and the last frame
    /// itself did to them. Called at the start of every frame, before its flush.
    pub fn follow_scrolls(&mut self, runtime: &mut Runtime<Surfaces>) {
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
    pub fn key_input(&mut self, runtime: &mut Runtime<Surfaces>, event: &KeyEvent) -> bool {
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
    }
}
