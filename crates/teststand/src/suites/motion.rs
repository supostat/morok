use std::time::Duration;

use morok_core::Size;
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;

pub(crate) const NAME: &str = "motion";

const STYLESHEET_PATH: &str = "crates/teststand/suites/motion.css";
const STYLESHEET: &str = include_str!("../../suites/motion.css");

/// Transitions and keyframe animations, each drawn early and late in its run. No time falls on
/// a jump of a step easing or on the end of an iteration.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            transitions("transitions-early", 130),
            transitions("transitions-late", 310),
            keyframes("keyframes-early", 150),
            keyframes("keyframes-late", 1000),
        ],
    }
}

fn transitions(name: &'static str, milliseconds: u64) -> Case {
    Case {
        name,
        viewport: Size {
            width: 640.0,
            height: 152.0,
        },
        build: movers,
        moment: Moment::InMotion {
            change: hover_movers,
            after: Duration::from_millis(milliseconds),
        },
    }
}

fn keyframes(name: &'static str, milliseconds: u64) -> Case {
    Case {
        name,
        viewport: Size {
            width: 520.0,
            height: 152.0,
        },
        build: runners,
        moment: Moment::InMotion {
            change: |_| {},
            after: Duration::from_millis(milliseconds),
        },
    }
}

fn row(scene: &mut Scene) -> morok_core::ElementId {
    let stage = scene.add(scene.root(), &["stage"]);
    scene.add(stage, &["row"])
}

fn movers(scene: &mut Scene) {
    let row = row(scene);
    for kind in ["tint", "fade", "slide", "widen", "lift", "round", "spin", "wait"] {
        scene.add(row, &["mover", kind]);
    }
}

fn hover_movers(scene: &mut Scene) {
    for mover in scene.elements_with_class("mover") {
        scene.set_state(mover, States::HOVER);
    }
}

fn runners(scene: &mut Scene) {
    let row = row(scene);
    for kind in ["pulse", "rise", "grow", "tint", "march", "glow"] {
        scene.add(row, &["runner", kind]);
    }
}
