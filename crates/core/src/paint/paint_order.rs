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
/// So is an element the frame shifts — a sticky element, a scroll thumb — so that it moves as one.
///
/// The content of a `Scroll` element paints inside its clip and moved by its scroll, its
/// positioned descendants included.
#[derive(Debug)]
pub(crate) struct PaintOrder<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) steps: &'a mut Vec<PaintStep>,
    pub(crate) damage: &'a mut Damage,
    /// Scratch space for the ancestors that clip or scroll a positioned element, innermost
    /// first, each with whether it clips and whether it scrolls.
    pub(crate) ancestor_layers: &'a mut Vec<(ElementId, bool, bool)>,
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
        let element = &self.elements[id];
        element.style.effects.creates_stacking_context() || element.is_shifted()
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
        self.positioned_descendants(id, id);
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
        self.begin_content(id);
        match self.elements[id].style.layout.display {
            Display::Block => {
                self.flow_boxes(id);
                self.flow_atomic_content(id);
                self.flow_outlines(id);
            }
            Display::Flex | Display::Grid => self.items(id),
            Display::None => unreachable!("an element without a box is not painted"),
        }
        self.end_content(id);
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
                self.begin_content(child);
                self.flow_boxes(child);
                self.end_content(child);
            }
        }
    }

    fn flow_atomic_content(&mut self, container: ElementId) {
        for position in 0..self.child_count(container) {
            let child = self.child(container, position);
            if !self.paints_in_flow(child) {
                continue;
            }
            self.begin_content(child);
            match self.elements[child].style.layout.display {
                Display::Block => self.flow_atomic_content(child),
                Display::Flex | Display::Grid => self.items(child),
                Display::None => unreachable!("an element without a box does not paint in flow"),
            }
            self.end_content(child);
        }
    }

    fn flow_outlines(&mut self, container: ElementId) {
        for position in 0..self.child_count(container) {
            let child = self.child(container, position);
            if !self.paints_in_flow(child) {
                continue;
            }
            if self.elements[child].style.layout.display == Display::Block {
                self.begin_content(child);
                self.flow_outlines(child);
                self.end_content(child);
            }
            self.steps.push(PaintStep::Outline(child));
        }
    }

    /// Visits every element below `id` in tree order: a positioned one paints here, under the
    /// clips of the ancestors that clip it and the scrolls of the `Scroll` elements around it
    /// inside `context`, the stacking context that paints it; a stacking context paints here
    /// with its own positioned descendants inside it; a hidden one leaves the order with its
    /// descendants.
    fn positioned_descendants(&mut self, id: ElementId, context: ElementId) {
        for position in 0..self.child_count(id) {
            let child = self.child(id, position);
            if !self.generates_box(child) {
                unpaint_subtree(self.elements, self.damage, child);
                continue;
            }
            if self.creates_stacking_context(child) {
                self.open_ancestors(child, context);
                self.stacking_context(child);
                self.close_ancestors(child, context);
                continue;
            }
            if self.elements[child].style.layout.positioning.is_positioned() {
                self.open_ancestors(child, context);
                self.atomic(child);
                self.close_ancestors(child, context);
            }
            self.positioned_descendants(child, context);
        }
    }

    /// Opens, outermost first, the clips of the ancestors of a positioned element and the
    /// scrolls of its `Scroll` ancestors inside `context`; the stacking context opened those
    /// outside it. An absolutely positioned element is clipped only from its containing block
    /// up: the ancestors between the two do not contain it.
    fn open_ancestors(&mut self, positioned: ElementId, context: ElementId) {
        self.collect_ancestors(positioned, context);
        for &(ancestor, clips, scrolls) in self.ancestor_layers.iter().rev() {
            if clips {
                self.steps.push(PaintStep::PushClip(ancestor));
            }
            if scrolls {
                self.steps.push(PaintStep::PushScroll(ancestor));
            }
        }
    }

    /// Closes, innermost first, what [`Self::open_ancestors`] opened for the element.
    fn close_ancestors(&mut self, positioned: ElementId, context: ElementId) {
        self.collect_ancestors(positioned, context);
        for &(_, clips, scrolls) in self.ancestor_layers.iter() {
            if scrolls {
                self.steps.push(PaintStep::PopScroll);
            }
            if clips {
                self.steps.push(PaintStep::PopClip);
            }
        }
    }

    /// The ancestors of a positioned element that clip it or scroll it, innermost first, each
    /// with whether it clips and whether it scrolls.
    fn collect_ancestors(&mut self, positioned: ElementId, context: ElementId) {
        let element = &self.elements[positioned];
        let containing_block = element.containing_block;
        let mut is_contained = element.style.layout.positioning != Positioning::Absolute;
        let mut is_inside_context = positioned != context;
        let mut ancestor = element.parent;
        self.ancestor_layers.clear();
        while let Some(current) = ancestor {
            if Some(current) == containing_block {
                is_contained = true;
            }
            let element = &self.elements[current];
            let clips = is_contained && element.style.layout.clips_overflow();
            let scrolls = is_inside_context && element.scroll.is_some();
            if clips || scrolls {
                self.ancestor_layers.push((current, clips, scrolls));
            }
            if current == context {
                is_inside_context = false;
            }
            ancestor = element.parent;
        }
    }

    /// Opens what the content of the element paints inside: its clip, and its scroll.
    fn begin_content(&mut self, id: ElementId) {
        if self.elements[id].style.layout.clips_overflow() {
            self.steps.push(PaintStep::PushClip(id));
        }
        if self.elements[id].scroll.is_some() {
            self.steps.push(PaintStep::PushScroll(id));
        }
    }

    fn end_content(&mut self, id: ElementId) {
        if self.elements[id].scroll.is_some() {
            self.steps.push(PaintStep::PopScroll);
        }
        if self.elements[id].style.layout.clips_overflow() {
            self.steps.push(PaintStep::PopClip);
        }
    }
}
