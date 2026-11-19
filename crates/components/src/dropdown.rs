use std::rc::Rc;

use morok_core::{ElementId, OverlayAlignment, OverlayPlacement, OverlaySide};
use morok_reactive::{Memo, Runtime, Signal, Source};
use morok_style::Declaration;

use crate::dropdown_step::DropdownStep;
use crate::length_in_pixels::pixels;
use crate::option_list::OptionList;
use crate::overlay_presence::show_overlay_while;
use crate::view::{Mount, Overlay, OverlayAnchor, OverlayInteraction, Property, Surfaces};

const PLACEMENT: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Below,
    alignment: OverlayAlignment::Start,
    gap: 4.0,
    shift: 0.0,
};

/// Takes the option the user chose from a dropdown list, by its place among the options.
pub(crate) type OptionChooser = Rc<dyn Fn(&mut Runtime<Surfaces>, usize)>;

/// The list of options a select or a combo box drops down under itself. The keyboard stays
/// with the control, whose keys move the highlight of the list; the list, in an overlay of
/// its own, takes the pointer alone.
#[derive(Clone)]
pub(crate) struct Dropdown {
    /// The names of the options of the control.
    pub(crate) labels: Rc<[String]>,
    /// The options the list shows now, by their places among the options, in the order of
    /// its rows.
    pub(crate) shown: Source<Vec<usize>>,
    /// The option the control holds now; its row is marked.
    pub(crate) chosen: Source<Option<usize>>,
    pub(crate) is_open: Signal<bool>,
    /// Whether the list is on screen: it is open and has a row to show.
    pub(crate) is_shown: Memo<bool>,
    /// The row the pointer or the arrow keys are on: the one Return chooses.
    pub(crate) highlighted: Signal<Option<usize>>,
    on_choose: OptionChooser,
}

impl Dropdown {
    /// A closed list of the options named `labels`.
    pub(crate) fn new(
        runtime: &mut Runtime<Surfaces>,
        labels: Rc<[String]>,
        shown: Source<Vec<usize>>,
        chosen: Source<Option<usize>>,
        on_choose: OptionChooser,
    ) -> Self {
        let is_open = runtime.signal(false);
        let is_shown =
            runtime.memo(move |runtime| runtime.get(is_open) && runtime.with(shown, |shown| !shown.is_empty()));
        Self {
            labels,
            shown,
            chosen,
            is_open,
            is_shown,
            highlighted: runtime.signal(None),
            on_choose,
        }
    }

    /// Opens the list with the highlight on `row`, or on no row.
    pub(crate) fn open(&self, runtime: &mut Runtime<Surfaces>, row: Option<usize>) {
        runtime.set(self.highlighted, row);
        runtime.set(self.is_open, true);
    }

    pub(crate) fn close(&self, runtime: &mut Runtime<Surfaces>) {
        if runtime.get(self.is_open) {
            runtime.set(self.is_open, false);
        }
    }

    /// The row that shows the option the control holds; none while the list does not show it.
    pub(crate) fn row_of_chosen(&self, runtime: &mut Runtime<Surfaces>) -> Option<usize> {
        let chosen = runtime.get(self.chosen)?;
        runtime.with(self.shown, |shown| shown.iter().position(|option| *option == chosen))
    }

    /// Moves the highlight by the step, up to the ends of the list; from no row it goes to
    /// the first row or to the last.
    pub(crate) fn move_highlight(&self, runtime: &mut Runtime<Surfaces>, step: DropdownStep) {
        let Some(last) = runtime.with(self.shown, Vec::len).checked_sub(1) else {
            return;
        };
        let to = match (step, runtime.get(self.highlighted)) {
            (DropdownStep::First, _) | (DropdownStep::Next, None) => 0,
            (DropdownStep::Last, _) | (DropdownStep::Previous, None) => last,
            (DropdownStep::Next, Some(row)) => (row + 1).min(last),
            (DropdownStep::Previous, Some(row)) => row.saturating_sub(1).min(last),
        };
        self.highlight(runtime, to);
    }

    /// Puts the highlight on the row.
    pub(crate) fn highlight(&self, runtime: &mut Runtime<Surfaces>, row: usize) {
        if runtime.get(self.highlighted) != Some(row) {
            runtime.set(self.highlighted, Some(row));
        }
    }

    /// Chooses the option of the highlighted row and closes the list. Returns whether a row
    /// was highlighted.
    pub(crate) fn choose_highlighted(&self, runtime: &mut Runtime<Surfaces>) -> bool {
        let highlighted = runtime.get(self.highlighted);
        if let Some(row) = highlighted {
            self.choose_row(runtime, row);
        }
        highlighted.is_some()
    }

    /// Chooses the option the row shows and closes the list.
    pub(crate) fn choose_row(&self, runtime: &mut Runtime<Surfaces>, row: usize) {
        let Some(option) = runtime.with(self.shown, |shown| shown.get(row).copied()) else {
            return;
        };
        (self.on_choose)(runtime, option);
        self.close(runtime);
    }

    /// Shows the list under `anchor`, an element of the surface of `mount`, while it is open
    /// and has a row to show: as wide as the anchor, and over the anchor where the screen
    /// ends. A press on the anchor is left to the anchor. `label` is what an assistive
    /// technology calls the list.
    pub(crate) fn attach(&self, mount: &mut Mount<'_>, anchor: ElementId, label: Property<String>) {
        let is_shown = self.is_shown;
        let width = mount.runtime().signal(0.0_f32);
        mount.effect(move |runtime, tree| {
            if runtime.get(is_shown) {
                runtime.set(width, tree.border_box(anchor).width);
            }
        });
        let (dismissed, list) = (self.clone(), self.clone());
        show_overlay_while(
            mount,
            anchor,
            is_shown,
            move |_| {
                let dismissed = dismissed.clone();
                Overlay {
                    anchor: OverlayAnchor::Element(anchor),
                    placement: PLACEMENT,
                    interaction: OverlayInteraction::PointerOnly {
                        on_dismiss: Box::new(move |runtime| dismissed.close(runtime)),
                    },
                }
            },
            move |mount, _| {
                let as_wide_as_anchor = mount
                    .runtime()
                    .memo(move |runtime| Declaration::Width(pixels(runtime.get(width))));
                let panel = list.mount_panel(mount, label.clone());
                mount.build(panel).inline(as_wide_as_anchor);
                panel
            },
        );
    }

    /// Builds the panel of the list on the surface of `mount` itself.
    pub(crate) fn mount_panel(&self, mount: &mut Mount<'_>, label: Property<String>) -> ElementId {
        OptionList {
            dropdown: self.clone(),
            label,
        }
        .mount(mount)
    }
}
