use std::mem;

use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::role::Role;
use crate::style::computed_style::ComputedStyle;
use crate::style::style_outcome::StyleOutcome;
use crate::style::style_resolver::StyleResolver;
use crate::style::styled_parent::StyledParent;

/// The style stage of a frame: resolves the style of every element marked for it, parents first.
#[derive(Debug)]
pub(crate) struct StylePass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) resolver: StyleResolver<'a>,
    pub(crate) matched_rules: &'a mut Vec<u32>,
    pub(crate) pending: &'a mut Vec<ElementId>,
}

impl StylePass<'_> {
    pub(crate) fn run(mut self, root: ElementId) -> StyleOutcome {
        let mut outcome = StyleOutcome::default();
        self.pending.clear();
        self.pending.push(root);
        while let Some(id) = self.pending.pop() {
            if self.elements[id].dirty.contains(Dirty::STYLE) {
                self.restyle(id, &mut outcome);
            }
            let element = &mut self.elements[id];
            let has_marked_descendants = element.dirty.contains(Dirty::STYLE_BELOW);
            element.dirty.remove(Dirty::STYLE.union(Dirty::STYLE_BELOW));
            if has_marked_descendants {
                let marked = Dirty::STYLE.union(Dirty::STYLE_BELOW);
                let elements = &*self.elements;
                self.pending.extend(
                    elements[id]
                        .children
                        .iter()
                        .copied()
                        .filter(|child| elements[*child].dirty.intersects(marked)),
                );
            }
        }
        outcome
    }

    fn restyle(&mut self, id: ElementId, outcome: &mut StyleOutcome) {
        let element = &self.elements[id];
        let parent = element.parent.map(|parent| {
            let parent = &self.elements[parent];
            StyledParent {
                target: parent.selector_target(),
                color: parent.style.paint.color,
                text: parent.style.text,
            }
        });
        let style = self.resolver.resolve(
            element.selector_target(),
            parent,
            &element.inline_style,
            self.matched_rules,
        );
        let previous = mem::replace(&mut self.elements[id].style, style);
        outcome.resolved_elements += 1;
        if previous.layout != style.layout {
            self.elements.invalidate_layout(id);
        }
        if changes_paint_order(&previous, &style) {
            outcome.reorders_paint = true;
        }
        if paints_differently(&previous, &style) {
            self.elements[id].dirty.insert(Dirty::PAINT);
            outcome.repaints = true;
        }
        if previous.text != style.text && self.elements[id].role == Role::Text {
            self.elements.invalidate_layout(id);
            self.elements.invalidate_text(id);
        }
        if previous.paint.color != style.paint.color || previous.text.inherited() != style.text.inherited() {
            self.mark_children(id);
        }
        if cuts_text_differently(&previous, &style) {
            self.invalidate_text_of_children(id);
        }
    }

    fn invalidate_text_of_children(&mut self, id: ElementId) {
        for position in 0..self.elements[id].children.len() {
            let child = self.elements[id].children[position];
            self.elements.invalidate_text(child);
        }
    }

    /// Children take inherited values from the element, so they are resolved again after it.
    fn mark_children(&mut self, id: ElementId) {
        for position in 0..self.elements[id].children.len() {
            let child = self.elements[id].children[position];
            self.elements[child].dirty.insert(Dirty::STYLE);
        }
        self.elements[id].dirty.insert(Dirty::STYLE_BELOW);
    }
}

/// Whether the text children of the element are cut with an ellipsis under one style and not
/// under the other.
fn cuts_text_differently(previous: &ComputedStyle, style: &ComputedStyle) -> bool {
    previous.text.text_overflow != style.text.text_overflow || previous.layout.overflow_x != style.layout.overflow_x
}

fn changes_paint_order(previous: &ComputedStyle, style: &ComputedStyle) -> bool {
    previous.layout.display != style.layout.display
        || previous.layout.positioning != style.layout.positioning
        || previous.layout.clips_overflow() != style.layout.clips_overflow()
        || previous.effects.creates_stacking_context() != style.effects.creates_stacking_context()
}

/// Whether the element draws something else in the same box. A box that moves or resizes is
/// found after layout.
fn paints_differently(previous: &ComputedStyle, style: &ComputedStyle) -> bool {
    previous.paint != style.paint
        || previous.effects != style.effects
        || previous.layout.used_border_width() != style.layout.used_border_width()
        || previous.layout.clips_overflow() != style.layout.clips_overflow()
        || previous.layout.generates_box() != style.layout.generates_box()
}
