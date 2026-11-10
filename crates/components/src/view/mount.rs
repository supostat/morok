use morok_core::{ElementId, ElementTree, Role};
use morok_reactive::{Effect, Runtime, Scope};

use crate::view::element_builder::ElementBuilder;
use crate::view::property::Property;
use crate::view::surface::Surface;
use crate::view::surface_id::SurfaceId;
use crate::view::surfaces::Surfaces;

/// What a component is given when it is mounted: the signal graph and the surface it builds
/// its elements on. A component creates its signals, builds its elements and returns the
/// outermost of them; whoever mounted it puts that element into the tree.
#[derive(Debug)]
pub struct Mount<'a> {
    pub(crate) runtime: &'a mut Runtime<Surfaces>,
    pub(crate) surface: &'a mut Surface,
    pub(crate) surface_id: SurfaceId,
}

impl<'a> Mount<'a> {
    /// Runs `component` in a new scope that belongs to `owner` and returns that scope with
    /// the element the component built. What the component reads is tracked by nothing.
    pub(crate) fn in_scope(
        runtime: &mut Runtime<Surfaces>,
        surface: &mut Surface,
        surface_id: SurfaceId,
        owner: Scope,
        component: impl FnOnce(&mut Mount<'_>) -> ElementId,
    ) -> (Scope, ElementId) {
        let mut built = None;
        let scope = runtime.scope_in(owner, |runtime| {
            built = Some(runtime.untracked(|runtime| {
                component(&mut Mount {
                    runtime,
                    surface,
                    surface_id,
                })
            }));
        });
        (scope, built.expect("a scope runs what builds it"))
    }

    /// The signal graph of the application.
    pub fn runtime(&mut self) -> &mut Runtime<Surfaces> {
        self.runtime
    }

    /// The tree of the surface, for a component that drives elements itself.
    pub fn tree(&mut self) -> &mut ElementTree {
        &mut self.surface.tree
    }

    /// Creates a box that lays out children and builds on it.
    pub fn element(&mut self) -> ElementBuilder<'_, 'a> {
        let element = self.surface.tree.create(Role::Element);
        self.build(element)
    }

    /// Creates a run of text and builds on it.
    pub fn text(&mut self, text: impl Into<Property<String>>) -> ElementBuilder<'_, 'a> {
        let element = self.surface.tree.create(Role::Text);
        let mut builder = self.build(element);
        builder.bind(text.into(), |tree, element, text| tree.set_text(element, text));
        builder
    }

    /// Builds on the root of the surface: what a component gives the root holds for the whole
    /// surface.
    pub fn surface_root(&mut self) -> ElementBuilder<'_, 'a> {
        let root = self.surface.tree.root();
        self.build(root)
    }

    /// Builds on an element that exists already.
    pub fn build(&mut self, element: ElementId) -> ElementBuilder<'_, 'a> {
        ElementBuilder { mount: self, element }
    }

    /// An effect that is handed the tree of the surface itself: it reads signals and drives
    /// elements directly, with no property binding in between. It runs in the next flush and
    /// again after a value it read changed.
    pub fn effect(&mut self, mut run: impl FnMut(&mut Runtime<Surfaces>, &mut ElementTree) + 'static) -> Effect {
        let surface = self.surface_id;
        self.runtime.effect(move |runtime, surfaces: &mut Surfaces| {
            run(runtime, &mut surfaces.surface_mut(surface).tree);
        })
    }
}
