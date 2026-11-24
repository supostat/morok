mod support;

use std::alloc::{GlobalAlloc, Layout, System};
use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};

use morok_components::view::SurfaceId;
use morok_components::{Select, Tree, TreeItem};
use morok_core::{ElementId, Point};
use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use support::Stage;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

// SAFETY: every method hands the request to the system allocator unchanged and only counts it.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the layout is the caller's, under the contract of this method.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the layout is the caller's, under the contract of this method.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the pointer and the layout are those of an allocation this allocator made.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the pointer and the layout are those of an allocation this allocator made,
        // and the new size is the caller's.
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// How many items the tree holds, and how many options the select offers. Every label has
/// the same length, so a buffer that took one label takes any other.
const ROWS: usize = 1000;
/// Frames scrolled before the measurement: every row slot shows another row once, and the
/// buffers the frames fill reach their size.
const WARM_UP_FRAMES: usize = 40;
const MEASURED_FRAMES: usize = 900;
const TREE_ROW_HEIGHT: f32 = 28.0;
const OPTION_ROW_HEIGHT: f32 = 26.0;

/// Runs a frame of the surface the rows are on.
type FrameRunner = fn(&mut Stage, SurfaceId);

/// The rows of a view of the kit on a surface of the stage, which the wheel scrolls a row a
/// frame.
struct ScrolledRows {
    stage: Stage,
    surface: SurfaceId,
    viewport: ElementId,
    row_height: f32,
    run_frame: FrameRunner,
}

impl ScrolledRows {
    /// A tree of a thousand items, one of them selected, in a view of 200 by 140 pixels on the
    /// surface of the window.
    fn tree() -> Self {
        let mut stage = Stage::new();
        let view = stage.mount(|mount| {
            let roots = (0..ROWS)
                .map(|index| TreeItem {
                    key: format!("folder-{index:04}"),
                    label: format!("Folder {index:04}"),
                    children: Vec::new(),
                })
                .collect();
            let expanded = mount.runtime().signal(Vec::new());
            let selected = mount.runtime().signal(Some(String::from("folder-0003")));
            let view = Tree::new(roots, expanded, selected).label("Folders").mount(mount);
            mount
                .build(view)
                .inline(Declaration::Width(pixels(200.0)))
                .inline(Declaration::Height(pixels(140.0)));
            view
        });
        stage.frame();
        stage.frame();
        let viewport = stage.children(view)[0];
        let surface = stage.surface;
        Self {
            stage,
            surface,
            viewport,
            row_height: TREE_ROW_HEIGHT,
            run_frame: |stage, _| stage.frame(),
        }
    }

    /// A select of a thousand options whose list is open in an overlay over the window.
    fn open_select() -> Self {
        let mut stage = Stage::new();
        let select = stage.mount(|mount| {
            let chosen = mount.runtime().signal(0_usize);
            (0..ROWS)
                .fold(Select::new(chosen), |select, option| {
                    select.option(option, format!("Option {option:04}"))
                })
                .label("Options")
                .mount(mount)
        });
        stage.frame_all();
        stage.click(select);
        // The first frame of the list measures its viewport, the second scrolls it, and the
        // third binds the row slots to the rows that came into view.
        for _ in 0..3 {
            stage.frame_all();
        }
        let overlays = stage.overlays_of(stage.surface);
        assert_eq!(overlays.len(), 1, "the list of the select is open");
        let overlay = overlays[0];
        let viewport = list_viewport(&stage, overlay);
        Self {
            stage,
            surface: overlay,
            viewport,
            row_height: OPTION_ROW_HEIGHT,
            run_frame: Stage::frame_overlay,
        }
    }

    /// The wheel scrolls the rows down by a row, and a frame of their surface runs. Returns
    /// the heap allocations the two made.
    fn scroll_one_row(&mut self) -> u64 {
        let position = self.stage.middle_on(self.surface, self.viewport);
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        self.stage.wheel_on(
            self.surface,
            position,
            Point {
                x: 0.0,
                y: self.row_height,
            },
        );
        (self.run_frame)(&mut self.stage, self.surface);
        ALLOCATIONS.load(Ordering::Relaxed) - before
    }

    fn scroll_offset(&self) -> f64 {
        self.stage.tree_of(self.surface).scroll_offset(self.viewport).y
    }

    /// What the row slots show, slot by slot: the text a row holds last, however deep.
    fn shown_labels(&self) -> Vec<String> {
        let tree = self.stage.tree_of(self.surface);
        tree.children(tree.children(self.viewport)[0])
            .iter()
            .map(|slot| {
                let mut label = tree.children(*slot)[0];
                while let Some(last) = tree.children(label).last() {
                    label = *last;
                }
                tree.text(label).to_owned()
            })
            .collect()
    }
}

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

/// The viewport of the rows of a dropdown list: inside the view on the panel of its overlay.
fn list_viewport(stage: &Stage, overlay: SurfaceId) -> ElementId {
    let tree = stage.tree_of(overlay);
    let [frame] = tree.children(tree.root()) else {
        panic!("an overlay surface holds one box");
    };
    let [panel] = tree.children(*frame) else {
        panic!("the box of an overlay surface holds the panel");
    };
    let [view] = tree.children(*panel) else {
        panic!("the panel of a dropdown list holds the view of its rows");
    };
    tree.children(*view)[0]
}

/// Scrolls the rows through the warm-up and the measured frames, and checks that every
/// measured frame shows a first row in view that the frame before did not and binds a slot to
/// another row. Returns the allocations of the measured frames and the most one of them made.
fn measure(rows: &mut ScrolledRows) -> (u64, u64) {
    for _ in 0..WARM_UP_FRAMES {
        rows.scroll_one_row();
    }
    let (mut allocations, mut most) = (0, 0);
    for frame in 0..MEASURED_FRAMES {
        let offset_before = rows.scroll_offset();
        let labels_before = rows.shown_labels();
        let made = rows.scroll_one_row();
        assert!(
            rows.scroll_offset() - offset_before >= f64::from(rows.row_height),
            "frame {frame} scrolls the rows by a whole row"
        );
        assert_ne!(
            rows.shown_labels(),
            labels_before,
            "frame {frame} binds a slot to the row that came into view"
        );
        allocations += made;
        most = most.max(made);
    }
    (allocations, most)
}

/// The wheel scrolls a tree of a thousand items and the open list of a select of a thousand
/// options a row a frame: once warm, a frame that binds a row slot to another row allocates
/// nothing.
fn main() {
    for (view, mut rows) in [
        ("tree", ScrolledRows::tree()),
        ("select list", ScrolledRows::open_select()),
    ] {
        let (allocations, most) = measure(&mut rows);
        assert_eq!(
            allocations, 0,
            "{view}: {allocations} allocations in {MEASURED_FRAMES} frames, most {most} in one"
        );
        println!(
            "rebind_allocations: {view}: 0 allocations in {MEASURED_FRAMES} frames that each bind a slot to another row"
        );
    }
}
