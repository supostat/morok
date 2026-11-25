mod support;

use std::borrow::Cow;
use std::time::Duration;

use morok_components::view::{SurfaceFrame, SurfaceStage, Surfaces};
use morok_core::{FrameStage, Point, StageObserver};
use morok_reactive::{FlushCycle, Runtime};
use morok_style::{Declaration, Display, FlexDirection, LengthPercentage, LengthPercentageAuto, Pixels, States};
use support::Stage;

const FRAME_INTERVAL: Duration = Duration::from_millis(16);
const DELAY: Duration = Duration::from_millis(300);
/// A millisecond short of the delay: `ALMOST_THE_DELAY` and `ONE_MILLISECOND` add up to it.
const ALMOST_THE_DELAY: Duration = Duration::from_millis(299);
const ONE_MILLISECOND: Duration = Duration::from_millis(1);

/// Writes down the stages of a frame in the order they tell of their end.
#[derive(Debug, Default)]
struct StageLog {
    ended: Vec<SurfaceStage>,
}

impl StageObserver<SurfaceStage> for StageLog {
    fn stage_ended(&mut self, stage: SurfaceStage) {
        self.ended.push(stage);
    }
}

fn height(length: f32) -> Declaration {
    Declaration::Height(LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    }))
}

#[test]
fn a_frame_runs_hover_scrolls_timers_and_effects_and_then_the_stages_of_the_tree() {
    let mut stage = Stage::new();
    stage.mount(|mount| mount.element().inline(height(50.0)).id());
    let mut log = StageLog::default();

    let frame = stage
        .frame_observed(FRAME_INTERVAL, &mut log)
        .expect("the components of the test have no cycle");

    assert!(matches!(frame, SurfaceFrame::Ran(_)), "{frame:?}");
    assert_eq!(
        log.ended,
        [
            SurfaceStage::Hover,
            SurfaceStage::Scrolls,
            SurfaceStage::Timers,
            SurfaceStage::Effects,
            SurfaceStage::Tree(FrameStage::Styles),
            SurfaceStage::Tree(FrameStage::Images),
            SurfaceStage::Tree(FrameStage::Layout),
            SurfaceStage::Tree(FrameStage::DisplayList),
        ]
    );
}

#[test]
fn what_a_delay_binds_shows_in_the_frame_the_delay_runs_out_in() {
    let mut stage = Stage::new();
    let is_pressed = stage.runtime.signal(false);
    let element = stage.mount(move |mount| {
        let mut element = mount.element();
        let has_been_pressed = element.held_for(is_pressed, DELAY);
        element.state(States::CHECKED, has_been_pressed);
        element.id()
    });
    stage.frame();
    stage.runtime.set(is_pressed, true);
    stage.frame();

    stage.frame_all_after(ALMOST_THE_DELAY);
    assert!(!stage.tree().states(element).contains(States::CHECKED));

    stage.frame_all_after(ONE_MILLISECOND);
    assert!(
        stage.tree().states(element).contains(States::CHECKED),
        "the timer runs before the flush and the stages of the same frame"
    );
}

#[test]
fn a_resting_pointer_hovers_the_box_the_last_frame_moved_under_it_in_the_next_frame() {
    let mut stage = Stage::new();
    let spacer_height = stage.runtime.signal(height(100.0));
    let column = stage.mount(move |mount| {
        let spacer = mount.element().inline(spacer_height).id();
        let moved = mount.element().inline(height(50.0)).id();
        let mut column = mount.element();
        column
            .inline(Declaration::Display(Display::Flex))
            .inline(Declaration::FlexDirection(FlexDirection::Column))
            .child(spacer)
            .child(moved);
        column.id()
    });
    stage.frame();
    let moved = stage.children(column)[1];
    stage.move_pointer(Point { x: 10.0, y: 25.0 });
    stage.frame();

    stage.runtime.set(spacer_height, height(0.0));
    stage.frame();
    assert!(
        !stage.tree().states(moved).contains(States::HOVER),
        "the frame that moves the box is laid out after the pointer was hit-tested"
    );

    stage.frame();
    assert!(stage.tree().states(moved).contains(States::HOVER));
}

#[test]
fn an_effect_that_closes_the_surface_in_its_frame_leaves_its_tree_without_a_stage() {
    let mut stage = Stage::new();
    let surface = stage.surface;
    let closes = stage.runtime.signal(false);
    stage
        .runtime
        .effect(move |runtime: &mut Runtime<Surfaces>, surfaces: &mut Surfaces| {
            if runtime.get(closes) {
                surfaces.close(runtime, surface);
            }
        });
    stage.frame();
    stage.runtime.set(closes, true);
    let mut log = StageLog::default();

    let frame = stage
        .frame_observed(FRAME_INTERVAL, &mut log)
        .expect("the components of the test have no cycle");

    assert_eq!(frame, SurfaceFrame::Closed);
    assert_eq!(
        log.ended,
        [
            SurfaceStage::Hover,
            SurfaceStage::Scrolls,
            SurfaceStage::Timers,
            SurfaceStage::Effects
        ]
    );
}

#[test]
fn an_effect_that_keeps_waking_itself_ends_the_frame_with_its_cycle() {
    let mut stage = Stage::new();
    let count = stage.runtime.signal(0_u32);
    let waking = stage
        .runtime
        .effect(move |runtime: &mut Runtime<Surfaces>, _surfaces: &mut Surfaces| {
            let seen = runtime.get(count);
            runtime.set(count, seen + 1);
        });
    let mut log = StageLog::default();

    let frame = stage.frame_observed(FRAME_INTERVAL, &mut log);

    assert_eq!(frame, Err(FlushCycle { effect: waking }));
    assert_eq!(
        log.ended,
        [SurfaceStage::Hover, SurfaceStage::Scrolls, SurfaceStage::Timers]
    );
}
