use std::borrow::Cow;
use std::fmt;

use morok_style::{
    BackdropFilterFunction, BackgroundImage, BlendMode, BoxShadow, Class, ClassStyle, Color, ColorStop, ColorValue,
    CompoundSelector, ContentAlignment, Corner, Declaration, Display, Easing, FilterFunction, FlexDirection, FlexWrap,
    FontFamilies, FontStyle, FontVariantNumeric, FontWeight, FontWeightValue, Gradient, GradientDirection,
    GridAutoFlow, GridLine, ItemAlignment, Leading, Length, LengthPercentage, LengthPercentageAuto, LineHeight,
    LineStyle, LinearGradient, Milliseconds, ObjectFit, Overflow, Pixels, Position, RadialExtent, RadialGradient,
    RadialShape, RepeatCount, Selector, SelfAlignment, Shadow, ShadowLayer, Side, States, StepPosition, StyleRule,
    StyleSheet, TextAlign, TextDecoration, TextOverflow, Theme, Token, TokenTerm, TrackBreadth, TrackListItem,
    TrackSize, TransformFunction, WhiteSpace,
};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

/// A schema value that prints itself as the Rust expression constructing it; `schema` is the path
/// the expression reaches the schema types by.
pub(crate) trait SchemaLiteral {
    fn literal(&self, schema: &TokenStream) -> TokenStream;
}

/// The name of the enum variant of `value`, read from its derived `Debug` output.
fn variant_name(value: &impl fmt::Debug) -> Ident {
    let debug = format!("{value:?}");
    let name_length = debug
        .find(|character: char| !character.is_alphanumeric())
        .unwrap_or(debug.len());
    format_ident!("{}", &debug[..name_length])
}

/// The expression of an enum value: a unit variant, or a tuple variant with its arguments.
fn variant(
    type_name: &str,
    value: &impl fmt::Debug,
    arguments: &[&dyn SchemaLiteral],
    schema: &TokenStream,
) -> TokenStream {
    let type_name = format_ident!("{type_name}");
    let variant = variant_name(value);
    if arguments.is_empty() {
        return quote!(#schema::#type_name::#variant);
    }
    let arguments = arguments.iter().map(|argument| argument.literal(schema));
    quote!(#schema::#type_name::#variant(#(#arguments),*))
}

fn one(argument: &dyn SchemaLiteral) -> Vec<&dyn SchemaLiteral> {
    vec![argument]
}

fn two<'a>(first: &'a dyn SchemaLiteral, second: &'a dyn SchemaLiteral) -> Vec<&'a dyn SchemaLiteral> {
    vec![first, second]
}

macro_rules! unit_enum_literals {
    ($($type:ident),* $(,)?) => {
        $(impl SchemaLiteral for $type {
            fn literal(&self, schema: &TokenStream) -> TokenStream {
                variant(stringify!($type), self, &[], schema)
            }
        })*
    };
}

unit_enum_literals!(
    BlendMode,
    ContentAlignment,
    Corner,
    Display,
    FlexDirection,
    FlexWrap,
    FontStyle,
    FontVariantNumeric,
    GridAutoFlow,
    ItemAlignment,
    LineStyle,
    ObjectFit,
    Overflow,
    Position,
    RadialExtent,
    RadialShape,
    Side,
    StepPosition,
    TextAlign,
    TextDecoration,
    TextOverflow,
    WhiteSpace,
);

impl SchemaLiteral for f32 {
    fn literal(&self, _schema: &TokenStream) -> TokenStream {
        let magnitude = Literal::f32_unsuffixed(self.abs());
        if self.is_sign_negative() {
            quote!(-#magnitude)
        } else {
            quote!(#magnitude)
        }
    }
}

impl SchemaLiteral for u32 {
    fn literal(&self, _schema: &TokenStream) -> TokenStream {
        let value = Literal::u32_unsuffixed(*self);
        quote!(#value)
    }
}

impl SchemaLiteral for u16 {
    fn literal(&self, _schema: &TokenStream) -> TokenStream {
        let value = Literal::u16_unsuffixed(*self);
        quote!(#value)
    }
}

impl SchemaLiteral for i16 {
    fn literal(&self, _schema: &TokenStream) -> TokenStream {
        let magnitude = Literal::u16_unsuffixed(self.unsigned_abs());
        if self.is_negative() {
            quote!(-#magnitude)
        } else {
            quote!(#magnitude)
        }
    }
}

impl SchemaLiteral for Cow<'static, str> {
    fn literal(&self, _schema: &TokenStream) -> TokenStream {
        let text: &str = self;
        quote!(::std::borrow::Cow::Borrowed(#text))
    }
}

impl<T: SchemaLiteral + Clone> SchemaLiteral for Cow<'static, [T]> {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let elements = self.iter().map(|element| element.literal(schema));
        quote!(::std::borrow::Cow::Borrowed(&[#(#elements),*]))
    }
}

impl<T: SchemaLiteral> SchemaLiteral for Option<T> {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        if let Some(value) = self {
            let value = value.literal(schema);
            quote!(::core::option::Option::Some(#value))
        } else {
            quote!(::core::option::Option::None)
        }
    }
}

impl<V> SchemaLiteral for Token<V> {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let index = Literal::usize_unsuffixed(self.index());
        quote!(#schema::Token::new(#index))
    }
}

impl SchemaLiteral for Class {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let index = Literal::usize_unsuffixed(self.index());
        quote!(#schema::Class::new(#index))
    }
}

impl SchemaLiteral for Color {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let [red, green, blue] = [self.red, self.green, self.blue].map(Literal::u8_unsuffixed);
        let alpha = self.alpha.literal(schema);
        quote!(#schema::Color { red: #red, green: #green, blue: #blue, alpha: #alpha })
    }
}

impl SchemaLiteral for Pixels {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let pixels = self.0.literal(schema);
        quote!(#schema::Pixels(#pixels))
    }
}

impl SchemaLiteral for Leading {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let multiplier = self.0.literal(schema);
        quote!(#schema::Leading(#multiplier))
    }
}

impl SchemaLiteral for FontWeight {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let weight = self.0.literal(schema);
        quote!(#schema::FontWeight(#weight))
    }
}

impl SchemaLiteral for Milliseconds {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let milliseconds = self.0.literal(schema);
        quote!(#schema::Milliseconds(#milliseconds))
    }
}

impl SchemaLiteral for FontFamilies {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let names = self.names.literal(schema);
        quote!(#schema::FontFamilies { names: #names })
    }
}

impl SchemaLiteral for ShadowLayer {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let offset_x = self.offset_x.literal(schema);
        let offset_y = self.offset_y.literal(schema);
        let blur = self.blur.literal(schema);
        let spread = self.spread.literal(schema);
        let color = self.color.literal(schema);
        let inset = self.inset;
        quote! {
            #schema::ShadowLayer {
                offset_x: #offset_x,
                offset_y: #offset_y,
                blur: #blur,
                spread: #spread,
                color: #color,
                inset: #inset,
            }
        }
    }
}

impl SchemaLiteral for Shadow {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let layers = self.layers.literal(schema);
        quote!(#schema::Shadow { layers: #layers })
    }
}

impl SchemaLiteral for Easing {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Linear => quote!(#schema::Easing::Linear),
            Self::CubicBezier { x1, y1, x2, y2 } => {
                let [x1, y1, x2, y2] = [x1, y1, x2, y2].map(|coordinate| coordinate.literal(schema));
                quote!(#schema::Easing::CubicBezier { x1: #x1, y1: #y1, x2: #x2, y2: #y2 })
            }
            Self::Steps { count, position } => {
                let count = count.literal(schema);
                let position = position.literal(schema);
                quote!(#schema::Easing::Steps { count: #count, position: #position })
            }
        }
    }
}

impl SchemaLiteral for Theme {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let colors = self.colors.literal(schema);
        let lengths = self.lengths.literal(schema);
        let font_families = self.font_families.literal(schema);
        let leadings = self.leadings.literal(schema);
        let font_weights = self.font_weights.literal(schema);
        let shadows = self.shadows.literal(schema);
        let durations = self.durations.literal(schema);
        let easings = self.easings.literal(schema);
        quote! {
            #schema::Theme {
                colors: #colors,
                lengths: #lengths,
                font_families: #font_families,
                leadings: #leadings,
                font_weights: #font_weights,
                shadows: #shadows,
                durations: #durations,
                easings: #easings,
            }
        }
    }
}

impl SchemaLiteral for States {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let mut constants = States::PSEUDO_CLASSES
            .into_iter()
            .filter(|(_, state)| self.contains(*state))
            .map(|(pseudo_class, _)| format_ident!("{}", pseudo_class.replace('-', "_").to_ascii_uppercase()));
        let Some(first) = constants.next() else {
            return quote!(#schema::States::NONE);
        };
        constants.fold(
            quote!(#schema::States::#first),
            |union, constant| quote!(#union.union(#schema::States::#constant)),
        )
    }
}

impl SchemaLiteral for CompoundSelector {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let classes = self.classes.literal(schema);
        let states = self.states.literal(schema);
        quote!(#schema::CompoundSelector { classes: #classes, states: #states })
    }
}

impl SchemaLiteral for Selector {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let subject = self.subject.literal(schema);
        let parent = self.parent.literal(schema);
        quote!(#schema::Selector { subject: #subject, parent: #parent })
    }
}

impl SchemaLiteral for StyleRule {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let selector = self.selector.literal(schema);
        let declarations = self.declarations.literal(schema);
        quote!(#schema::StyleRule { selector: #selector, declarations: #declarations })
    }
}

impl SchemaLiteral for ClassStyle {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let name = self.name.literal(schema);
        let rules = self.rules.literal(schema);
        quote!(#schema::ClassStyle { name: #name, rules: #rules })
    }
}

impl SchemaLiteral for StyleSheet {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let rules = self.rules.literal(schema);
        let classes = self.classes.literal(schema);
        quote!(#schema::StyleSheet { rules: #rules, classes: #classes })
    }
}

impl SchemaLiteral for TokenTerm {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let token = self.token.literal(schema);
        let factor = self.factor.literal(schema);
        quote!(#schema::TokenTerm { token: #token, factor: #factor })
    }
}

impl SchemaLiteral for Length {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let pixels = self.pixels.literal(schema);
        let tokens = self.tokens.literal(schema);
        quote!(#schema::Length { pixels: #pixels, tokens: #tokens })
    }
}

impl SchemaLiteral for LengthPercentage {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let pixels = self.pixels.literal(schema);
        let fraction = self.fraction.literal(schema);
        let tokens = self.tokens.literal(schema);
        quote!(#schema::LengthPercentage { pixels: #pixels, fraction: #fraction, tokens: #tokens })
    }
}

impl SchemaLiteral for LengthPercentageAuto {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Auto => variant("LengthPercentageAuto", self, &[], schema),
            Self::LengthPercentage(length) => variant("LengthPercentageAuto", self, &one(length), schema),
        }
    }
}

impl SchemaLiteral for SelfAlignment {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Auto => variant("SelfAlignment", self, &[], schema),
            Self::Item(alignment) => variant("SelfAlignment", self, &one(alignment), schema),
        }
    }
}

impl SchemaLiteral for GridLine {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Auto => variant("GridLine", self, &[], schema),
            Self::Line(number) => variant("GridLine", self, &one(number), schema),
            Self::Span(count) => variant("GridLine", self, &one(count), schema),
        }
    }
}

impl SchemaLiteral for RepeatCount {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Times(times) => variant("RepeatCount", self, &one(times), schema),
            Self::AutoFill | Self::AutoFit => variant("RepeatCount", self, &[], schema),
        }
    }
}

impl SchemaLiteral for TrackBreadth {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::LengthPercentage(length) => variant("TrackBreadth", self, &one(length), schema),
            Self::Flex(share) => variant("TrackBreadth", self, &one(share), schema),
            Self::Auto | Self::MinContent | Self::MaxContent => variant("TrackBreadth", self, &[], schema),
        }
    }
}

impl SchemaLiteral for TrackSize {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Breadth(breadth) => variant("TrackSize", self, &one(breadth), schema),
            Self::FitContent(limit) => variant("TrackSize", self, &one(limit), schema),
            Self::MinMax { min, max } => {
                let min = min.literal(schema);
                let max = max.literal(schema);
                quote!(#schema::TrackSize::MinMax { min: #min, max: #max })
            }
        }
    }
}

impl SchemaLiteral for TrackListItem {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Track(size) => variant("TrackListItem", self, &one(size), schema),
            Self::Repeat { count, tracks } => {
                let count = count.literal(schema);
                let tracks = tracks.literal(schema);
                quote!(#schema::TrackListItem::Repeat { count: #count, tracks: #tracks })
            }
        }
    }
}

impl SchemaLiteral for ColorValue {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Token(token) => variant("ColorValue", self, &one(token), schema),
            Self::Transparent | Self::CurrentColor => variant("ColorValue", self, &[], schema),
        }
    }
}

impl SchemaLiteral for FontWeightValue {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Token(token) => variant("FontWeightValue", self, &one(token), schema),
            Self::Keyword(weight) => variant("FontWeightValue", self, &one(weight), schema),
        }
    }
}

impl SchemaLiteral for LineHeight {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Normal => variant("LineHeight", self, &[], schema),
            Self::Token(token) => variant("LineHeight", self, &one(token), schema),
        }
    }
}

impl SchemaLiteral for BoxShadow {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::None => variant("BoxShadow", self, &[], schema),
            Self::Token(token) => variant("BoxShadow", self, &one(token), schema),
        }
    }
}

impl SchemaLiteral for ColorStop {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let color = self.color.literal(schema);
        let position = self.position.literal(schema);
        quote!(#schema::ColorStop { color: #color, position: #position })
    }
}

impl SchemaLiteral for GradientDirection {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Angle(degrees) => variant("GradientDirection", self, &one(degrees), schema),
            Self::ToCorner(corner) => variant("GradientDirection", self, &one(corner), schema),
        }
    }
}

impl SchemaLiteral for LinearGradient {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let direction = self.direction.literal(schema);
        let stops = self.stops.literal(schema);
        quote!(#schema::LinearGradient { direction: #direction, stops: #stops })
    }
}

impl SchemaLiteral for RadialGradient {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let shape = self.shape.literal(schema);
        let extent = self.extent.literal(schema);
        let center_x = self.center_x.literal(schema);
        let center_y = self.center_y.literal(schema);
        let stops = self.stops.literal(schema);
        quote! {
            #schema::RadialGradient {
                shape: #shape,
                extent: #extent,
                center_x: #center_x,
                center_y: #center_y,
                stops: #stops,
            }
        }
    }
}

impl SchemaLiteral for Gradient {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Linear(gradient) => variant("Gradient", self, &one(gradient), schema),
            Self::Radial(gradient) => variant("Gradient", self, &one(gradient), schema),
        }
    }
}

impl SchemaLiteral for BackgroundImage {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::None => variant("BackgroundImage", self, &[], schema),
            Self::Gradient(gradient) => variant("BackgroundImage", self, &one(gradient), schema),
        }
    }
}

impl SchemaLiteral for TransformFunction {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Translate(x, y) => variant("TransformFunction", self, &two(x, y), schema),
            Self::TranslateX(offset) | Self::TranslateY(offset) => {
                variant("TransformFunction", self, &one(offset), schema)
            }
            Self::Scale(x, y) => variant("TransformFunction", self, &two(x, y), schema),
            Self::ScaleX(factor) | Self::ScaleY(factor) | Self::Rotate(factor) => {
                variant("TransformFunction", self, &one(factor), schema)
            }
        }
    }
}

impl SchemaLiteral for FilterFunction {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Blur(radius) => variant("FilterFunction", self, &one(radius), schema),
            Self::Brightness(amount)
            | Self::Contrast(amount)
            | Self::Grayscale(amount)
            | Self::Saturate(amount)
            | Self::Opacity(amount) => variant("FilterFunction", self, &one(amount), schema),
        }
    }
}

impl SchemaLiteral for BackdropFilterFunction {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        match self {
            Self::Blur(radius) => variant("BackdropFilterFunction", self, &one(radius), schema),
            Self::Brightness(amount) | Self::Saturate(amount) => {
                variant("BackdropFilterFunction", self, &one(amount), schema)
            }
        }
    }
}

impl SchemaLiteral for Declaration {
    fn literal(&self, schema: &TokenStream) -> TokenStream {
        let arguments = match self {
            Self::Display(display) => one(display),
            Self::Position(position) => one(position),
            Self::OverflowX(overflow) | Self::OverflowY(overflow) => one(overflow),
            Self::Width(size)
            | Self::Height(size)
            | Self::MinWidth(size)
            | Self::MinHeight(size)
            | Self::MaxWidth(size)
            | Self::MaxHeight(size)
            | Self::FlexBasis(size) => one(size),
            Self::Margin(side, offset) | Self::Inset(side, offset) => two(side, offset),
            Self::Padding(side, padding) => two(side, padding),
            Self::FlexDirection(direction) => one(direction),
            Self::FlexWrap(wrap) => one(wrap),
            Self::FlexGrow(factor) | Self::FlexShrink(factor) => one(factor),
            Self::JustifyContent(alignment) | Self::AlignContent(alignment) => one(alignment),
            Self::JustifyItems(alignment) | Self::AlignItems(alignment) => one(alignment),
            Self::JustifySelf(alignment) | Self::AlignSelf(alignment) => one(alignment),
            Self::RowGap(gap) | Self::ColumnGap(gap) => one(gap),
            Self::GridTemplateColumns(tracks) | Self::GridTemplateRows(tracks) => one(tracks),
            Self::GridAutoColumns(tracks) | Self::GridAutoRows(tracks) => one(tracks),
            Self::GridAutoFlow(flow) => one(flow),
            Self::GridColumnStart(line)
            | Self::GridColumnEnd(line)
            | Self::GridRowStart(line)
            | Self::GridRowEnd(line) => one(line),
            Self::BackgroundColor(color) | Self::OutlineColor(color) | Self::Color(color) => one(color),
            Self::BackgroundImage(image) => one(image),
            Self::BorderWidth(side, width) => two(side, width),
            Self::BorderStyle(side, style) => two(side, style),
            Self::BorderColor(side, color) => two(side, color),
            Self::BorderRadius(corner, radius) => two(corner, radius),
            Self::BoxShadow(shadow) => one(shadow),
            Self::OutlineWidth(length)
            | Self::OutlineOffset(length)
            | Self::FontSize(length)
            | Self::LetterSpacing(length) => one(length),
            Self::OutlineStyle(style) => one(style),
            Self::FontFamily(families) => one(families),
            Self::FontWeight(weight) => one(weight),
            Self::FontStyle(style) => one(style),
            Self::LineHeight(line_height) => one(line_height),
            Self::TextAlign(alignment) => one(alignment),
            Self::TextDecoration(decoration) => one(decoration),
            Self::TextOverflow(overflow) => one(overflow),
            Self::WhiteSpace(white_space) => one(white_space),
            Self::FontVariantNumeric(variant) => one(variant),
            Self::Opacity(opacity) => one(opacity),
            Self::Transform(functions) => one(functions),
            Self::TransformOrigin(horizontal, vertical) => two(horizontal, vertical),
            Self::Filter(functions) => one(functions),
            Self::BackdropFilter(functions) => one(functions),
            Self::MixBlendMode(mode) => one(mode),
            Self::ObjectFit(fit) => one(fit),
        };
        variant("Declaration", self, &arguments, schema)
    }
}
