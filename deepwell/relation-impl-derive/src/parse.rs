//! Parsing macro generation settings.
//!
//! This defines `RelationSettings`, which drives code
//! generation in `expand.rs`.
//!
//! See code comments to for further details on each
//! accepted argument.

use crate::case::pascal_to_snake_case;
use crate::types::{GenerateMethod, RelationObjectType};
use crate::util::*;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Type};

pub struct RelationSettings {
    pub struct_name: String,
    pub field_name: String,
    pub dest: (Ident, RelationObjectType),
    pub from: (Ident, RelationObjectType),
    pub data_type: Option<Type>,
    pub create_fn: GenerateMethod,
    pub remove_fn: GenerateMethod,
}

impl Parse for RelationSettings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut dest = None;
        let mut from = None;
        let mut data_type = None;
        let mut create_fn = None;
        let mut remove_fn = None;

        macro_rules! error_if_set {
            ($field:expr) => {
                if $field.is_some() {
                    return Err(make_error(format!(
                        "argument '{}' present multiple times",
                        stringify!($field),
                    )));
                }
            };
        }

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
                    error_if_set!(name);
                    let struct_name_ident: Ident = input.parse()?;
                    let struct_name = struct_name_ident.to_string();
                    let field_name = pascal_to_snake_case(&struct_name);
                    name = Some((struct_name, field_name));
                }

                // Define the "dest" name and type
                //
                //  dest => blocked_user: User
                "dest" => {
                    error_if_set!(dest);
                    let field_name: Ident = input.parse()?;
                    let _: Token![:] = input.parse()?;
                    let field_type = RelationObjectType::parse(input)?;
                    dest = Some((field_name, field_type));
                }

                // Define the "from" name and type
                //
                //  from => blocking_user: User
                "from" => {
                    error_if_set!(from);
                    let field_name: Ident = input.parse()?;
                    let _: Token![:] = input.parse()?;
                    let field_type = RelationObjectType::parse(input)?;
                    from = Some((field_name, field_type));
                }

                // Define the associated metadata type for this relation
                // This key is optional, by default no extra metadata is included
                //
                //  data => UserBlockData
                "data" => {
                    error_if_set!(data_type);
                    let t_type: Type = input.parse()?;
                    data_type = Some(process_type(t_type));
                }

                // Designate how the create method should be generated
                // This key is optional, default is "true".
                //
                // If set to "extern", then neither the method nor the
                // corresponding struct are generated.
                //
                //  create_fn => extern
                "create_fn" => {
                    error_if_set!(create_fn);
                    let option = GenerateMethod::parse(input)?;
                    create_fn = Some(option);
                }

                // Designate how the remove method should be generated
                // This key is optional, default is "true".
                //
                // If set to "extern", then neither the method nor the
                // corresponding struct are generated.
                //
                //  remove_fn => true
                "remove_fn" => {
                    error_if_set!(remove_fn);
                    let option = GenerateMethod::parse(input)?;
                    remove_fn = Some(option);
                }

                _ => return Err(make_error(format!("invalid key in macro: {key}"))),
            }

            if input.is_empty() {
                // Trailing comma is optional
                break;
            }

            let _: Token![,] = input.parse()?;
        }

        // Gather fields and return

        // Required fields
        let (struct_name, field_name) =
            name.ok_or_else(|| make_error("no 'name' argument passed"))?;
        let dest = dest.ok_or_else(|| make_error("no 'dest' argument passed"))?;
        let from = from.ok_or_else(|| make_error("no 'from' argument passed"))?;
        // Default fields
        let data_type = data_type.unwrap_or(None);
        let create_fn = create_fn.unwrap_or(GenerateMethod::default());
        let remove_fn = remove_fn.unwrap_or(GenerateMethod::default());

        Ok(RelationSettings {
            struct_name,
            field_name,
            dest,
            from,
            data_type,
            create_fn,
            remove_fn,
        })
    }
}
