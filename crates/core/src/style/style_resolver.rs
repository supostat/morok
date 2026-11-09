use morok_style::{
    BackgroundImage, BoxShadow, Color, ColorValue, Declaration, FontWeightValue, LineHeight, StyleRule, StyleSheet,
    Theme,
};

use crate::style::computed_style::ComputedStyle;
use crate::style::declaration_source::DeclarationSource;
use crate::style::effect_style::EffectStyle;
use crate::style::inline_style::InlineStyle;
use crate::style::layout_style::LayoutStyle;
use crate::style::paint_style::{INITIAL_COLOR, PaintStyle, TRANSPARENT};
use crate::style::positioning::Positioning;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;
use crate::style::selector_target::SelectorTarget;
use crate::style::styled_parent::StyledParent;
use crate::style::text_style::TextStyle;

/// Computes the style of an element from the rules that match it, its inline declarations, its
/// parent and the active theme.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StyleResolver<'a> {
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) theme: &'a Theme,
}

impl<'a> StyleResolver<'a> {
    /// `matched_rules` is scratch space; it comes back holding the rules that matched, in cascade order.
    pub(crate) fn resolve(
        self,
        subject: SelectorTarget<'_>,
        parent: Option<StyledParent<'_>>,
        inline_style: &InlineStyle,
        matched_rules: &mut Vec<u32>,
    ) -> ComputedStyle {
        self.collect_matched_rules(subject, parent.map(|parent| parent.target), matched_rules);
        let inherited_color = parent.map_or(INITIAL_COLOR, |parent| parent.color);
        let color = self.cascaded_color(matched_rules, inline_style, inherited_color);
        let mut style = ComputedStyle {
            layout: LayoutStyle::INITIAL,
            paint: PaintStyle::initial(color),
            text: parent.map_or(TextStyle::INITIAL, |parent| parent.text.inherited()),
            effects: EffectStyle::INITIAL,
        };
        for &rule in matched_rules.iter() {
            for (position, declaration) in self.rule(rule).declarations.iter().enumerate() {
                let source = DeclarationSource::Rule {
                    rule,
                    declaration: position as u32,
                };
                self.apply(&mut style, declaration, source);
            }
        }
        for (position, declaration) in inline_style.declarations().iter().enumerate() {
            let source = DeclarationSource::Inline {
                declaration: position as u32,
                revision: inline_style.revision(),
            };
            self.apply(&mut style, declaration, source);
        }
        style
    }

    /// Whether a child rule matches `child` under one of the two parents and not under the other,
    /// so that changing the parent from one to the other changes the style of the child.
    pub(crate) fn child_rules_tell_parents_apart(
        self,
        child: SelectorTarget<'_>,
        parent_before: SelectorTarget<'_>,
        parent_after: SelectorTarget<'_>,
    ) -> bool {
        self.keyed_rules(child)
            .filter(|rule| child.matches(&rule.selector.subject))
            .filter_map(|rule| rule.selector.parent.as_ref())
            .any(|parent_selector| parent_before.matches(parent_selector) != parent_after.matches(parent_selector))
    }

    fn rule(self, rule: u32) -> &'a StyleRule {
        &self.style_sheet.rules[rule as usize]
    }

    /// The rules a class of the element keys, each paired with its position in the cascade.
    fn keyed_rule_positions(self, subject: SelectorTarget<'_>) -> impl Iterator<Item = u32> {
        subject
            .classes
            .iter()
            .flat_map(move |class| self.style_sheet.class_style(*class).rules.iter().copied())
    }

    fn keyed_rules(self, subject: SelectorTarget<'_>) -> impl Iterator<Item = &'a StyleRule> {
        self.keyed_rule_positions(subject).map(move |rule| self.rule(rule))
    }

    fn collect_matched_rules(
        self,
        subject: SelectorTarget<'_>,
        parent: Option<SelectorTarget<'_>>,
        matched_rules: &mut Vec<u32>,
    ) {
        matched_rules.clear();
        matched_rules.extend(self.keyed_rule_positions(subject).filter(|&rule| {
            let selector = &self.rule(rule).selector;
            let parent_matches = match &selector.parent {
                None => true,
                Some(parent_selector) => parent.is_some_and(|parent| parent.matches(parent_selector)),
            };
            parent_matches && subject.matches(&selector.subject)
        }));
        matched_rules.sort_unstable();
    }

    /// The `color` of the element. It is cascaded before every other property because they may
    /// refer to it as `currentColor`.
    fn cascaded_color(self, matched_rules: &[u32], inline_style: &InlineStyle, inherited_color: Color) -> Color {
        let rule_declarations = matched_rules
            .iter()
            .flat_map(|&rule| self.rule(rule).declarations.iter());
        rule_declarations
            .chain(inline_style.declarations())
            .filter_map(|declaration| match declaration {
                Declaration::Color(color) => Some(*color),
                _ => None,
            })
            .next_back()
            .map_or(inherited_color, |color| self.color(color, inherited_color))
    }

    fn color(self, color: ColorValue, current_color: Color) -> Color {
        match color {
            ColorValue::Token(token) => *self.theme.resolve(token),
            ColorValue::Transparent => TRANSPARENT,
            ColorValue::CurrentColor => current_color,
        }
    }

    fn apply(self, style: &mut ComputedStyle, declaration: &Declaration, source: DeclarationSource) {
        let theme = self.theme;
        let current_color = style.paint.color;
        let ComputedStyle {
            layout,
            paint,
            text,
            effects,
        } = style;
        match declaration {
            Declaration::Display(display) => layout.display = *display,
            Declaration::Position(position) => layout.positioning = Positioning::from(*position),
            Declaration::OverflowX(overflow) => layout.overflow_x = *overflow,
            Declaration::OverflowY(overflow) => layout.overflow_y = *overflow,
            Declaration::Width(size) => layout.width = ResolvedLengthOrAuto::of(size, theme),
            Declaration::Height(size) => layout.height = ResolvedLengthOrAuto::of(size, theme),
            Declaration::MinWidth(size) => layout.min_width = ResolvedLengthOrAuto::of(size, theme),
            Declaration::MinHeight(size) => layout.min_height = ResolvedLengthOrAuto::of(size, theme),
            Declaration::MaxWidth(size) => layout.max_width = ResolvedLengthOrAuto::of(size, theme),
            Declaration::MaxHeight(size) => layout.max_height = ResolvedLengthOrAuto::of(size, theme),
            Declaration::Margin(side, margin) => layout.margin[*side] = ResolvedLengthOrAuto::of(margin, theme),
            Declaration::Padding(side, padding) => layout.padding[*side] = ResolvedLength::of(padding, theme),
            Declaration::Inset(side, inset) => layout.inset[*side] = ResolvedLengthOrAuto::of(inset, theme),
            Declaration::FlexDirection(direction) => layout.flex_direction = *direction,
            Declaration::FlexWrap(wrap) => layout.flex_wrap = *wrap,
            Declaration::FlexGrow(grow) => layout.flex_grow = *grow,
            Declaration::FlexShrink(shrink) => layout.flex_shrink = *shrink,
            Declaration::FlexBasis(basis) => layout.flex_basis = ResolvedLengthOrAuto::of(basis, theme),
            Declaration::JustifyContent(alignment) => layout.justify_content = *alignment,
            Declaration::AlignContent(alignment) => layout.align_content = *alignment,
            Declaration::JustifyItems(alignment) => layout.justify_items = *alignment,
            Declaration::AlignItems(alignment) => layout.align_items = *alignment,
            Declaration::JustifySelf(alignment) => layout.justify_self = *alignment,
            Declaration::AlignSelf(alignment) => layout.align_self = *alignment,
            Declaration::RowGap(gap) => layout.row_gap = ResolvedLength::of(gap, theme),
            Declaration::ColumnGap(gap) => layout.column_gap = ResolvedLength::of(gap, theme),
            Declaration::GridTemplateColumns(_) => layout.grid_template_columns = Some(source),
            Declaration::GridTemplateRows(_) => layout.grid_template_rows = Some(source),
            Declaration::GridAutoColumns(_) => layout.grid_auto_columns = Some(source),
            Declaration::GridAutoRows(_) => layout.grid_auto_rows = Some(source),
            Declaration::GridAutoFlow(flow) => layout.grid_auto_flow = *flow,
            Declaration::GridColumnStart(line) => layout.grid_column_start = *line,
            Declaration::GridColumnEnd(line) => layout.grid_column_end = *line,
            Declaration::GridRowStart(line) => layout.grid_row_start = *line,
            Declaration::GridRowEnd(line) => layout.grid_row_end = *line,
            Declaration::BackgroundColor(color) => paint.background_color = self.color(*color, current_color),
            Declaration::BackgroundImage(image) => {
                paint.background_image = match image {
                    BackgroundImage::None => None,
                    BackgroundImage::Gradient(_) => Some(source),
                };
            }
            Declaration::BorderWidth(side, width) => {
                layout.border_width[*side] = ResolvedLength::pixels_of(width, theme);
            }
            Declaration::BorderStyle(side, line_style) => layout.border_style[*side] = *line_style,
            Declaration::BorderColor(side, color) => paint.border_color[*side] = self.color(*color, current_color),
            Declaration::BorderRadius(corner, radius) => {
                paint.border_radius[*corner] = ResolvedLength::pixels_of(radius, theme);
            }
            Declaration::BoxShadow(shadow) => {
                paint.box_shadow = match shadow {
                    BoxShadow::None => None,
                    BoxShadow::Token(token) => Some(*token),
                };
            }
            Declaration::OutlineWidth(width) => paint.outline_width = ResolvedLength::pixels_of(width, theme),
            Declaration::OutlineStyle(line_style) => paint.outline_style = *line_style,
            Declaration::OutlineColor(color) => paint.outline_color = self.color(*color, current_color),
            Declaration::OutlineOffset(offset) => paint.outline_offset = ResolvedLength::pixels_of(offset, theme),
            Declaration::Color(_) => {}
            Declaration::FontFamily(families) => text.font_family = Some(*families),
            Declaration::FontSize(size) => text.font_size = ResolvedLength::pixels_of(size, theme).max(0.0),
            Declaration::FontWeight(weight) => {
                text.font_weight = match weight {
                    FontWeightValue::Token(token) => *theme.resolve(*token),
                    FontWeightValue::Keyword(weight) => *weight,
                };
            }
            Declaration::FontStyle(font_style) => text.font_style = *font_style,
            Declaration::LineHeight(line_height) => {
                text.line_height = match line_height {
                    LineHeight::Normal => None,
                    LineHeight::Token(token) => Some(theme.resolve(*token).0),
                };
            }
            Declaration::LetterSpacing(spacing) => text.letter_spacing = ResolvedLength::pixels_of(spacing, theme),
            Declaration::TextAlign(alignment) => text.text_align = *alignment,
            Declaration::TextDecoration(decoration) => text.text_decoration = *decoration,
            Declaration::TextOverflow(overflow) => text.text_overflow = *overflow,
            Declaration::WhiteSpace(white_space) => text.white_space = *white_space,
            Declaration::FontVariantNumeric(numeric) => text.font_variant_numeric = *numeric,
            Declaration::Opacity(_)
            | Declaration::Transform(_)
            | Declaration::TransformOrigin(..)
            | Declaration::Filter(_)
            | Declaration::BackdropFilter(_)
            | Declaration::MixBlendMode(_) => self.apply_effect(effects, declaration, source),
        }
    }

    /// The properties that composite the element and its content as a group. A list stays
    /// where it is declared and is read from there; an empty one is no list at all.
    fn apply_effect(self, effects: &mut EffectStyle, declaration: &Declaration, source: DeclarationSource) {
        let theme = self.theme;
        match declaration {
            Declaration::Opacity(opacity) => effects.opacity = opacity.clamp(0.0, 1.0),
            Declaration::Transform(functions) => effects.transform = (!functions.is_empty()).then_some(source),
            Declaration::TransformOrigin(horizontal, vertical) => {
                effects.transform_origin_x = ResolvedLength::of(horizontal, theme);
                effects.transform_origin_y = ResolvedLength::of(vertical, theme);
            }
            Declaration::Filter(functions) => effects.filter = (!functions.is_empty()).then_some(source),
            Declaration::BackdropFilter(functions) => {
                effects.backdrop_filter = (!functions.is_empty()).then_some(source);
            }
            Declaration::MixBlendMode(mode) => effects.mix_blend_mode = *mode,
            _ => unreachable!("only the effect properties are applied here"),
        }
    }
}
