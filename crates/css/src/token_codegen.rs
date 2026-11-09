use morok_style::{Color, Easing, FontFamilies, Pixels, Shadow, ShadowLayer, StepPosition, Theme};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::compiled_tokens::CompiledTokens;
use crate::schema_path::SchemaPath;
use crate::token_id::TokenId;

/// Generates the Rust source of compiled tokens: the `Tokens` struct of typed IDs with its `TOKENS`
/// constant, and the `LIGHT` and `DARK` theme tables.
pub fn generate_tokens_source(tokens: &CompiledTokens, schema: SchemaPath) -> String {
    let schema = schema.path();
    let fields: Vec<Ident> = tokens.entries.iter().map(|entry| field(&entry.name)).collect();
    let value_types = tokens.entries.iter().map(|entry| value_type(entry.id, &schema));
    let indices = tokens
        .entries
        .iter()
        .map(|entry| Literal::usize_unsuffixed(entry.id.index()));
    let light = theme(&tokens.light, &schema);
    let dark = theme(&tokens.dark, &schema);
    let module = quote! {
        #[derive(Clone, Copy, Debug)]
        pub struct Tokens {
            #(pub #fields: #schema::Token<#value_types>,)*
        }

        pub const TOKENS: Tokens = Tokens {
            #(#fields: #schema::Token::new(#indices),)*
        };

        pub const LIGHT: #schema::Theme = #light;

        pub const DARK: #schema::Theme = #dark;
    };
    let file = syn::parse2::<syn::File>(module).expect("generated token tables are valid Rust");
    prettyplease::unparse(&file)
}

fn field(token_name: &str) -> Ident {
    format_ident!("{}", token_name.replace('-', "_"))
}

fn value_type(id: TokenId, schema: &TokenStream) -> TokenStream {
    match id {
        TokenId::Color(_) => quote!(#schema::Color),
        TokenId::Length(_) => quote!(#schema::Pixels),
        TokenId::FontFamilies(_) => quote!(#schema::FontFamilies),
        TokenId::Leading(_) => quote!(#schema::Leading),
        TokenId::FontWeight(_) => quote!(#schema::FontWeight),
        TokenId::Shadow(_) => quote!(#schema::Shadow),
        TokenId::Duration(_) => quote!(#schema::Milliseconds),
        TokenId::Easing(_) => quote!(#schema::Easing),
    }
}

fn theme(theme: &Theme, schema: &TokenStream) -> TokenStream {
    let colors = borrowed_slice(theme.colors.iter().map(|color| color_literal(*color, schema)));
    let lengths = borrowed_slice(theme.lengths.iter().map(|length| pixels_literal(*length, schema)));
    let font_families = borrowed_slice(
        theme
            .font_families
            .iter()
            .map(|families| font_families_literal(families, schema)),
    );
    let leadings = borrowed_slice(theme.leadings.iter().map(|leading| {
        let multiplier = float_literal(leading.0);
        quote!(#schema::Leading(#multiplier))
    }));
    let font_weights = borrowed_slice(theme.font_weights.iter().map(|weight| {
        let weight = Literal::u16_unsuffixed(weight.0);
        quote!(#schema::FontWeight(#weight))
    }));
    let shadows = borrowed_slice(theme.shadows.iter().map(|shadow| shadow_literal(shadow, schema)));
    let durations = borrowed_slice(theme.durations.iter().map(|duration| {
        let milliseconds = float_literal(duration.0);
        quote!(#schema::Milliseconds(#milliseconds))
    }));
    let easings = borrowed_slice(theme.easings.iter().map(|easing| easing_literal(*easing, schema)));
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

fn borrowed_slice(elements: impl Iterator<Item = TokenStream>) -> TokenStream {
    quote!(::std::borrow::Cow::Borrowed(&[#(#elements),*]))
}

fn float_literal(value: f32) -> TokenStream {
    let magnitude = Literal::f32_unsuffixed(value.abs());
    if value.is_sign_negative() {
        quote!(-#magnitude)
    } else {
        quote!(#magnitude)
    }
}

fn color_literal(color: Color, schema: &TokenStream) -> TokenStream {
    let [red, green, blue] = [color.red, color.green, color.blue].map(Literal::u8_unsuffixed);
    let alpha = float_literal(color.alpha);
    quote!(#schema::Color { red: #red, green: #green, blue: #blue, alpha: #alpha })
}

fn pixels_literal(length: Pixels, schema: &TokenStream) -> TokenStream {
    let pixels = float_literal(length.0);
    quote!(#schema::Pixels(#pixels))
}

fn font_families_literal(families: &FontFamilies, schema: &TokenStream) -> TokenStream {
    let names = borrowed_slice(families.names.iter().map(|name| {
        let name: &str = name;
        quote!(::std::borrow::Cow::Borrowed(#name))
    }));
    quote!(#schema::FontFamilies { names: #names })
}

fn shadow_literal(shadow: &Shadow, schema: &TokenStream) -> TokenStream {
    let layers = borrowed_slice(shadow.layers.iter().map(|layer| shadow_layer_literal(*layer, schema)));
    quote!(#schema::Shadow { layers: #layers })
}

fn shadow_layer_literal(layer: ShadowLayer, schema: &TokenStream) -> TokenStream {
    let offset_x = pixels_literal(layer.offset_x, schema);
    let offset_y = pixels_literal(layer.offset_y, schema);
    let blur = pixels_literal(layer.blur, schema);
    let spread = pixels_literal(layer.spread, schema);
    let color = color_literal(layer.color, schema);
    let inset = layer.inset;
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

fn easing_literal(easing: Easing, schema: &TokenStream) -> TokenStream {
    match easing {
        Easing::Linear => quote!(#schema::Easing::Linear),
        Easing::CubicBezier { x1, y1, x2, y2 } => {
            let [x1, y1, x2, y2] = [x1, y1, x2, y2].map(float_literal);
            quote!(#schema::Easing::CubicBezier { x1: #x1, y1: #y1, x2: #x2, y2: #y2 })
        }
        Easing::Steps { count, position } => {
            let count = Literal::u32_unsuffixed(count);
            let position = match position {
                StepPosition::JumpStart => quote!(JumpStart),
                StepPosition::JumpEnd => quote!(JumpEnd),
                StepPosition::JumpNone => quote!(JumpNone),
                StepPosition::JumpBoth => quote!(JumpBoth),
            };
            quote!(#schema::Easing::Steps { count: #count, position: #schema::StepPosition::#position })
        }
    }
}
