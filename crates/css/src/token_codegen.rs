use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

use crate::compiled_tokens::CompiledTokens;
use crate::schema_literal::SchemaLiteral;
use crate::schema_path::SchemaPath;
use crate::token_id::TokenId;
use crate::value_kind::ValueKind;

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
    let light = tokens.light.literal(&schema);
    let dark = tokens.dark.literal(&schema);
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
    match id.kind() {
        ValueKind::Color => quote!(#schema::Color),
        ValueKind::Length => quote!(#schema::Pixels),
        ValueKind::FontFamilies => quote!(#schema::FontFamilies),
        ValueKind::Leading => quote!(#schema::Leading),
        ValueKind::FontWeight => quote!(#schema::FontWeight),
        ValueKind::Shadow => quote!(#schema::Shadow),
        ValueKind::Duration => quote!(#schema::Milliseconds),
        ValueKind::Easing => quote!(#schema::Easing),
    }
}
