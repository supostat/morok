mod support;

use std::path::PathBuf;

use morok_components::styles::CLASSES;
use morok_components::view::{ElementEvent, Mount};
use morok_core::{
    DragEventKind, DragOutcome, DragReach, DragStart, ElementId, Modifiers, Payload, PayloadType, PointerButton,
    PointerInputKind, Propagation, SystemDrag,
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
