use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::{Attribute, ImplItem, Item, Meta, TraitItem, Visibility};

/// A public item that carries no documentation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UndocumentedItem {
    /// The line of the name of the item.
    pub(crate) line: usize,
    /// The keyword of the item and its name, as in `fn retry`.
    pub(crate) item: String,
}

/// What the rule reads of an item: its attributes, its keyword and its name.
struct Declared<'item> {
    attributes: &'item [Attribute],
    keyword: &'static str,
    name: &'item Ident,
}

impl UndocumentedItem {
    /// The public items of a Rust source that carry no documentation, in the order of the
    /// source. Fields, variants, modules and re-exports are not items the rule asks of; code
    /// inside a macro invocation is not read.
    pub(crate) fn all_in(source: &str) -> syn::Result<Vec<Self>> {
        let file = syn::parse_file(source)?;
        let mut undocumented = Vec::new();
        collect(&file.items, &mut undocumented);
        Ok(undocumented)
    }
}

impl Declared<'_> {
    fn lacks_documentation(&self) -> bool {
        !is_exempt(self.attributes) && !self.attributes.iter().any(is_documentation)
    }

    fn undocumented(&self) -> UndocumentedItem {
        UndocumentedItem {
            line: self.name.span().start().line,
            item: format!("{} {}", self.keyword, self.name),
        }
    }
}

fn collect(items: &[Item], undocumented: &mut Vec<UndocumentedItem>) {
    for item in items {
        if let Some((visibility, declared)) = declared(item)
            && is_public(visibility)
            && declared.lacks_documentation()
        {
            undocumented.push(declared.undocumented());
        }
        match item {
            Item::Trait(public_trait) if is_public(&public_trait.vis) && !is_exempt(&public_trait.attrs) => {
                undocumented.extend(
                    public_trait
                        .items
                        .iter()
                        .filter_map(declared_in_trait)
                        .filter(Declared::lacks_documentation)
                        .map(|declared| declared.undocumented()),
                );
            }
            Item::Impl(inherent) if inherent.trait_.is_none() && !is_exempt(&inherent.attrs) => {
                undocumented.extend(
                    inherent
                        .items
                        .iter()
                        .filter_map(declared_in_impl)
                        .filter(|(visibility, declared)| is_public(visibility) && declared.lacks_documentation())
                        .map(|(_, declared)| declared.undocumented()),
                );
            }
            Item::Mod(module) if !is_exempt(&module.attrs) => {
                if let Some((_, inline_items)) = &module.content {
                    collect(inline_items, undocumented);
                }
            }
            _ => {}
        }
    }
}

fn declared(item: &Item) -> Option<(&Visibility, Declared<'_>)> {
    let (visibility, attributes, keyword, name) = match item {
        Item::Const(item) => (&item.vis, &item.attrs, "const", &item.ident),
        Item::Enum(item) => (&item.vis, &item.attrs, "enum", &item.ident),
        Item::Fn(item) => (&item.vis, &item.attrs, "fn", &item.sig.ident),
        Item::Static(item) => (&item.vis, &item.attrs, "static", &item.ident),
        Item::Struct(item) => (&item.vis, &item.attrs, "struct", &item.ident),
        Item::Trait(item) => (&item.vis, &item.attrs, "trait", &item.ident),
        Item::Type(item) => (&item.vis, &item.attrs, "type", &item.ident),
        Item::Union(item) => (&item.vis, &item.attrs, "union", &item.ident),
        _ => return None,
    };
    Some((
        visibility,
        Declared {
            attributes,
            keyword,
            name,
        },
    ))
}

fn declared_in_impl(item: &ImplItem) -> Option<(&Visibility, Declared<'_>)> {
    let (visibility, attributes, keyword, name) = match item {
        ImplItem::Const(item) => (&item.vis, &item.attrs, "const", &item.ident),
        ImplItem::Fn(item) => (&item.vis, &item.attrs, "fn", &item.sig.ident),
        ImplItem::Type(item) => (&item.vis, &item.attrs, "type", &item.ident),
        _ => return None,
    };
    Some((
        visibility,
        Declared {
            attributes,
            keyword,
            name,
        },
    ))
}

fn declared_in_trait(item: &TraitItem) -> Option<Declared<'_>> {
    let (attributes, keyword, name) = match item {
        TraitItem::Const(item) => (&item.attrs, "const", &item.ident),
        TraitItem::Fn(item) => (&item.attrs, "fn", &item.sig.ident),
        TraitItem::Type(item) => (&item.attrs, "type", &item.ident),
        _ => return None,
    };
    Some(Declared {
        attributes,
        keyword,
        name,
    })
}

fn is_public(visibility: &Visibility) -> bool {
    matches!(visibility, Visibility::Public(_))
}

fn is_documentation(attribute: &Attribute) -> bool {
    matches!(&attribute.meta, Meta::NameValue(name_value) if name_value.path.is_ident("doc"))
}

/// Whether the item is hidden from the documentation or compiled for tests only.
fn is_exempt(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        arguments_of(attribute, "doc").is_some_and(names_hidden)
            || arguments_of(attribute, "cfg").is_some_and(is_test_alone)
    })
}

fn arguments_of<'attribute>(attribute: &'attribute Attribute, name: &str) -> Option<&'attribute TokenStream> {
    match &attribute.meta {
        Meta::List(list) if list.path.is_ident(name) => Some(&list.tokens),
        _ => None,
    }
}

fn names_hidden(arguments: &TokenStream) -> bool {
    arguments
        .clone()
        .into_iter()
        .any(|argument| matches!(argument, TokenTree::Ident(name) if name == "hidden"))
}

fn is_test_alone(arguments: &TokenStream) -> bool {
    let mut arguments = arguments.clone().into_iter();
    matches!(
        (arguments.next(), arguments.next()),
        (Some(TokenTree::Ident(name)), None) if name == "test"
    )
}
