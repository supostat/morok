use std::mem;

use morok_style::Display;

use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::motion::motion_resolver::MotionResolver;
use crate::style::computed_style::ComputedStyle;
use crate::style::style_outcome::StyleOutcome;
use crate::style::style_resolver::StyleResolver;
use crate::style::style_scratch::StyleScratch;
use crate::style::styled_parent::StyledParent;

/// The style stage of a frame: resolves the style of every element marked for it, parents first,
/// and works out what every element in motion shows at the time of the frame.
#[derive(Debug)]
pub(crate) struct StylePass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) resolver: StyleResolver<'a>,
    pub(crate) motion: MotionResolver<'a>,
    pub(crate) scratch: &'a mut StyleScratch,
    pub(crate) pending: &'a mut Vec<ElementId>,
    /// Receives the elements whose motion goes on after this frame.
    pub(crate) moving: &'a mut Vec<ElementId>,
}

impl StylePass<'_> {
    pub(crate) fn run(mut self, root: ElementId) -> StyleOutcome {
        let mut outcome = StyleOutcome::default();
        self.pending.clear();
        self.pending.push(root);
        while let Some(id) = self.pending.pop() {
            let dirty = self.elements[id].dirty;
            if dirty.contains(Dirty::STYLE) {
                self.restyle(id, &mut outcome);
            } else if dirty.contains(Dirty::MOTION) {
                self.animate(id, &mut outcome);
            }
            let element = &mut self.elements[id];
            let has_marked_descendants = element.dirty.contains(Dirty::STYLE_BELOW);
            element
                .dirty
                .remove(Dirty::STYLE.union(Dirty::STYLE_BELOW).union(Dirty::MOTION));
            if has_marked_descendants {
                let marked = Dirty::STYLE.union(Dirty::STYLE_BELOW).union(Dirty::MOTION);
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

    /// Cascades the style of the element again and lets its motion follow the change.
    fn restyle(&mut self, id: ElementId, outcome: &mut StyleOutcome) {
        let element = &self.elements[id];
        let parent = element.parent.map(|parent| {
            let parent = &self.elements[parent];
            StyledParent {
                target: parent.selector_target(),
                color: parent.style.paint.color,
                text: parent.style.text,
                is_rendered: parent.is_rendered,
            }
        });
        let parent_is_rendered = parent.is_none_or(|parent| parent.is_rendered);
        let base = self.resolver.resolve(
            element.selector_target(),
            parent,
            &element.inline_style,
            &mut self.scratch.matched_rules,
            &mut self.scratch.base_lists,
        );
        outcome.resolved_elements += 1;
        let is_rendered = parent_is_rendered && base.layout.display != Display::None;
        let element = &mut self.elements[id];
        self.motion.restart(
            element,
            &base,
            &self.scratch.base_lists,
            is_rendered,
            &mut self.scratch.motion,
        );
        let was_rendered = mem::replace(&mut element.is_rendered, is_rendered);
        self.show(id, &base, outcome);
        if was_rendered != is_rendered {
            self.mark_children(id);
        }
    }

    /// Works out what the element shows at the time of the frame from the style its motion runs on.
    fn animate(&mut self, id: ElementId, outcome: &mut StyleOutcome) {
        let Some(motion) = &self.elements[id].motion else {
            return;
        };
        let base = motion.base;
        self.scratch.base_lists.copy_from(&motion.base_lists);
        self.show(id, &base, outcome);
    }

    /// Makes the element show `base`, with the lists in the scratch space, under its motion, and
    /// marks what the change touches.
    fn show(&mut self, id: ElementId, base: &ComputedStyle, outcome: &mut StyleOutcome) {
        let element = &mut self.elements[id];
        let mut style = *base;
        let is_moving = self.motion.show(
            element.motion.as_deref_mut(),
            base,
            &self.scratch.base_lists,
            &mut style,
            &mut self.scratch.shown_lists,
            &mut self.scratch.motion,
        );
        if is_moving {
            self.moving.push(id);
        }
        let lists_changed = element.lists != self.scratch.shown_lists;
        if lists_changed {
            element.lists.copy_from(&self.scratch.shown_lists);
        }
        let previous = mem::replace(&mut element.style, style);
        if previous.layout != style.layout {
            self.elements.invalidate_layout(id);
        }
        if changes_paint_order(&previous, &style) {
            outcome.reorders_paint = true;
        }
        if lists_changed || paints_differently(&previous, &style) {
            self.elements[id].dirty.insert(Dirty::PAINT);
            outcome.repaints = true;
        }
        if previous.text != style.text && self.elements[id].role.sets_lines() {
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

    /// Children take inherited values and whether they are rendered from the element, so they
    /// are resolved again after it.
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
