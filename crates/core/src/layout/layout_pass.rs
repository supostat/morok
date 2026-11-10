use morok_style::{Display, StyleSheet, Theme};
use taffy::{
    AvailableSpace, BlockContext, CacheTree, DetailedGridInfo, DetailedLayoutInfo, Layout, LayoutBlockContainer,
    LayoutContainingBlock, LayoutFlexboxContainer, LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree,
    NodeId, RunMode, TraversePartialTree, compute_block_layout, compute_cached_layout, compute_flexbox_layout,
    compute_grid_layout, compute_hidden_layout, compute_leaf_layout, compute_oof_layout, compute_root_layout,
};

use crate::dirty::Dirty;
use crate::editing::line_mode::LineMode;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::size::Size;
use crate::layout::child_nodes::ChildNodes;
use crate::layout::element_styles::ElementStyles;
use crate::layout::taffy_style::TaffyStyle;
use crate::media::media_content::MediaContent;
use crate::role::Role;
use crate::text::line_metrics::LineMetrics;
use crate::text::text_request::TextRequest;
use crate::text::text_shaper::TextShaper;
use crate::text::white_space_processing::processed;
use crate::text::width_constraint::WidthConstraint;

/// The layout stage of a frame: the element arena laid out by the algorithms of Taffy.
///
/// Every element caches the boxes it was given, so only elements whose cache was dropped — and
/// those whose constraints changed with them — are laid out again.
pub(crate) struct LayoutPass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) theme: &'a Theme,
    pub(crate) text_shaper: &'a mut dyn TextShaper,
    /// Device pixels to a CSS pixel on the surface.
    pub(crate) scale: f32,
    /// Scratch space for texts whose white space collapses.
    pub(crate) processed_text: &'a mut String,
    pub(crate) laid_out_elements: usize,
}

impl LayoutPass<'_> {
    /// Lays the tree out inside the viewport and returns the number of elements whose box was computed.
    pub(crate) fn run(mut self, root: ElementId, viewport: Size) -> usize {
        let available_space = taffy::Size {
            width: AvailableSpace::Definite(viewport.width),
            height: AvailableSpace::Definite(viewport.height),
        };
        compute_root_layout(&mut self, NodeId::from(root), available_space);
        self.laid_out_elements
    }

    fn styles(&self) -> ElementStyles<'_> {
        ElementStyles {
            elements: self.elements,
            style_sheet: self.style_sheet,
            theme: self.theme,
        }
    }

    fn style(&self, node: NodeId) -> TaffyStyle<'_> {
        self.styles().taffy_style(ElementId::from(node))
    }

    fn compute_element_layout(
        &mut self,
        node: NodeId,
        inputs: LayoutInput,
        block_context: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return compute_hidden_layout(self, node);
        }
        compute_cached_layout(self, node, inputs, |pass, node, inputs| {
            let id = ElementId::from(node);
            let element = &mut pass.elements[id];
            let display = element.style.layout.display;
            let lays_out_children = element.role.is_container() && !element.children.is_empty();
            if display != Display::Grid {
                element.grid_info = DetailedLayoutInfo::None;
            }
            if inputs.run_mode == RunMode::PerformLayout {
                pass.laid_out_elements += 1;
            }
            let mut output = match (display, lays_out_children) {
                (Display::None, _) => compute_hidden_layout(pass, node),
                (Display::Block, true) => compute_block_layout(pass, node, inputs, block_context),
                (Display::Flex, true) => compute_flexbox_layout(pass, node, inputs),
                (Display::Grid, true) => compute_grid_layout(pass, node, inputs),
                (_, false) => pass.compute_leaf_layout(id, inputs),
            };
            if inputs.run_mode == RunMode::PerformLayout {
                compute_oof_layout(pass, node, &mut output);
            }
            output
        })
    }

    /// The layout of an element without children: a text is as wide as its widest line and as
    /// high as its lines at the line height of CSS, every other element has no content of its
    /// own. A text that does not wrap is measured on one line whatever width it is offered.
    /// An edited text scrolls inside its box, so it asks for no width; it is as high as its
    /// one line, or as the lines it wraps into at the width it is given.
    fn compute_leaf_layout(&mut self, id: ElementId, inputs: LayoutInput) -> LayoutOutput {
        let styles = ElementStyles {
            elements: self.elements,
            style_sheet: self.style_sheet,
            theme: self.theme,
        };
        let element = &styles.elements[id];
        let text_shaper = &mut *self.text_shaper;
        let processed_text = &mut *self.processed_text;
        let theme = self.theme;
        let scale = self.scale;
        compute_leaf_layout(
            inputs,
            &styles.taffy_style(id),
            |handle, basis| styles.calc_value(handle, basis),
            |known_dimensions, available_space| {
                if let MediaContent::Image {
                    natural_size: Some(natural),
                    ..
                } = element.media
                {
                    return taffy::Size {
                        width: known_dimensions.width.unwrap_or(natural.width),
                        height: known_dimensions.height.unwrap_or(natural.height),
                    };
                }
                let style = &element.style.text;
                let font = style.font_request(theme);
                if let Some(editable) = element.editable.as_deref() {
                    let wrap_width = match (known_dimensions.width, available_space.width) {
                        (Some(width), _) | (None, AvailableSpace::Definite(width)) => WidthConstraint::Definite(width),
                        (None, AvailableSpace::MinContent | AvailableSpace::MaxContent) => WidthConstraint::MaxContent,
                    };
                    let line_count = match editable.line_mode {
                        LineMode::Single => 1,
                        LineMode::Multiple => editable
                            .editor
                            .text()
                            .split('\n')
                            .map(|paragraph| {
                                let extent = text_shaper.measure(&TextRequest {
                                    text: paragraph,
                                    font,
                                    letter_spacing: style.letter_spacing,
                                    width: wrap_width,
                                });
                                extent.line_count.max(1)
                            })
                            .sum(),
                    };
                    let line = LineMetrics::of(style, &text_shaper.font_metrics(&font), scale);
                    return taffy::Size {
                        width: known_dimensions.width.unwrap_or(0.0),
                        height: known_dimensions.height.unwrap_or(line_count as f32 * line.line_height),
                    };
                }
                if element.role != Role::Text {
                    return taffy::Size::ZERO;
                }
                let offered_width = match (known_dimensions.width, available_space.width) {
                    (Some(width), _) | (None, AvailableSpace::Definite(width)) => WidthConstraint::Definite(width),
                    (None, AvailableSpace::MinContent) => WidthConstraint::MinContent,
                    (None, AvailableSpace::MaxContent) => WidthConstraint::MaxContent,
                };
                let extent = text_shaper.measure(&TextRequest {
                    text: processed(&element.text, style.white_space, processed_text),
                    font,
                    letter_spacing: style.letter_spacing,
                    width: if style.white_space.wraps() {
                        offered_width
                    } else {
                        WidthConstraint::MaxContent
                    },
                });
                let line = LineMetrics::of(style, &text_shaper.font_metrics(&font), scale);
                taffy::Size {
                    width: known_dimensions.width.unwrap_or(extent.width),
                    height: known_dimensions
                        .height
                        .unwrap_or(extent.line_count as f32 * line.line_height),
                }
            },
        )
    }
}

impl TraversePartialTree for LayoutPass<'_> {
    type ChildIter<'a>
        = ChildNodes<'a>
    where
        Self: 'a;

    fn child_ids(&self, parent: NodeId) -> Self::ChildIter<'_> {
        ChildNodes(self.elements[ElementId::from(parent)].children.iter())
    }

    fn child_count(&self, parent: NodeId) -> usize {
        self.elements[ElementId::from(parent)].children.len()
    }

    fn get_child_id(&self, parent: NodeId, position: usize) -> NodeId {
        NodeId::from(self.elements[ElementId::from(parent)].children[position])
    }
}

impl LayoutPartialTree for LayoutPass<'_> {
    type CoreContainerStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    type CustomIdent = String;

    fn get_core_container_style(&self, node: NodeId) -> Self::CoreContainerStyle<'_> {
        self.style(node)
    }

    fn resolve_calc_value(&self, handle: *const (), basis: f32) -> f32 {
        self.styles().calc_value(handle, basis)
    }

    fn set_unrounded_layout(&mut self, node: NodeId, layout: &Layout) {
        let id = ElementId::from(node);
        self.elements[id].layout = *layout;
        self.elements.mark(id, Dirty::GEOMETRY, Dirty::GEOMETRY_BELOW);
    }

    fn compute_child_layout(&mut self, node: NodeId, inputs: LayoutInput) -> LayoutOutput {
        self.compute_element_layout(node, inputs, None)
    }
}

impl LayoutContainingBlock for LayoutPass<'_> {
    type OofItemStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    fn get_oof_item_style(&self, node: NodeId) -> Self::OofItemStyle<'_> {
        self.style(node)
    }

    /// Nothing to clear: an out-of-flow element records its containing block itself, and the
    /// record is replaced whenever the element is positioned again.
    fn clear_hoisted_children(&mut self, _containing_block: NodeId) {}

    fn add_hoisted_children(&mut self, containing_block: NodeId, hoisted: &[NodeId]) {
        let containing_block = ElementId::from(containing_block);
        for positioned in hoisted {
            self.elements[ElementId::from(*positioned)].containing_block = Some(containing_block);
        }
    }

    fn get_detailed_layout_info(&self, node: NodeId) -> &DetailedLayoutInfo<String> {
        &self.elements[ElementId::from(node)].grid_info
    }
}

impl CacheTree for LayoutPass<'_> {
    fn cache_get(&mut self, node: NodeId, inputs: &LayoutInput) -> Option<LayoutOutput> {
        self.elements[ElementId::from(node)].layout_cache.get(inputs)
    }

    fn cache_store(&mut self, node: NodeId, inputs: &LayoutInput, output: LayoutOutput) {
        self.elements[ElementId::from(node)].layout_cache.store(inputs, output);
    }

    fn cache_clear(&mut self, node: NodeId) {
        self.elements[ElementId::from(node)].layout_cache.clear();
    }
}

impl LayoutFlexboxContainer for LayoutPass<'_> {
    type FlexboxContainerStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    fn get_flexbox_container_style(&self, node: NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.style(node)
    }

    fn get_flexbox_child_style(&self, child: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.style(child)
    }
}

impl LayoutGridContainer for LayoutPass<'_> {
    type GridContainerStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    type GridItemStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    fn get_grid_container_style(&self, node: NodeId) -> Self::GridContainerStyle<'_> {
        self.style(node)
    }

    fn get_grid_child_style(&self, child: NodeId) -> Self::GridItemStyle<'_> {
        self.style(child)
    }

    /// Kept for the absolutely positioned children of the grid: their containing block is the
    /// grid area their placement names.
    fn set_detailed_grid_info(&mut self, node: NodeId, grid_info: DetailedGridInfo<String>) {
        let element = &mut self.elements[ElementId::from(node)];
        match &mut element.grid_info {
            DetailedLayoutInfo::Grid(stored) => **stored = grid_info,
            DetailedLayoutInfo::None => element.grid_info = DetailedLayoutInfo::Grid(Box::new(grid_info)),
        }
    }
}

impl LayoutBlockContainer for LayoutPass<'_> {
    type BlockContainerStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    type BlockItemStyle<'a>
        = TaffyStyle<'a>
    where
        Self: 'a;

    fn get_block_container_style(&self, node: NodeId) -> Self::BlockContainerStyle<'_> {
        self.style(node)
    }

    fn get_block_child_style(&self, child: NodeId) -> Self::BlockItemStyle<'_> {
        self.style(child)
    }

    fn compute_block_child_layout(
        &mut self,
        node: NodeId,
        inputs: LayoutInput,
        block_context: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        self.compute_element_layout(node, inputs, block_context)
    }
}
