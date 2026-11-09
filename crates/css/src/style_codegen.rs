use morok_style::StyleSheet;
use proc_macro2::{Ident, Literal, Span};
use quote::quote;

use crate::schema_literal::SchemaLiteral;
use crate::schema_path::SchemaPath;

/// Generates the Rust source of a compiled style sheet: the `Classes` struct of class IDs with
/// its `CLASSES` constant, and the `STYLE_SHEET` table.
pub fn generate_styles_source(style_sheet: &StyleSheet, schema: SchemaPath) -> String {
    let schema = schema.path();
    let fields: Vec<Ident> = style_sheet.classes.iter().map(|class| field(&class.name)).collect();
    let indices = (0..style_sheet.classes.len()).map(Literal::usize_unsuffixed);
    let style_sheet = style_sheet.literal(&schema);
    let module = quote! {
        #[derive(Clone, Copy, Debug)]
        pub struct Classes {
            #(pub #fields: #schema::Class,)*
        }

        pub const CLASSES: Classes = Classes {
            #(#fields: #schema::Class::new(#indices),)*
        };

        pub const STYLE_SHEET: #schema::StyleSheet = #style_sheet;
    };
    let file = syn::parse2::<syn::File>(module).expect("generated style tables are valid Rust");
    prettyplease::unparse(&file)
}

/// The field of a class: its name in snake case, as a raw identifier when it is a Rust keyword.
fn field(class_name: &str) -> Ident {
    let snake_case = class_name.replace('-', "_");
    syn::parse_str::<Ident>(&snake_case).unwrap_or_else(|_| Ident::new_raw(&snake_case, Span::call_site()))
}
