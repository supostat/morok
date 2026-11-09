use morok_core::{ElementId, ElementTree, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::scene::Scene;
use crate::scene_tree::SceneTree;

const STYLESHEET_PATH: &str = "crates/bench/scenes/reference.css";
const STYLESHEET: &str = include_str!("../scenes/reference.css");

const BOXES: usize = 2000;
/// Every fourth box carries a text run.
const BOXES_PER_LABEL: usize = 4;
const LABELS: [&str; 8] = ["ok", "new", "old", "run", "idle", "hot", "cold", "sum"];
/// The share of the nodes of the scene that changes from one frame to the next.
const CHANGED_SHARE: f64 = 0.01;

/// The grid the boxes fall into: 50 boxes of 32 px with 4 px between them and around them.
const COLUMNS: usize = 50;
const BOX_WIDTH: f32 = 32.0;
const BOX_HEIGHT: f32 = 20.0;
const GAP: f32 = 4.0;

/// The scene the frame budgets are measured on: 2,000 styled boxes, 500 of them carrying a
/// text run, with one per cent of the nodes changing state every frame.
#[derive(Debug)]
pub struct ReferenceScene {
    boxes: Vec<ElementId>,
    changes_per_frame: usize,
    next_change: usize,
}

impl ReferenceScene {
    /// The scene with its boxes changing state frame after frame.
    pub fn changing() -> Self {
        let nodes = BOXES + BOXES / BOXES_PER_LABEL;
        Self::with_changes_per_frame((nodes as f64 * CHANGED_SHARE) as usize)
    }

    /// The scene standing still: nothing changes once it is built.
    pub fn still() -> Self {
        Self::with_changes_per_frame(0)
    }

    fn with_changes_per_frame(changes_per_frame: usize) -> Self {
        Self {
            boxes: Vec::with_capacity(BOXES),
            changes_per_frame,
            next_change: 0,
        }
    }
}

impl Scene for ReferenceScene {
    fn viewport(&self) -> Size {
        let rows = BOXES.div_ceil(COLUMNS);
        Size {
            width: COLUMNS as f32 * (BOX_WIDTH + GAP) + GAP,
            height: rows as f32 * (BOX_HEIGHT + GAP) + GAP,
        }
    }

    fn stylesheet(&self) -> SourceFile {
        SourceFile::new(STYLESHEET_PATH, STYLESHEET)
    }

    fn build(&mut self, tree: &mut SceneTree) {
        let root = tree.root();
        tree.add_classes(root, &["scene"]);
        for position in 0..BOXES {
            let element = tree.add(root, &["box"]);
            if position % BOXES_PER_LABEL == 0 {
                tree.add_text(element, &["label"], LABELS[position / BOXES_PER_LABEL % LABELS.len()]);
            }
            self.boxes.push(element);
        }
    }

    /// Flips the hover state of the next boxes in order, round after round.
    fn change(&mut self, _frame: usize, tree: &mut ElementTree) {
        for _ in 0..self.changes_per_frame {
            let element = self.boxes[self.next_change];
            let is_hovered = tree.states(element).contains(States::HOVER);
            tree.set_state(element, States::HOVER, !is_hovered);
            self.next_change = (self.next_change + 1) % self.boxes.len();
        }
    }
}
