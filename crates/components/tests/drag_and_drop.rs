mod support;

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use morok_components::styles::CLASSES;
use morok_components::view::{ElementEvent, Mount, SurfaceId};
use morok_components::{Button, Popover};
use morok_core::{
    DragEventKind, DragOutcome, DragReach, DragStart, ElementId, Key, Modifiers, Payload, PayloadType, Point,
    PointerButton, PointerInputKind, Propagation, SystemDrag,
};
use morok_reactive::Signal;
use morok_style::States;
use support::Stage;

/// What a drop target of the test shows of the drags over it, as signals.
#[derive(Clone, Copy)]
struct Shelf {
    /// Whether a drag the shelf takes is over it.
    is_under_drag: Signal<bool>,
    /// What the last drag dropped on the shelf carried.
    taken: Signal<Payload>,
}

impl Shelf {
    fn new(stage: &mut Stage) -> Self {
        Self {
            is_under_drag: stage.runtime.signal(false),
            taken: stage.runtime.signal(Payload::default()),
        }
    }

    /// A box that takes drags of the types and is checked while one is over it.
    fn mount(self, mount: &mut Mount<'_>, types: &[PayloadType]) -> ElementId {
        mount
            .element()
            .class(CLASSES.button)
            .drop_target(types)
            .state(States::CHECKED, self.is_under_drag)
            .on_event(move |runtime, event| {
                if let ElementEvent::Drag(drag) = event {
                    match drag.kind {
                        DragEventKind::Entered => runtime.set(self.is_under_drag, true),
                        DragEventKind::Moved => {}
                        DragEventKind::Left => runtime.set(self.is_under_drag, false),
                        DragEventKind::Dropped => {
                            runtime.set(self.is_under_drag, false);
                            runtime.set(self.taken, drag.payload.clone());
                        }
                    }
                }
                Propagation::Continue
            })
            .id()
    }
}

/// A box the pointer drags with `payload` as far as `reach`; `ended` hears how its drags end.
fn card(mount: &mut Mount<'_>, payload: Payload, reach: DragReach, ended: Signal<Option<DragOutcome>>) -> ElementId {
    mount
        .element()
        .class(CLASSES.button)
        .drag_source(move |_| {
            Some(DragStart {
                payload: payload.clone(),
                reach,
            })
        })
        .on_event(move |runtime, event| {
            if let ElementEvent::DragEnded(outcome) = event {
                runtime.set(ended, Some(*outcome));
            }
            Propagation::Continue
        })
        .id()
}

fn files() -> Payload {
    Payload {
        files: vec![PathBuf::from("/tmp/report.pdf")],
        ..Payload::default()
    }
}

#[test]
fn a_component_drags_what_its_source_carries_onto_a_drop_target() {
    let mut stage = Stage::new();
    let shelf = Shelf::new(&mut stage);
    let ended = stage.runtime.signal(None);
    let source = stage.mount(move |mount| card(mount, Payload::of_text("a row"), DragReach::Surface, ended));
    let target = stage.mount(move |mount| shelf.mount(mount, &[PayloadType::Text]));
    stage.frame();
    let (on_source, over_target) = (stage.middle_of(source), stage.middle_of(target));
    let primary = PointerButton::Primary;

    stage.pointer_on(
        stage.surface,
        PointerInputKind::Pressed {
            button: primary,
            click_count: 1,
        },
        on_source,
    );
    stage.pointer_on(stage.surface, PointerInputKind::Moved, over_target);
    stage.frame();
    let states = stage.tree().states(target);
    assert!(
        states.contains(States::HOVER),
        "the tree hovers the target under the drag"
    );
    assert!(
        states.contains(States::CHECKED),
        "the component shows the drag over it through its signal"
    );
    assert_eq!(stage.tree().dragged_payload(), Some(&Payload::of_text("a row")));

    stage.pointer_on(
        stage.surface,
        PointerInputKind::Released {
            button: primary,
            click_count: 1,
        },
        over_target,
    );
    stage.frame();
    assert_eq!(stage.runtime.get(shelf.taken), Payload::of_text("a row"));
    assert_eq!(stage.runtime.get(ended), Some(DragOutcome::Dropped));
    assert!(!stage.tree().states(target).contains(States::CHECKED));
}

#[test]
fn a_drag_handed_to_the_system_reaches_the_host_and_its_end_reaches_the_component() {
    let mut stage = Stage::new();
    let ended = stage.runtime.signal(None);
    let source = stage.mount(move |mount| card(mount, files(), DragReach::System, ended));
    stage.frame();
    let on_source = stage.middle_of(source);
    let mut dragged_to = on_source;
    dragged_to.x += 10.0;

    stage.pointer_on(
        stage.surface,
        PointerInputKind::Pressed {
            button: PointerButton::Primary,
            click_count: 1,
        },
        on_source,
    );
    stage.pointer_on(stage.surface, PointerInputKind::Moved, dragged_to);
    let surface = stage.surfaces.surface_mut(stage.surface);
    assert_eq!(
        surface.tree_mut().take_system_drag(),
        Some(SystemDrag {
            source,
            payload: files(),
        })
    );

    surface.system_drag_ended(&mut stage.runtime, source, DragOutcome::Cancelled);
    assert_eq!(stage.runtime.get(ended), Some(DragOutcome::Cancelled));
}

#[test]
fn a_drag_of_the_system_reaches_the_components_through_the_surface() {
    let mut stage = Stage::new();
    let shelf = Shelf::new(&mut stage);
    let target = stage.mount(move |mount| shelf.mount(mount, &[PayloadType::Files]));
    stage.frame();
    let over_target = stage.middle_of(target);
    let surface = stage.surfaces.surface_mut(stage.surface);

    assert!(surface.system_drag_entered(&mut stage.runtime, files(), over_target, Modifiers::NONE));
    assert!(stage.runtime.get(shelf.is_under_drag));
    surface.system_drag_exited(&mut stage.runtime);
    assert!(!stage.runtime.get(shelf.is_under_drag));

    assert!(surface.system_drag_entered(&mut stage.runtime, files(), over_target, Modifiers::NONE));
    assert!(surface.system_drag_moved(&mut stage.runtime, over_target, Modifiers::NONE));
    assert!(surface.system_drag_dropped(&mut stage.runtime, over_target, Modifiers::NONE));
    assert_eq!(stage.runtime.get(shelf.taken), files());
    assert!(!stage.runtime.get(shelf.is_under_drag));
}

/// A stage with a card on its surface that the pointer drags with a text, and a shelf that
/// takes texts on another surface, after the first frame of each.
struct TwoSurfaces {
    stage: Stage,
    other: SurfaceId,
    shelf: Shelf,
    ended: Signal<Option<DragOutcome>>,
    on_card: Point,
    over_shelf: Point,
}

impl TwoSurfaces {
    fn new() -> Self {
        let mut stage = Stage::new();
        let other = stage.open_surface();
        let shelf = Shelf::new(&mut stage);
        let ended = stage.runtime.signal(None);
        let source = stage.mount(move |mount| card(mount, Payload::of_text("a row"), DragReach::Surface, ended));
        let target = stage.mount_on(other, move |mount| shelf.mount(mount, &[PayloadType::Text]));
        stage.frame();
        stage.frame_surface(other);
        let (on_card, over_shelf) = (stage.middle_of(source), stage.middle_on(other, target));
        Self {
            stage,
            other,
            shelf,
            ended,
            on_card,
            over_shelf,
        }
    }

    /// Presses the card and moves the pointer far enough to drag it.
    fn drag_card(&mut self) {
        let surface = self.stage.surface;
        self.stage.pointer_on(surface, PRESSED, self.on_card);
        self.stage
            .pointer_on(surface, PointerInputKind::Moved, beside(self.on_card));
    }

    /// The pointer that drags the card goes over the shelf of the other surface, a place the
    /// surface of the card measures far past its own right edge.
    fn over_the_shelf(&mut self, kind: PointerInputKind) {
        let (surface, other, over_shelf) = (self.stage.surface, self.other, self.over_shelf);
        self.stage.pointer_over(surface, kind, FAR_RIGHT, other, over_shelf);
    }
}

const PRESSED: PointerInputKind = PointerInputKind::Pressed {
    button: PointerButton::Primary,
    click_count: 1,
};
const RELEASED: PointerInputKind = PointerInputKind::Released {
    button: PointerButton::Primary,
    click_count: 1,
};
/// A place past the right edge of the surface of the stage.
const FAR_RIGHT: Point = Point { x: 1200.0, y: 300.0 };

/// A point far enough from `point` for a press there to become a drag.
fn beside(point: Point) -> Point {
    Point {
        x: point.x + 10.0,
        ..point
    }
}

#[test]
fn a_drag_crosses_to_another_surface_and_drops_on_its_target() {
    let mut surfaces = TwoSurfaces::new();
    surfaces.drag_card();
    surfaces.over_the_shelf(PointerInputKind::Moved);
    let (stage, other, shelf) = (&mut surfaces.stage, surfaces.other, surfaces.shelf);
    assert!(stage.runtime.get(shelf.is_under_drag));
    assert_eq!(
        stage.tree_of(other).dragged_payload(),
        Some(&Payload::of_text("a row")),
        "the other surface takes the drag of the card"
    );
    assert_eq!(
        stage.tree().pointer_drag(),
        Some(&Payload::of_text("a row")),
        "the drag stays the pointer's on the surface of the card"
    );

    surfaces.over_the_shelf(RELEASED);
    let stage = &mut surfaces.stage;
    assert_eq!(stage.runtime.get(shelf.taken), Payload::of_text("a row"));
    assert_eq!(stage.runtime.get(surfaces.ended), Some(DragOutcome::Dropped));
    assert!(!stage.runtime.get(shelf.is_under_drag));
    assert_eq!(stage.tree().dragged_payload(), None);
    assert_eq!(stage.tree_of(other).dragged_payload(), None);
}

#[test]
fn a_drag_back_over_its_own_surface_leaves_the_other_and_a_release_over_nothing_cancels_it() {
    let mut surfaces = TwoSurfaces::new();
    surfaces.drag_card();
    surfaces.over_the_shelf(PointerInputKind::Moved);
    let (surface, shelf) = (surfaces.stage.surface, surfaces.shelf);

    surfaces
        .stage
        .pointer_on(surface, PointerInputKind::Moved, beside(surfaces.on_card));
    assert!(
        !surfaces.stage.runtime.get(shelf.is_under_drag),
        "the shelf hears that the drag left"
    );
    assert_eq!(surfaces.stage.tree_of(surfaces.other).dragged_payload(), None);

    surfaces.stage.pointer_on(surface, RELEASED, FAR_RIGHT);
    assert_eq!(surfaces.stage.runtime.get(surfaces.ended), Some(DragOutcome::Cancelled));
    assert_eq!(surfaces.stage.runtime.get(shelf.taken), Payload::default());
}

#[test]
fn escape_cancels_a_drag_over_another_surface_whose_target_hears_that_it_left() {
    let mut surfaces = TwoSurfaces::new();
    surfaces.drag_card();
    surfaces.over_the_shelf(PointerInputKind::Moved);
    let (other, shelf) = (surfaces.other, surfaces.shelf);
    assert!(surfaces.stage.runtime.get(shelf.is_under_drag));

    assert!(surfaces.stage.press_key_on(other, Key::Escape));
    assert!(!surfaces.stage.runtime.get(shelf.is_under_drag));
    assert_eq!(surfaces.stage.runtime.get(surfaces.ended), Some(DragOutcome::Cancelled));
    assert_eq!(surfaces.stage.tree().dragged_payload(), None);
    assert_eq!(surfaces.stage.tree_of(other).dragged_payload(), None);

    surfaces.over_the_shelf(RELEASED);
    assert_eq!(
        surfaces.stage.runtime.get(shelf.taken),
        Payload::default(),
        "a cancelled drag is dropped nowhere"
    );
}

#[test]
fn a_drag_leaves_the_panel_of_an_overlay_for_the_surface_under_it() {
    let mut stage = Stage::new();
    let shelf = Shelf::new(&mut stage);
    let ended = stage.runtime.signal(None);
    let is_open = stage.runtime.signal(true);
    let target = stage.mount(move |mount| shelf.mount(mount, &[PayloadType::Text]));
    let dragged = Rc::new(Cell::new(None));
    let card_of_panel = Rc::clone(&dragged);
    stage.mount(move |mount| {
        let trigger = Button::new("Open").mount(mount);
        Popover::new(is_open)
            .title("Rows")
            .content(move |mount| {
                let row = card(mount, Payload::of_text("a row"), DragReach::Surface, ended);
                card_of_panel.set(Some(row));
                row
            })
            .attach(mount, trigger);
        trigger
    });
    stage.frame_all();
    stage.frame_all();
    let overlays = stage.overlays_of(stage.surface);
    let [overlay] = overlays[..] else {
        panic!("the popover is open");
    };
    let row = dragged.get().expect("the popover shows its content");
    let on_row = stage.middle_on(overlay, row);
    let origin = stage.origin_of(overlay);
    let over_shelf = stage.middle_of(target);
    let over_shelf_on_overlay = Point {
        x: over_shelf.x - origin.x,
        y: over_shelf.y - origin.y,
    };

    stage.pointer_on(overlay, PRESSED, on_row);
    stage.pointer_on(overlay, PointerInputKind::Moved, beside(on_row));
    assert_eq!(stage.tree_of(overlay).pointer_drag(), Some(&Payload::of_text("a row")));
    stage.pointer_on(overlay, PointerInputKind::Moved, over_shelf_on_overlay);
    assert!(
        stage.runtime.get(shelf.is_under_drag),
        "the surface under the overlay takes the drag in its own coordinates"
    );

    stage.pointer_on(overlay, RELEASED, over_shelf_on_overlay);
    assert_eq!(stage.runtime.get(shelf.taken), Payload::of_text("a row"));
    assert_eq!(stage.runtime.get(ended), Some(DragOutcome::Dropped));
}
