use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, ElementTree, OverlaySide, Point};

use crate::panel_content::PanelContent;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// What an open popover shows: its arrow, then its title, its message and its content, each
/// when the popover has one.
#[derive(Clone)]
pub(crate) struct PopoverPanel {
    /// What an assistive technology calls the popover.
    pub(crate) label: Property<String>,
    pub(crate) title: Option<Property<String>>,
    pub(crate) message: Option<Property<String>>,
    pub(crate) content: Option<PanelContent>,
}

impl PopoverPanel {
    /// Builds the panel with its arrow on the edge that faces the anchor: the top edge for a
    /// popover below its anchor, the bottom edge for one above it. Where an edge of the screen
    /// moved the panel along the anchor by `edge_shift`, the arrow moves back against it.
    pub(crate) fn mount(
        &self,
        mount: &mut Mount<'_>,
        side: &Property<OverlaySide>,
        edge_shift: &Property<f32>,
    ) -> ElementId {
        let is_above = match *side {
            Property::Fixed(side) => Property::Fixed(side == OverlaySide::Above),
            Property::Bound(side) => mount
                .runtime()
                .memo(move |runtime| runtime.get(side) == OverlaySide::Above)
                .into(),
        };
        let arrow = mount
            .element()
            .class(CLASSES.popover_arrow)
            .class_when(CLASSES.above, is_above)
            .bind(edge_shift.clone(), |tree, arrow, edge_shift| {
                point_back_at_anchor(tree, arrow, *edge_shift);
            })
            .id();
        let title = self
            .title
            .clone()
            .map(|title| mount.text(title).class(CLASSES.popover_title).id());
        let message = self
            .message
            .clone()
            .map(|message| mount.text(message).class(CLASSES.popover_message).id());
        let content = self.content.as_ref().map(|content| content(mount));
        let mut panel = mount.element();
        panel
            .class(CLASSES.popover)
            .accessible_role(AccessibleRole::Dialog)
            .accessible_name(self.label.clone())
            .child(arrow);
        for part in title.into_iter().chain(message).chain(content) {
            panel.child(part);
        }
        panel.id()
    }
}

/// Moves the arrow against `edge_shift` from where its stylesheet puts it, so that it points
/// at the anchor again, as far as the straight part of the edge of the panel reaches: the
/// arrow stays clear of the rounded corners. The panel was laid out before the host placed it.
fn point_back_at_anchor(tree: &mut ElementTree, arrow: ElementId, edge_shift: f32) {
    let moved_by = if edge_shift == 0.0 {
        0.0
    } else {
        let panel = tree.parent(arrow).expect("the arrow lies in its panel");
        let (panel_box, arrow_box) = (tree.border_box(panel), tree.border_box(arrow));
        let corners = tree.style(panel).paint.border_radius;
        let first = panel_box.x + corners.top_left.max(corners.bottom_left);
        let last = panel_box.right() - corners.top_right.max(corners.bottom_right) - arrow_box.width;
        (arrow_box.x - edge_shift).min(last).max(first) - arrow_box.x
    };
    if moved_by != 0.0 || tree.displacement(arrow).is_some() {
        tree.set_displacement(arrow, Point { x: moved_by, y: 0.0 });
    }
}
