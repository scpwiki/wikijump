use crate::case::pascal_to_snake_case;
use proc_macro2::Span;
use std::fmt::Display;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Type};

pub struct RelationSettings {
    relation_name: Ident,
    field_name: Ident,
    dest: (Ident, Type),
    from: (Ident, Type),
    data_type: Option<Type>,
    define_create: bool,
    define_struct: bool,
}

impl Parse for RelationSettings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut dest = None;
        let mut from = None;
        let mut data_type = None;
        let mut define_create = true;
        let mut define_struct = true;

        // Iterate through all entries in the macro's arguments
        while !input.is_empty() {
            // Parse one "key => value" entry.
            // The processing depends on what the key is.
            let key = input.parse::<Ident>()?.to_string();
            let _: Token![=>] = input.parse()?;

            match key.as_str() {
                // The name of the relation, in PascalCase.
                // The macro converts it to snake_case as needed.
                //
                //  name => UserBlock
                "name" => {
                    let relation_name: Ident = input.parse()?;
                    let relation_name_str = relation_name.to_string();
                    let field_name_str = pascal_to_snake_case(&relation_name_str);
                    let field_name = make_ident(field_name_str);
                    name = Some((relation_name, field_name));
                }
                // Define the "dest" name and type
                //
                //  dest => blocked_user: User
                "dest" => {
                    let field_name: Ident = input.parse()?;
                    let _: Token![:] = input.parse()?;
                    let field_type: Type = input.parse()?;
                    dest = Some((field_name, field_type));
                }
                // Define the "from" name and type
                //
                //  from => blocking_user: User
                "from" => {
                    let field_name: Ident = input.parse()?;
                    let _: Token![:] = input.parse()?;
                    let field_type: Type = input.parse()?;
                    from = Some((field_name, field_type));
                }
                // Define the associated metadata type for this relation
                // This key is optional, by default no extra metadata is included
                //
                //  data => UserBlockData
                "data" => {
                    let t_type: Type = input.parse()?;
                    data_type = Some(t_type);
                }
                _ => return Err(make_error(format!("invalid key in macro: {key}"))),
            }
        }

        // Gather fields and return

        let (relation_name, field_name) =
            name.ok_or_else(|| make_error("no 'name' argument passed"))?;
        let dest = dest.ok_or_else(|| make_error("no 'dest' argument passed"))?;
        let from = from.ok_or_else(|| make_error("no 'from' argument passed"))?;

        Ok(RelationSettings {
            relation_name,
            field_name,
            dest,
            from,
            data_type,
            define_create,
            define_struct,
        })
    }
}

#[inline]
fn make_ident(value: impl AsRef<str>) -> Ident {
    Ident::new(value.as_ref(), Span::call_site())
}

#[inline]
fn make_error(message: impl Display) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}
