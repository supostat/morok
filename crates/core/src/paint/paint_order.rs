use morok_style::Display;

use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::paint::damage::Damage;
use crate::paint::paint_step::PaintStep;
use crate::paint::unpaint::unpaint_subtree;
use crate::style::positioning::Positioning;

/// Works out the order in which the parts of elements are painted, as CSS defines it.
///
/// An element paints atomically — its box, its content, its outline — when it is the root, an
/// item of a flex or grid container, or positioned. The children of a block container paint in
/// phases instead: every box of the container's flow first, then the content that paints
/// atomically, then every outline, so the outline of a block lies over the background of the
/// block after it. Positioned elements paint last, in tree order, over everything in the flow.
///
/// An element with a transform or a compositing effect is a stacking context: it paints among
/// the positioned elements as a group, and its own positioned descendants paint inside the group.
#[derive(Debug)]
pub(crate) struct PaintOrder<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) steps: &'a mut Vec<PaintStep>,
    pub(crate) damage: &'a mut Damage,
    /// Scratch space for the clipping ancestors of a positioned element.
    pub(crate) clipping_ancestors: &'a mut Vec<ElementId>,
}

impl PaintOrder<'_> {
    /// Replaces the steps with the order of the tree as it is now. An element that leaves the
    /// order, hidden by `display: none`, damages the area it drew.
    pub(crate) fn rebuild(mut self, root: ElementId) {
        self.steps.clear();
        if !self.generates_box(root) {
            unpaint_subtree(self.elements, self.damage, root);
            return;
        }
        self.stacking_context(root);
    }

    fn generates_box(&self, id: ElementId) -> bool {
        self.elements[id].style.layout.generates_box()
    }

    fn creates_stacking_context(&self, id: ElementId) -> bool {
        self.elements[id].style.effects.creates_stacking_context()
    }

    /// Whether the element paints with the flow of its container rather than as a positioned
    /// element or a stacking context.
    fn paints_in_flow(&self, id: ElementId) -> bool {
        let style = &self.elements[id].style.layout;
        style.generates_box() && !style.positioning.is_positioned() && !self.creates_stacking_context(id)
    }

    /// The element with its content and, after them, its positioned descendants; grouped when
    /// the element has effects, so that they move and composite as one.
    fn stacking_context(&mut self, id: ElementId) {
        let is_grouped = self.creates_stacking_context(id);
        if is_grouped {
            self.steps.push(PaintStep::PushGroup(id));
        }
        self.atomic(id);
        self.positioned_descendants(id);
        if is_grouped {
            self.steps.push(PaintStep::PopGroup);
        }
    }

    fn child(&self, id: ElementId, position: usize) -> ElementId {
        self.elements[id].children[position]
    }

    fn child_count(&self, id: ElementId) -> usize {
        self.elements[id].children.len()
    }

    fn atomic(&mut self, id: ElementId) {
        self.steps.push(PaintStep::Box(id));
        self.begin_clip(id);
        match self.elements[id].style.layout.display {
            Display::Block => {
                self.flow_boxes(id);
                self.flow_atomic_content(id);
                self.flow_outlines(id);
            }
            Display::Flex | Display::Grid => self.items(id),
            Display::None => unreachable!("an element without a box is not painted"),
        }
        self.end_clip(id);
        self.steps.push(PaintStep::Outline(id));
    }

    /// The children of a flex or grid container: each paints atomically, in order.
    fn items(&mut self, container: ElementId) {
        for position in 0..self.child_count(container) {
            let item = self.child(container, position);
            if self.paints_in_flow(item) {
                self.atomic(item);
            }
        }
    }

    fn flow_boxes(&mut self, container: ElementId) {
        for position in 0..self.child_count(container) {
            let child = self.child(container, position);
            if !self.paints_in_flow(child) {
                continue;
            }
            self.steps.push(PaintStep::Box(child));
            if self.elements[child].style.layout.display == Display::Block {
                self.begin_clip(child);
                self.flow_boxes(child);
                self.end_clip(child);
            }
        }
    }

    fn flow_atomic_content(&mut self, container: ElementId) {
        for position in 0..self.child_count(container) {
            let child = self.child(container, position);
            if !self.paints_in_flow(child) {
                continue;
            }
            self.begin_clip(child);
            match self.elements[child].style.layout.display {
                Display::Block => self.flow_atomic_content(child),
                Display::Flex | Display::Grid => self.items(child),
                Display::None => unreachable!("an element without a box does not paint in flow"),
            }
            self.end_clip(child);
        }
    }

    fn flow_outlines(&mut self, container: ElementId) {
        for position in 0..self.child_count(container) {
            let child = self.child(container, position);
            if !self.paints_in_flow(child) {
                continue;
            }
            if self.elements[child].style.layout.display == Display::Block {
                self.begin_clip(child);
                self.flow_outlines(child);
                self.end_clip(child);
            }
            self.steps.push(PaintStep::Outline(child));
        }
    }

    /// Visits every element below `id` in tree order: a positioned one paints here, under the
    /// clips of the ancestors that clip it; a stacking context paints here with its own
    /// positioned descendants inside it; a hidden one leaves the order with its descendants.
    fn positioned_descendants(&mut self, id: ElementId) {
        for position in 0..self.child_count(id) {
            let child = self.child(id, position);
            if !self.generates_box(child) {
                unpaint_subtree(self.elements, self.damage, child);
                continue;
            }
            if self.creates_stacking_context(child) {
                let clips = self.push_ancestor_clips(child);
                self.stacking_context(child);
                for _ in 0..clips {
                    self.steps.push(PaintStep::PopClip);
                }
                continue;
            }
            if self.elements[child].style.layout.positioning.is_positioned() {
                let clips = self.push_ancestor_clips(child);
                self.atomic(child);
                for _ in 0..clips {
                    self.steps.push(PaintStep::PopClip);
                }
            }
            self.positioned_descendants(child);
        }
    }

    /// Opens the clips of the ancestors of a positioned element, outermost first, and returns
    /// their number. An absolutely positioned element is clipped only from its containing block
    /// up: the ancestors between the two do not contain it.
    fn push_ancestor_clips(&mut self, positioned: ElementId) -> usize {
        let element = &self.elements[positioned];
        let containing_block = element.containing_block;
        let mut is_contained = element.style.layout.positioning != Positioning::Absolute;
        let mut ancestor = element.parent;
        self.clipping_ancestors.clear();
        while let Some(current) = ancestor {
            if Some(current) == containing_block {
                is_contained = true;
            }
            let element = &self.elements[current];
            if is_contained && element.style.layout.clips_overflow() {
                self.clipping_ancestors.push(current);
            }
            ancestor = element.parent;
        }
        self.steps
            .extend(self.clipping_ancestors.iter().rev().copied().map(PaintStep::PushClip));
        self.clipping_ancestors.len()
    }

    fn begin_clip(&mut self, id: ElementId) {
        if self.elements[id].style.layout.clips_overflow() {
            self.steps.push(PaintStep::PushClip(id));
        }
    }

    fn end_clip(&mut self, id: ElementId) {
        if self.elements[id].style.layout.clips_overflow() {
            self.steps.push(PaintStep::PopClip);
        }
    }
}
