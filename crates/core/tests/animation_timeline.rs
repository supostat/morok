mod support;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use morok_core::{DisplayItem, ElementId, FrameScheduler, Role, Spring, SpringSample};
use morok_style::{Color, States};
use support::Scene;

/// The ease curve of CSS, `cubic-bezier(0.25, 0.1, 0.25, 1)`, at half time, as browsers solve it.
const EASE_AT_HALF_TIME: f32 = 0.802_403_4;
const STIFFNESS: f32 = 170.0;
const PRECISION: f32 = 0.01;

fn milliseconds(milliseconds: u64) -> Duration {
    Duration::from_millis(milliseconds)
}

fn assert_near(actual: f32, expected: f32, what: &str) {
    assert!((actual - expected).abs() < 1e-3, "{what}: {actual} is not {expected}");
}

/// Counts the frames a tree asked its host for.
#[derive(Debug)]
struct CountingScheduler {
    requested_frames: Rc<Cell<usize>>,
}

impl FrameScheduler for CountingScheduler {
    fn request_frame(&self) {
        self.requested_frames.set(self.requested_frames.get() + 1);
    }
}

/// A scene of the rules with one element of the classes on a block root, after its first frame
/// at time zero.
fn subject_of(rules: &str, class_names: &[&str]) -> (Scene, ElementId) {
    let mut scene = Scene::new(&format!(".stage {{ display: block; }}\n{rules}"));
    scene.add_classes(scene.root(), &["stage"]);
    let subject = scene.add(scene.root(), Role::Element, class_names);
    scene.frame_at(Duration::ZERO);
    (scene, subject)
}

fn hosted(scene: &mut Scene) -> Rc<Cell<usize>> {
    let requested_frames = Rc::new(Cell::new(0));
    scene.tree.set_frame_scheduler(Box::new(CountingScheduler {
        requested_frames: Rc::clone(&requested_frames),
    }));
    requested_frames
}

fn opacity(scene: &Scene, element: ElementId) -> f32 {
    scene.tree.style(element).effects.opacity
}

fn background(scene: &Scene, element: ElementId) -> Color {
    scene.tree.style(element).paint.background_color
}

fn shadow_layers(scene: &Scene) -> Vec<morok_style::ShadowLayer> {
    scene
        .tree
        .display_list()
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::Shadow { layer, .. } => Some(*layer),
            _ => None,
        })
        .collect()
}

/// Opacity transitions from 0 to 1 over a second, with the easing in `easing`, started at time zero.
fn opacity_transition(easing: &str) -> (Scene, ElementId) {
    let (mut scene, subject) = subject_of(
        &format!(
            ".fader {{ width: 10px; height: 10px; opacity: 0; transition: opacity 1s {easing}; }}\n\
             .fader:hover {{ opacity: 1; }}"
        ),
        &["fader"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);
    (scene, subject)
}

fn opacity_at(easing: &str, time: Duration) -> f32 {
    let (mut scene, subject) = opacity_transition(easing);
    scene.frame_at(time);
    opacity(&scene, subject)
}

#[test]
fn a_linear_transition_moves_in_step_with_time() {
    let (mut scene, subject) = opacity_transition("linear");

    assert_near(opacity(&scene, subject), 0.0, "the value at this time");
    scene.frame_at(milliseconds(250));
    assert_near(opacity(&scene, subject), 0.25, "a quarter of the way");
    scene.frame_at(milliseconds(1000));
    assert_near(opacity(&scene, subject), 1.0, "the value at this time");
}

#[test]
fn a_cubic_bezier_is_solved_as_browsers_solve_it() {
    assert_near(
        opacity_at("cubic-bezier(0.25, 0.1, 0.25, 1)", milliseconds(500)),
        EASE_AT_HALF_TIME,
        "ease at half time",
    );
    assert_near(
        opacity_at("cubic-bezier(0, 0, 1, 1)", milliseconds(300)),
        0.3,
        "a straight curve",
    );
    let overshoot = opacity_at("cubic-bezier(0.3, 1.8, 0.6, 1)", milliseconds(500));
    assert_near(
        overshoot,
        1.0,
        "an overshooting curve is clamped to what opacity can take",
    );
}

#[test]
fn an_omitted_easing_is_ease() {
    let (mut scene, subject) = subject_of(
        ".fader { width: 10px; height: 10px; opacity: 0; transition: opacity 1s; }\n.fader:hover { opacity: 1; }",
        &["fader"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    scene.frame_at(milliseconds(500));

    assert_near(opacity(&scene, subject), EASE_AT_HALF_TIME, "ease at half time");
}

#[test]
fn steps_jump_where_their_position_puts_the_jumps() {
    let at_thirty_percent = |easing: &str| opacity_at(easing, milliseconds(300));

    assert_near(at_thirty_percent("steps(4)"), 0.25, "thirty per cent through");
    assert_near(
        at_thirty_percent("steps(4, jump-start)"),
        0.5,
        "thirty per cent through",
    );
    assert_near(
        at_thirty_percent("steps(5, jump-none)"),
        0.25,
        "thirty per cent through",
    );
    assert_near(
        at_thirty_percent("steps(3, jump-both)"),
        0.25,
        "thirty per cent through",
    );
    assert_near(
        opacity_at("steps(2, jump-start)", Duration::ZERO),
        0.5,
        "a jump at the start is taken at the start",
    );
}

#[test]
fn a_delay_holds_the_start_and_a_negative_delay_starts_part_way() {
    let (mut scene, subject) = subject_of(
        ".late { width: 10px; height: 10px; opacity: 0; transition: opacity 1s linear 200ms; }\n\
         .early { width: 10px; height: 10px; opacity: 0; transition: opacity 1s linear -400ms; }\n\
         .late:hover, .early:hover { opacity: 1; }",
        &["late"],
    );
    let early = scene.add(scene.root(), Role::Element, &["early"]);
    scene.frame_at(Duration::ZERO);
    scene.tree.set_state(subject, States::HOVER, true);
    scene.tree.set_state(early, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    assert_near(opacity(&scene, subject), 0.0, "the value at this time");
    assert_near(opacity(&scene, early), 0.4, "the negative delay is already behind");
    scene.frame_at(milliseconds(150));
    assert_near(opacity(&scene, subject), 0.0, "still in the delay");
    scene.frame_at(milliseconds(700));
    assert_near(opacity(&scene, subject), 0.5, "half way after the delay");
}

#[test]
fn a_length_transition_moves_the_box_through_layout() {
    let (mut scene, subject) = subject_of(
        ".bar { width: 100px; height: 20px; transition: width 1s linear, padding 1s linear; }\n\
         .bar:hover { width: 200px; padding-left: 40px; }",
        &["bar"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    scene.frame_at(milliseconds(500));

    assert_near(scene.border_box(subject).width, 150.0, "the value at this time");
    assert_near(
        scene.tree.style(subject).layout.padding.left.pixels,
        20.0,
        "the value at this time",
    );
}

#[test]
fn colors_are_interpolated_on_premultiplied_channels() {
    let (mut scene, subject) = subject_of(
        ".swatch { width: 10px; height: 10px; background-color: var(--color-surface); \
         transition: background-color 1s linear; }\n\
         .swatch:hover { background-color: var(--color-accent); }\n\
         .glass { width: 10px; height: 10px; background-color: transparent; transition: background-color 1s linear; }\n\
         .glass:hover { background-color: var(--color-accent); }",
        &["swatch"],
    );
    let glass = scene.add(scene.root(), Role::Element, &["glass"]);
    scene.frame_at(Duration::ZERO);
    scene.tree.set_state(subject, States::HOVER, true);
    scene.tree.set_state(glass, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    scene.frame_at(milliseconds(500));

    let from = scene.light_color("color-surface");
    let to = scene.light_color("color-accent");
    let half_way = |from: u8, to: u8| f32::midpoint(f32::from(from), f32::from(to)).round() as u8;
    assert_eq!(
        background(&scene, subject),
        Color {
            red: half_way(from.red, to.red),
            green: half_way(from.green, to.green),
            blue: half_way(from.blue, to.blue),
            alpha: 1.0,
        }
    );
    assert_eq!(
        background(&scene, glass),
        Color { alpha: 0.5, ..to },
        "a transparent end fades the color in without darkening it"
    );
}

#[test]
fn a_border_of_current_color_follows_the_color_in_motion() {
    let (mut scene, subject) = subject_of(
        ".chip { width: 10px; height: 10px; border: 1px solid currentColor; color: var(--color-text); \
         transition: color 1s linear; }\n.chip:hover { color: var(--color-accent); }",
        &["chip"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    scene.frame_at(milliseconds(400));

    let style = scene.tree.style(subject);
    assert_ne!(style.paint.color, scene.light_color("color-text"));
    assert_ne!(style.paint.color, scene.light_color("color-accent"));
    assert_eq!(style.paint.border_color.left, style.paint.color);
}

#[test]
fn shadows_are_interpolated_layer_by_layer_from_transparent_layers() {
    let (mut scene, subject) = subject_of(
        ".card { width: 40px; height: 40px; box-shadow: none; transition: box-shadow 1s linear; }\n\
         .card:hover { box-shadow: var(--shadow-float); }",
        &["card"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    scene.frame_at(milliseconds(500));

    let layers = shadow_layers(&scene);
    assert_eq!(layers.len(), 2, "both layers of the float shadow are on their way");
    let blurs: Vec<f32> = layers.iter().map(|layer| layer.blur.0).collect();
    let offsets: Vec<f32> = layers.iter().map(|layer| layer.offset_y.0).collect();
    assert_eq!(blurs, [1.5, 12.0], "painted last layer first");
    assert_eq!(offsets, [0.5, 4.0]);
    assert!(layers.iter().all(|layer| layer.color.alpha < 0.08));
}

#[test]
fn transforms_of_one_primitive_interpolate_function_by_function() {
    let (mut scene, subject) = subject_of(
        ".tile { width: 40px; height: 40px; transition: transform 1s linear; }\n\
         .tile:hover { transform: translateX(100px) translateY(50%); }",
        &["tile"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);
    let at_rest = scene.border_box(subject);

    scene.frame_at(milliseconds(250));

    let moved = scene.tree.surface_box(subject);
    assert_near(moved.x - at_rest.x, 25.0, "the value at this time");
    assert_near(moved.y - at_rest.y, 5.0, "the value at this time");
}

#[test]
fn transforms_that_differ_interpolate_as_matrices() {
    let (mut scene, subject) = subject_of(
        ".tile { width: 40px; height: 40px; transform: rotate(0deg) scale(1); transition: transform 1s linear; }\n\
         .tile:hover { transform: scale(2) rotate(90deg); }",
        &["tile"],
    );
    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(Duration::ZERO);

    scene.frame_at(milliseconds(500));

    let half_turned = scene.tree.surface_box(subject);
    let side = 40.0 * 1.5 * (45.0_f32.to_radians().cos() + 45.0_f32.to_radians().sin());
    assert_near(
        half_turned.width,
        side,
        "a box half way to a quarter turn at half the growth",
    );
}

#[test]
fn a_transition_reversed_part_way_takes_as_long_back_as_it_went() {
    let (mut scene, subject) = opacity_transition("linear");
    scene.frame_at(milliseconds(300));

    scene.tree.set_state(subject, States::HOVER, false);
    scene.frame_at(milliseconds(300));
    scene.frame_at(milliseconds(450));

    assert_near(opacity(&scene, subject), 0.15, "half way back");
    scene.frame_at(milliseconds(600));
    assert_near(opacity(&scene, subject), 0.0, "back after as long as it went");
}

#[test]
fn a_property_that_is_not_transitioned_changes_at_once() {
    let (mut scene, subject) = subject_of(
        ".fader { width: 10px; height: 10px; opacity: 0; transition: background-color 1s; }\n\
         .fader:hover { opacity: 1; }",
        &["fader"],
    );

    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(milliseconds(10));

    assert_near(opacity(&scene, subject), 1.0, "the value at this time");
}

#[test]
fn the_first_style_of_an_element_does_not_transition() {
    let (mut scene, _) = subject_of(
        ".fader { width: 10px; height: 10px; opacity: 0.5; transition: opacity 1s; }",
        &[],
    );

    let fader = scene.add(scene.root(), Role::Element, &["fader"]);
    scene.frame_at(milliseconds(10));

    assert_near(opacity(&scene, fader), 0.5, "the value at this time");
}

#[test]
fn a_running_motion_asks_for_every_next_frame_and_stops_asking_once_it_ends() {
    let (mut scene, subject) = opacity_transition("linear");
    let requested_frames = hosted(&mut scene);
    requested_frames.set(0);

    scene.frame_at(milliseconds(500));
    assert_eq!(requested_frames.get(), 1, "the transition goes on");
    scene.frame_at(milliseconds(1000));
    assert_eq!(requested_frames.get(), 1, "the frame that ends it asks for no other");
    assert_near(opacity(&scene, subject), 1.0, "the value at this time");
}

const KEYFRAMES: &str = "@keyframes rise { from { opacity: 0.2; } 50% { opacity: 1; } to { opacity: 0.6; } }\n\
                         @keyframes widen { to { width: 100px; } }";

/// An element of the class `runner` animated by `animation`, started at time zero.
fn animated(animation: &str) -> (Scene, ElementId) {
    subject_of(
        &format!("{KEYFRAMES}\n.runner {{ width: 20px; height: 10px; opacity: 0.9; animation: {animation}; }}"),
        &["runner"],
    )
}

fn opacity_of_animation_at(animation: &str, time: Duration) -> f32 {
    let (mut scene, subject) = animated(animation);
    scene.frame_at(time);
    opacity(&scene, subject)
}

#[test]
fn keyframes_are_interpolated_between_the_two_around_the_progress() {
    assert_near(
        opacity_of_animation_at("rise 1s linear", Duration::ZERO),
        0.2,
        "the first keyframe",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear", milliseconds(250)),
        0.6,
        "half way to the middle",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear", milliseconds(750)),
        0.8,
        "half way to the end",
    );
}

#[test]
fn the_easing_of_an_animation_eases_every_interval_between_keyframes() {
    assert_near(
        opacity_of_animation_at("rise 1s cubic-bezier(0.25, 0.1, 0.25, 1)", milliseconds(250)),
        0.2 + 0.8 * EASE_AT_HALF_TIME,
        "ease at half of the first interval",
    );
}

#[test]
fn a_missing_end_keyframe_takes_the_value_of_the_style() {
    let (mut scene, subject) = animated("widen 1s linear");

    scene.frame_at(milliseconds(500));

    assert_near(scene.border_box(subject).width, 60.0, "the value at this time");
}

#[test]
fn a_delay_shows_the_style_unless_the_animation_fills_backwards() {
    assert_near(
        opacity_of_animation_at("rise 1s linear 500ms", milliseconds(200)),
        0.9,
        "no fill",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear 500ms backwards", milliseconds(200)),
        0.2,
        "the first keyframe fills the delay",
    );
}

#[test]
fn after_its_end_an_animation_shows_the_style_unless_it_fills_forwards() {
    assert_near(
        opacity_of_animation_at("rise 1s linear", milliseconds(1500)),
        0.9,
        "no fill",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear forwards", milliseconds(1500)),
        0.6,
        "the last keyframe stays",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear 1.5 forwards", milliseconds(3000)),
        1.0,
        "a fractional count stops part way and that point stays",
    );
}

#[test]
fn directions_and_iteration_counts_run_the_keyframes_back_and_forth() {
    assert_near(
        opacity_of_animation_at("rise 1s linear reverse", milliseconds(250)),
        0.8,
        "reverse",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear 2 alternate", milliseconds(1250)),
        0.8,
        "the second iteration of alternate runs backwards",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear 2 alternate-reverse", milliseconds(1250)),
        0.6,
        "the second iteration of alternate-reverse runs forwards",
    );
    assert_near(
        opacity_of_animation_at("rise 1s linear infinite", milliseconds(5250)),
        0.6,
        "the sixth iteration of an endless animation",
    );
}

#[test]
fn an_animation_that_ended_does_not_run_again_when_the_element_is_restyled() {
    let (mut scene, subject) = animated("rise 1s linear");
    scene.frame_at(milliseconds(1500));

    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(milliseconds(1600));

    assert_near(opacity(&scene, subject), 0.9, "the style, not a new run");
}

#[test]
fn a_finite_animation_asks_for_frames_until_its_end_and_an_endless_one_for_ever() {
    let (mut finite, _) = animated("rise 1s linear");
    let finite_frames = hosted(&mut finite);
    let (mut endless, _) = animated("rise 1s linear infinite");
    let endless_frames = hosted(&mut endless);
    finite_frames.set(0);
    endless_frames.set(0);

    finite.frame_at(milliseconds(1000));
    endless.frame_at(milliseconds(60_000));

    assert_eq!(finite_frames.get(), 0);
    assert_eq!(endless_frames.get(), 1);
}

#[test]
fn an_element_with_display_none_runs_no_motion() {
    let (mut scene, subject) = animated("rise 1s linear infinite");
    let requested_frames = hosted(&mut scene);
    scene
        .tree
        .set_inline(subject, morok_style::Declaration::Display(morok_style::Display::None));
    scene.frame_at(milliseconds(100));
    requested_frames.set(0);

    scene.frame_at(milliseconds(200));

    assert_eq!(requested_frames.get(), 0);
}

fn spring() -> Spring {
    Spring {
        stiffness: STIFFNESS,
        precision: PRECISION,
    }
}

#[test]
fn a_critically_damped_spring_follows_its_closed_form_without_overshooting() {
    let natural_frequency = STIFFNESS.sqrt();
    let mut previous = 100.0;
    for step in 1..=20 {
        let time = milliseconds(step * 25);
        let sample = spring().sample(100.0, 0.0, 0.0, time);
        let seconds = time.as_secs_f32();
        let expected = 100.0 * (1.0 + natural_frequency * seconds) * (-natural_frequency * seconds).exp();
        assert_near(sample.value, expected, "the closed form");
        assert!(
            sample.value >= 0.0 && sample.value < previous,
            "it closes in without passing the target"
        );
        previous = sample.value;
    }
}

#[test]
fn a_spring_comes_to_rest_at_its_target() {
    let sample = spring().sample(100.0, 0.0, -400.0, Duration::from_secs(3));

    assert_eq!(
        sample,
        SpringSample {
            value: 0.0,
            velocity: 0.0,
            opacity: 1.0,
            is_settled: true,
        }
    );
}

#[test]
fn a_running_spring_holds_the_frames_of_its_tree_until_it_settles() {
    let (mut scene, _) = subject_of(".idle { width: 10px; height: 10px; }", &["idle"]);
    let requested_frames = hosted(&mut scene);
    requested_frames.set(0);

    let id = scene.tree.start_spring(spring(), 0.0, 100.0, 0.0);
    assert_eq!(requested_frames.get(), 1, "starting a spring asks for a frame");
    scene.frame_at(milliseconds(1000));
    assert_near(
        scene.tree.spring_sample(id).value,
        0.0,
        "the spring starts with its first frame",
    );
    scene.frame_at(milliseconds(1100));
    assert!(scene.tree.spring_sample(id).value > 0.0);
    assert_eq!(
        requested_frames.get(),
        3,
        "every frame of a moving spring asks for the next"
    );
    scene.frame_at(milliseconds(4000));
    assert!(scene.tree.spring_sample(id).is_settled);
    assert_eq!(requested_frames.get(), 3, "a settled spring asks for nothing");
    scene.tree.stop_spring(id);
}

#[test]
fn under_reduced_motion_a_transition_ends_at_once() {
    let (mut scene, subject) = subject_of(
        ".fader { width: 10px; height: 10px; opacity: 0; transition: opacity 1s; }\n.fader:hover { opacity: 1; }",
        &["fader"],
    );
    scene.tree.set_reduced_motion(true);
    scene.frame_at(Duration::ZERO);

    scene.tree.set_state(subject, States::HOVER, true);
    scene.frame_at(milliseconds(10));

    assert_near(opacity(&scene, subject), 1.0, "the value at this time");
}

#[test]
fn reducing_motion_ends_the_transitions_that_run() {
    let (mut scene, subject) = opacity_transition("linear");
    scene.frame_at(milliseconds(200));

    scene.tree.set_reduced_motion(true);
    scene.frame_at(milliseconds(220));

    assert_near(opacity(&scene, subject), 1.0, "the value at this time");
}

#[test]
fn under_reduced_motion_a_spring_fades_in_at_its_target() {
    let (mut scene, _) = subject_of(".idle { width: 10px; height: 10px; }", &["idle"]);
    scene.tree.set_reduced_motion(true);
    let id = scene.tree.start_spring(spring(), 0.0, 100.0, 0.0);
    scene.frame_at(milliseconds(1000));

    let start = scene.tree.spring_sample(id);
    scene.frame_at(milliseconds(1075));
    let half_way = scene.tree.spring_sample(id);
    scene.frame_at(milliseconds(1000) + Spring::REDUCED_MOTION_FADE);
    let faded_in = scene.tree.spring_sample(id);

    assert_near(start.value, 100.0, "the value starts at its target");
    assert_near(start.opacity, 0.0, "the fade starts from nothing");
    assert_near(half_way.value, 100.0, "the value stays at its target");
    assert_near(half_way.opacity, 0.5, "half way through the fade");
    assert!(faded_in.is_settled);
    assert_near(faded_in.opacity, 1.0, "the fade is over");
}

#[test]
fn under_reduced_motion_a_finite_animation_runs() {
    let (mut scene, subject) = animated("rise 1s linear");
    scene.tree.set_reduced_motion(true);

    scene.frame_at(milliseconds(250));

    assert_near(opacity(&scene, subject), 0.6, "half way to the middle keyframe");
}

#[test]
fn under_reduced_motion_an_endless_animation_stops() {
    let (mut scene, endless) = animated("rise 1s linear infinite");
    let requested_frames = hosted(&mut scene);
    scene.tree.set_reduced_motion(true);
    scene.frame_at(milliseconds(250));
    requested_frames.set(0);

    scene.frame_at(milliseconds(500));

    assert_near(opacity(&scene, endless), 0.9, "the style without its endless animation");
    assert_eq!(requested_frames.get(), 0, "nothing runs on");
}
