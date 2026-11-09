use proc_macro2::TokenStream;
use quote::quote;

/// The path generated code reaches the style schema by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaPath {
    /// `::morok_style` — for crates of the kit that depend on the schema crate directly.
    Crate,
    /// `::morok::style` — for applications that depend on the `morok` facade.
    Facade,
}

impl SchemaPath {
    pub(crate) fn path(self) -> TokenStream {
        match self {
            Self::Crate => quote!(::morok_style),
            Self::Facade => quote!(::morok::style),
        }
    }
}
