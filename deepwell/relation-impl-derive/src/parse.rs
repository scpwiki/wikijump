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
        let mut dest: Option<()> = None;
        let mut from: Option<()> = None;
        let mut data_type: Option<()> = None;
        let mut define_create: Option<bool> = None;
        let mut define_struct: Option<bool> = None;

        // Iterate through all entries in the macro's arguments
        loop {
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
                    let field_name = Ident::new(&field_name_str, Span::call_site());
                    name = Some((relation_name, field_name));
                }
                // Define the "dest" name and type
                //
                //  dest => blocked_user: User
                "dest" => {
                    todo!();
                }
                // Define the "from" name and type
                //
                //  from => blocking_user: User
                "from" => {
                    todo!();
                }
                // TODO
                "data" => {
                    todo!();
                }
                _ => return Err(make_error(format!("invalid key in macro: {key}"))),
            }
        }
    }
}

#[inline]
fn make_error(message: impl Display) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}
