mod support;

use std::path::PathBuf;

use morok_core::{
    CustomData, DisplayItem, DisplayList, DragEventKind, DragOutcome, DragReach, DragStart, ElementId, Focusability,
    Key, KeyChord, Modifiers, Payload, PayloadType, Point, PointerEventKind, Rect, Role, ShortcutScope, SystemDrag,
    TextSelection,
};
use morok_style::States;
use support::{InputLog, Scene};

const BOARD: &str = ".board { position: relative; width: 600px; height: 300px; }\n\
    .card { position: absolute; left: 20px; top: 20px; width: 100px; height: 60px; \
            background-color: var(--color-surface); }\n\
    .card:active { background-color: var(--color-accent); }\n\
    .lifted { transform: translateX(10px); }\n\
    .handle { width: 20px; height: 20px; background-color: var(--color-accent); }\n\
    .shelf { position: absolute; left: 200px; top: 20px; width: 150px; height: 200px; }\n\
    .shelf:hover { background-color: var(--color-surface-2); }\n\
    .slot { width: 150px; height: 50px; }\n\
    .bin { position: absolute; left: 400px; top: 20px; width: 150px; height: 200px; \
           background-color: var(--color-surface); }\n\
    .title { width: 80px; height: 20px; }\n\
    .list { position: absolute; left: 20px; top: 120px; width: 150px; height: 100px; overflow: auto; }\n\
    .entry { height: 40px; overflow: hidden; background-color: var(--color-surface); }\n\
    .badge { width: 100px; height: 10px; background-color: var(--color-accent); }";

/// The type of the application's own a card is dragged in.
const CARD_TYPE: &str = "com.example.board.card";

/// A point over the card and outside its handle, the same point once the pointer has travelled
/// too little to drag and once it has travelled far enough.
const ON_CARD: (f32, f32) = (70.0, 60.0);
const BARELY_MOVED: (f32, f32) = (72.0, 60.0);
const DRAGGED: (f32, f32) = (80.0, 60.0);
const ON_HANDLE: (f32, f32) = (30.0, 30.0);
/// Points over the slot of the shelf, over the shelf under its slot, over the bin, and over
/// the board alone.
const OVER_SLOT: (f32, f32) = (250.0, 40.0);
const OVER_SHELF: (f32, f32) = (250.0, 150.0);
const OVER_BIN: (f32, f32) = (450.0, 100.0);
const OVER_BOARD: (f32, f32) = (375.0, 280.0);

fn point(place: (f32, f32)) -> Point {
    Point { x: place.0, y: place.1 }
}

fn card_payload() -> Payload {
    Payload {
        text: Some(String::from("Buy milk")),
        custom: vec![CustomData {
            type_name: String::from(CARD_TYPE),
            bytes: vec![7, 0, 42],
        }],
        ..Payload::default()
    }
}

fn files_payload() -> Payload {
    Payload {
        files: vec![PathBuf::from("/tmp/report.pdf"), PathBuf::from("/tmp/notes")],
        ..Payload::default()
    }
}

/// A board with a card that is a drag source, a shelf that takes text, a slot on the shelf that
/// takes cards, and a bin that takes files; after its first frame.
struct Board {
    scene: Scene,
    log: InputLog,
    card: ElementId,
    handle: ElementId,
    shelf: ElementId,
    slot: ElementId,
    bin: ElementId,
}

impl Board {
    fn new() -> Self {
        let mut scene = Scene::new(BOARD);
        scene.add_classes(scene.root(), &["board"]);
        let card = scene.add(scene.root(), Role::Element, &["card"]);
        let handle = scene.add(card, Role::Element, &["handle"]);
        let shelf = scene.add(scene.root(), Role::Element, &["shelf"]);
        let slot = scene.add(shelf, Role::Element, &["slot"]);
        let bin = scene.add(scene.root(), Role::Element, &["bin"]);
        scene.tree.set_drag_source(card, true);
        scene.tree.set_drop_types(shelf, &[PayloadType::Text]);
        scene
            .tree
            .set_drop_types(slot, &[PayloadType::Custom(String::from(CARD_TYPE))]);
        scene.tree.set_drop_types(bin, &[PayloadType::Files]);
        scene.frame();
        Self {
            scene,
            log: InputLog::default(),
            card,
            handle,
            shelf,
            slot,
            bin,
        }
    }

    /// A board whose card answers the pointer that drags it with `payload`, to stay on the
    /// surface.
    fn dragging(payload: Payload) -> Self {
        let mut board = Self::new();
        board.log.drag_start = Some(DragStart {
            payload,
            reach: DragReach::Surface,
        });
        board
    }

    fn press(&mut self, place: (f32, f32)) {
        self.scene.press(place.0, place.1, &mut self.log);
    }

    fn move_to(&mut self, place: (f32, f32)) {
        self.scene.move_pointer(place.0, place.1, &mut self.log);
    }

    fn release(&mut self, place: (f32, f32)) {
        self.scene.release(place.0, place.1, &mut self.log);
    }

    /// Presses the card and moves the pointer far enough to drag it.
    fn drag_card(&mut self) {
        self.press(ON_CARD);
        self.move_to(DRAGGED);
    }

    fn is_hovered(&self, element: ElementId) -> bool {
        self.scene.tree.states(element).contains(States::HOVER)
    }

    fn is_active(&self, element: ElementId) -> bool {
        self.scene.tree.states(element).contains(States::ACTIVE)
    }

    /// The drag events `target` heard, in the order it heard them.
    fn drag_events_of(&self, target: ElementId) -> Vec<DragEventKind> {
        self.log
            .drag_events
            .iter()
            .filter(|(heard_by, _)| *heard_by == target)
            .map(|(_, kind)| *kind)
            .collect()
    }
}

#[test]
fn a_press_on_a_source_becomes_a_drag_once_the_pointer_has_travelled() {
    let mut board = Board::dragging(card_payload());
    let root = board.scene.root();

    board.press(ON_CARD);
    board.move_to(BARELY_MOVED);
    assert!(
        board.log.dragged_sources.is_empty(),
        "a pointer that barely moved drags nothing"
    );
    assert_eq!(board.log.receivers_of(PointerEventKind::Moved), [board.card, root]);
    assert!(board.is_active(board.card));
    assert_eq!(board.scene.tree.dragged_payload(), None);

    board.move_to(DRAGGED);
    assert_eq!(board.log.dragged_sources, [board.card]);
    assert_eq!(board.scene.tree.dragged_payload(), Some(&card_payload()));
    assert!(!board.is_active(board.card), "the press ended where the drag began");
    assert!(!board.is_hovered(board.card), "the pointer hovers nothing but a target");
    assert_eq!(
        board.log.receivers_of(PointerEventKind::Moved),
        [board.card, root],
        "the move that began the drag is the drag's"
    );
    assert_eq!(board.log.receivers_of(PointerEventKind::Left), [board.card, root]);

    board.release(DRAGGED);
    assert_eq!(board.log.ended_drags, [(board.card, DragOutcome::Cancelled)]);
    assert!(
        board.log.pointer_events.iter().all(|(_, kind)| !matches!(
            kind,
            PointerEventKind::Clicked { .. } | PointerEventKind::Released { .. }
        )),
        "a press that became a drag is neither released nor clicked: {:?}",
        board.log.pointer_events
    );
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert!(board.is_hovered(board.card), "the pointer hovers what it is over again");
}

#[test]
fn a_drag_enters_hovers_and_drops_on_a_target_with_its_flags_and_its_payload() {
    let mut board = Board::dragging(card_payload());
    let root = board.scene.root();
    board.drag_card();
    assert!(board.log.drag_events.is_empty(), "the card itself takes no drag");

    board.move_to(OVER_SHELF);
    assert_eq!(
        board.log.drag_events,
        [
            (board.shelf, DragEventKind::Entered),
            (board.shelf, DragEventKind::Moved)
        ]
    );
    assert!(board.is_hovered(board.shelf) && board.is_hovered(root));
    assert!(!board.is_hovered(board.card) && !board.is_active(board.card));
    board.scene.frame();
    assert_eq!(
        board.scene.tree.style(board.shelf).paint.background_color,
        board.scene.light_color("color-surface-2"),
        "the target under the drag is styled as hovered"
    );

    board.move_to((OVER_SHELF.0 + 10.0, OVER_SHELF.1));
    assert_eq!(
        board.drag_events_of(board.shelf),
        [DragEventKind::Entered, DragEventKind::Moved, DragEventKind::Moved]
    );

    board.release(OVER_SHELF);
    assert_eq!(
        board.drag_events_of(board.shelf),
        [
            DragEventKind::Entered,
            DragEventKind::Moved,
            DragEventKind::Moved,
            DragEventKind::Dropped
        ]
    );
    assert!(
        board
            .log
            .drag_event_payloads
            .iter()
            .all(|carried| *carried == card_payload()),
        "every event tells what the drag carries"
    );
    assert_eq!(board.log.ended_drags, [(board.card, DragOutcome::Dropped)]);
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert_eq!(
        board.log.receivers_of(PointerEventKind::Entered).last(),
        Some(&board.shelf),
        "the pointer comes over the target once the drag is over"
    );
    assert!(board.is_hovered(board.shelf));
}

#[test]
fn a_drag_is_offered_to_the_innermost_target_that_takes_a_type_it_carries() {
    let mut board = Board::dragging(card_payload());
    board.drag_card();

    board.move_to(OVER_SLOT);
    assert_eq!(
        board.log.drag_events,
        [(board.slot, DragEventKind::Entered), (board.slot, DragEventKind::Moved)]
    );
    assert!(
        board.is_hovered(board.slot) && board.is_hovered(board.shelf),
        "the elements around the target are hovered with it"
    );

    board.move_to(OVER_SHELF);
    assert_eq!(
        board.log.drag_events[2..],
        [
            (board.slot, DragEventKind::Left),
            (board.shelf, DragEventKind::Entered),
            (board.shelf, DragEventKind::Moved)
        ]
    );
    assert!(!board.is_hovered(board.slot) && board.is_hovered(board.shelf));

    let mut text_alone = Board::dragging(Payload::of_text("Buy milk"));
    text_alone.drag_card();
    text_alone.move_to(OVER_SLOT);
    assert_eq!(
        text_alone.log.drag_events,
        [
            (text_alone.shelf, DragEventKind::Entered),
            (text_alone.shelf, DragEventKind::Moved)
        ],
        "a slot that takes cards alone leaves a text to the shelf around it"
    );
    assert!(!text_alone.is_hovered(text_alone.slot));
}

#[test]
fn a_drag_over_nothing_that_takes_it_hovers_nothing_and_is_cancelled() {
    let mut board = Board::dragging(card_payload());
    let root = board.scene.root();
    board.drag_card();

    board.move_to(OVER_BIN);
    assert!(board.log.drag_events.is_empty(), "the bin takes files alone");
    assert!(!board.is_hovered(board.bin) && !board.is_hovered(root));

    board.move_to(OVER_SHELF);
    board.move_to(OVER_BOARD);
    assert_eq!(board.drag_events_of(board.shelf).last(), Some(&DragEventKind::Left));
    assert!(!board.is_hovered(board.shelf) && !board.is_hovered(root));

    board.release(OVER_BOARD);
    assert_eq!(board.log.ended_drags, [(board.card, DragOutcome::Cancelled)]);
    assert!(
        board
            .log
            .drag_events
            .iter()
            .all(|(_, kind)| *kind != DragEventKind::Dropped)
    );
}

#[test]
fn a_disabled_target_takes_no_drag_and_a_disabled_source_is_not_dragged() {
    let mut board = Board::dragging(card_payload());
    board.scene.tree.set_state(board.slot, States::DISABLED, true);
    board.drag_card();
    board.move_to(OVER_SLOT);
    assert_eq!(
        board.log.drag_events,
        [
            (board.shelf, DragEventKind::Entered),
            (board.shelf, DragEventKind::Moved)
        ],
        "the drag goes on to the target around the disabled one"
    );

    let mut disabled = Board::dragging(card_payload());
    disabled.scene.tree.set_state(disabled.card, States::DISABLED, true);
    disabled.drag_card();
    assert!(disabled.log.dragged_sources.is_empty());
    assert!(disabled.is_active(disabled.card), "the press stays a press");
}

#[test]
fn a_press_inside_a_source_drags_it_unless_an_element_inside_used_the_press() {
    let mut board = Board::dragging(card_payload());
    board.press(ON_HANDLE);
    board.move_to((ON_HANDLE.0 + 10.0, ON_HANDLE.1));
    assert_eq!(board.log.dragged_sources, [board.card]);

    let mut used = Board::dragging(card_payload());
    used.log.consumers.push(used.handle);
    used.press(ON_HANDLE);
    used.move_to((ON_HANDLE.0 + 10.0, ON_HANDLE.1));
    assert!(
        used.log.dragged_sources.is_empty(),
        "the handle used the press: it is the handle's"
    );
    assert!(used.is_active(used.handle));

    let mut used_by_source = Board::dragging(card_payload());
    used_by_source.log.consumers.push(used_by_source.card);
    used_by_source.press(ON_HANDLE);
    used_by_source.move_to((ON_HANDLE.0 + 10.0, ON_HANDLE.1));
    assert_eq!(used_by_source.log.dragged_sources, [used_by_source.card]);
}

#[test]
fn a_source_with_nothing_to_carry_leaves_the_press_a_press() {
    let mut board = Board::new();
    board.drag_card();
    board.move_to(OVER_SHELF);
    assert_eq!(board.log.dragged_sources, [board.card], "the source is asked once");
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert!(board.is_active(board.card));
    assert!(board.log.drag_events.is_empty());

    board.release(ON_CARD);
    assert_eq!(
        board.log.receivers_of(PointerEventKind::Clicked { click_count: 1 }),
        [board.card, board.scene.root()]
    );
    assert!(board.log.ended_drags.is_empty());
}

#[test]
fn escape_cancels_a_drag_and_reaches_nothing_else() {
    let mut board = Board::dragging(card_payload());
    let root = board.scene.root();
    let escape = KeyChord {
        key: Key::Escape,
        modifiers: Modifiers::NONE,
    };
    board.scene.tree.register_shortcut(ShortcutScope::Surface, escape);
    board.drag_card();
    board.move_to(OVER_SHELF);

    assert!(board.scene.press_key(Key::Escape, Modifiers::NONE, &mut board.log));
    assert_eq!(
        board.drag_events_of(board.shelf),
        [DragEventKind::Entered, DragEventKind::Moved, DragEventKind::Left]
    );
    assert_eq!(board.log.ended_drags, [(board.card, DragOutcome::Cancelled)]);
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert!(
        board.log.key_events.is_empty() && board.log.shortcuts.is_empty(),
        "the Escape that cancels a drag is offered to no element and to no shortcut"
    );
    assert!(
        board.is_hovered(board.shelf) && board.is_hovered(root),
        "the pointer hovers what it is over again"
    );

    board.release(OVER_SHELF);
    assert!(
        board
            .log
            .drag_events
            .iter()
            .all(|(_, kind)| *kind != DragEventKind::Dropped),
        "a cancelled drag is dropped nowhere"
    );
    assert_eq!(board.log.ended_drags.len(), 1);

    board.drag_card();
    assert_eq!(
        board.log.dragged_sources,
        [board.card, board.card],
        "the next press drags afresh"
    );
    assert_eq!(board.scene.tree.dragged_payload(), Some(&card_payload()));
    assert!(!board.scene.press_key(Key::Escape, Modifiers::SHIFT, &mut board.log));
    assert_eq!(
        board.scene.tree.dragged_payload(),
        Some(&card_payload()),
        "Escape with a modifier leaves the drag alone"
    );
    assert_eq!(board.log.key_receivers(), [root]);

    board.release(OVER_BOARD);
    assert!(
        board.scene.press_key(Key::Escape, Modifiers::NONE, &mut board.log),
        "with no drag under way Escape goes its usual way"
    );
    assert_eq!(board.log.key_receivers(), [root, root]);
    assert_eq!(board.log.shortcuts.len(), 1);
}

#[test]
fn a_press_in_the_selected_text_of_a_focused_input_inside_a_source_selects_and_drags_nothing() {
    let mut board = Board::dragging(card_payload());
    let title = board.scene.add(board.card, Role::TextInput, &["title"]);
    board.scene.tree.set_focusability(title, Focusability::TabStop);
    board.scene.tree.set_text(title, "Buy milk");
    assert!(board.scene.tree.focus(title));
    board
        .scene
        .tree
        .set_text_selection(title, TextSelection { anchor: 0, head: 8 });
    board.scene.frame();
    let title_box = board.scene.border_box(title);
    let (row, third_character, sixth_character) = (title_box.y + 10.0, title_box.x + 22.0, title_box.x + 52.0);

    board.press((third_character, row));
    board.move_to((sixth_character, row));

    assert!(
        board.log.dragged_sources.is_empty(),
        "the source is not asked to drag: the press selects"
    );
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert_eq!(
        board.scene.tree.text_selection(title),
        TextSelection { anchor: 2, head: 5 },
        "the pointer selects from where it went down"
    );
    board.release((sixth_character, row));
    assert!(board.log.ended_drags.is_empty());
}

#[test]
fn a_drag_handed_to_the_system_ends_the_press_and_waits_for_the_host() {
    let mut board = Board::new();
    board.log.drag_start = Some(DragStart {
        payload: files_payload(),
        reach: DragReach::System,
    });
    board.drag_card();

    assert!(!board.is_active(board.card), "the system has the pointer from here on");
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert_eq!(
        board.scene.tree.take_system_drag(),
        Some(SystemDrag {
            source: board.card,
            payload: files_payload(),
        })
    );
    assert_eq!(board.scene.tree.take_system_drag(), None, "a drag is taken once");

    board
        .scene
        .tree
        .system_drag_ended(board.card, DragOutcome::Dropped, &mut board.log);
    assert_eq!(board.log.ended_drags, [(board.card, DragOutcome::Dropped)]);

    board.scene.tree.remove(board.card);
    board
        .scene
        .tree
        .system_drag_ended(board.card, DragOutcome::Cancelled, &mut board.log);
    assert_eq!(
        board.log.ended_drags.len(),
        1,
        "a source that left the tree hears nothing"
    );
}

#[test]
fn a_drag_of_the_system_is_carried_over_the_targets_by_the_host() {
    let mut board = Board::new();
    board.move_to(ON_CARD);
    assert!(board.is_hovered(board.card));
    let (card, bin) = (board.card, board.bin);
    let (tree, log) = (&mut board.scene.tree, &mut board.log);

    assert!(
        !tree.system_drag_entered(files_payload(), point(OVER_BOARD), Modifiers::NONE, log),
        "the board takes nothing"
    );
    assert_eq!(tree.dragged_payload(), Some(&files_payload()));
    assert!(
        !tree.states(card).contains(States::HOVER),
        "the pointer hovers nothing while the system carries a drag over the surface"
    );

    assert!(tree.system_drag_moved(point(OVER_BIN), Modifiers::NONE, log));
    assert!(tree.states(bin).contains(States::HOVER));
    assert!(
        !tree.system_drag_moved(point(OVER_SHELF), Modifiers::NONE, log),
        "the shelf takes no files"
    );
    assert!(!tree.states(bin).contains(States::HOVER));
    assert!(tree.system_drag_dropped(point(OVER_BIN), Modifiers::OPTION, log));

    assert_eq!(
        log.drag_events,
        [
            (bin, DragEventKind::Entered),
            (bin, DragEventKind::Moved),
            (bin, DragEventKind::Left),
            (bin, DragEventKind::Entered),
            (bin, DragEventKind::Dropped)
        ]
    );
    assert_eq!(log.drag_event_payloads.last(), Some(&files_payload()));
    assert!(log.ended_drags.is_empty(), "the drag began on no source of the tree");
    assert_eq!(tree.dragged_payload(), None);
    assert!(!tree.states(bin).contains(States::HOVER));
    assert!(
        !tree.system_drag_moved(point(OVER_BIN), Modifiers::NONE, log),
        "a drag that is over takes nothing"
    );

    assert!(tree.system_drag_entered(files_payload(), point(OVER_BIN), Modifiers::NONE, log));
    tree.system_drag_exited(log);
    assert_eq!(log.drag_events.last(), Some(&(bin, DragEventKind::Left)));
    assert_eq!(tree.dragged_payload(), None);
    assert!(!tree.system_drag_dropped(point(OVER_BIN), Modifiers::NONE, log));
}

#[test]
fn pointer_input_ends_a_drag_of_the_system_the_host_never_told_the_end_of() {
    let mut board = Board::new();
    assert!(
        board
            .scene
            .tree
            .system_drag_entered(files_payload(), point(OVER_BIN), Modifiers::NONE, &mut board.log)
    );

    board.move_to(OVER_SHELF);
    assert_eq!(board.log.drag_events.last(), Some(&(board.bin, DragEventKind::Left)));
    assert_eq!(board.scene.tree.dragged_payload(), None);
    assert!(board.is_hovered(board.shelf) && !board.is_hovered(board.bin));
}

#[test]
fn a_target_that_leaves_the_tree_under_a_drag_is_forgotten() {
    let mut board = Board::dragging(card_payload());
    board.drag_card();
    board.move_to(OVER_SLOT);
    assert!(board.is_hovered(board.slot));

    board.scene.tree.remove(board.slot);
    assert!(
        !board.is_hovered(board.shelf),
        "nothing is hovered until the frame finds the target the drag is over"
    );
    board.scene.frame();
    board.scene.tree.refresh_hover(&mut board.log);
    assert_eq!(
        board.log.drag_events.last(),
        Some(&(board.shelf, DragEventKind::Entered))
    );
    assert!(board.is_hovered(board.shelf));

    board.scene.tree.remove(board.card);
    board.release(OVER_SHELF);
    assert_eq!(
        board.log.drag_events.last(),
        Some(&(board.shelf, DragEventKind::Dropped))
    );
    assert!(
        board.log.ended_drags.is_empty(),
        "a source that left the tree hears nothing"
    );
}

#[test]
fn the_types_the_targets_take_are_told_to_the_host_as_they_change() {
    let mut board = Board::new();
    let card_type = PayloadType::Custom(String::from(CARD_TYPE));
    let mut taken = board.scene.tree.accepted_drop_types();
    taken.sort_by_key(|payload_type| format!("{payload_type:?}"));
    assert_eq!(taken, [card_type.clone(), PayloadType::Files, PayloadType::Text]);
    let revision = board.scene.tree.drop_types_revision();

    board.scene.tree.set_drop_types(board.card, &[PayloadType::Text]);
    assert_eq!(
        board.scene.tree.drop_types_revision(),
        revision,
        "a type another target takes already changes nothing"
    );
    board.scene.tree.set_drop_types(board.shelf, &[]);
    assert_eq!(board.scene.tree.drop_types_revision(), revision);
    assert!(board.scene.tree.drop_types(board.shelf).is_empty());

    board.scene.tree.remove(board.shelf);
    assert_eq!(
        board.scene.tree.drop_types_revision(),
        revision + 1,
        "the slot went with the shelf, and no target takes cards"
    );
    assert!(!board.scene.tree.accepted_drop_types().contains(&card_type));
}

#[test]
fn the_picture_of_a_subtree_holds_what_the_surface_drew_of_it_and_nothing_else() {
    let mut board = Board::new();
    let (card_box, handle_box) = (board.scene.border_box(board.card), board.scene.border_box(board.handle));
    let mut picture = DisplayList::default();

    let covered = board.scene.tree.subtree_display_list(board.card, &mut picture);
    assert_eq!(covered, Some(card_box));
    let painted_boxes: Vec<Rect> = picture
        .items()
        .iter()
        .map(|item| match item {
            DisplayItem::Background { frame, .. } => frame.border_box,
            other => panic!("the card and its handle draw their backgrounds alone: {other:?}"),
        })
        .collect();
    assert_eq!(painted_boxes, [card_box, handle_box]);
    assert!(
        board.scene.tree.display_list().items().len() > picture.items().len(),
        "the surface draws the bin as well"
    );

    board.scene.add_classes(board.card, &["lifted"]);
    board.scene.frame();
    let covered = board.scene.tree.subtree_display_list(board.card, &mut picture);
    assert_eq!(
        covered,
        Some(Rect::new(
            card_box.x + 10.0,
            card_box.y,
            card_box.width,
            card_box.height
        )),
        "the picture lies where the surface shows the card"
    );
    assert!(
        matches!(
            picture.items(),
            [
                DisplayItem::PushGroup { .. },
                DisplayItem::Background { .. },
                DisplayItem::Background { .. },
                DisplayItem::PopGroup
            ]
        ),
        "the group that moves the card moves its picture: {:?}",
        picture.items()
    );

    assert_eq!(
        board.scene.tree.subtree_display_list(board.slot, &mut picture),
        Some(board.scene.border_box(board.slot))
    );
    assert!(
        picture.items().is_empty(),
        "a slot that paints nothing has an empty picture"
    );
}

#[test]
fn the_picture_of_a_subtree_holds_the_part_the_clip_of_an_ancestor_hides() {
    let mut scene = Scene::new(BOARD);
    scene.add_classes(scene.root(), &["board"]);
    let list = scene.add(scene.root(), Role::Scroll, &["list"]);
    let entries: Vec<ElementId> = (0..5).map(|_| scene.add(list, Role::Element, &["entry"])).collect();
    let badge = scene.add(entries[0], Role::Element, &["badge"]);
    scene.frame();
    scene.tree.scroll_to(list, Point { x: 0.0, y: 20.0 });
    scene.frame();
    let (list_box, entry_box) = (scene.border_box(list), scene.border_box(entries[0]));
    let mut picture = DisplayList::default();

    let covered = scene.tree.subtree_display_list(entries[0], &mut picture);
    assert_eq!(
        covered,
        Some(Rect::new(
            entry_box.x,
            entry_box.y - 20.0,
            entry_box.width,
            entry_box.height
        )),
        "the picture covers the whole entry, the half the scroll moved above the list included"
    );
    assert!(
        covered.is_some_and(|covered| covered.y < list_box.y),
        "the top of the entry lies above the list that clips it"
    );
    let [
        DisplayItem::PushGroup { .. },
        DisplayItem::Background { frame: entry_frame, .. },
        DisplayItem::PushClip { frame: clip_frame },
        DisplayItem::Background { frame: badge_frame, .. },
        DisplayItem::PopClip,
        DisplayItem::PopGroup,
    ] = picture.items()
    else {
        panic!(
            "the scroll moves the picture, the entry clips its badge, and the list clips nothing: {:?}",
            picture.items()
        );
    };
    assert_eq!(entry_frame.border_box, entry_box);
    assert_eq!(
        clip_frame.border_box, entry_box,
        "the clip that holds is the entry's own"
    );
    assert_eq!(badge_frame.border_box, scene.border_box(badge));
    assert!(
        scene
            .tree
            .display_list()
            .items()
            .iter()
            .any(|item| matches!(item, DisplayItem::PushClip { frame } if frame.border_box == list_box)),
        "the surface draws the entries inside the clip of the list"
    );
}
