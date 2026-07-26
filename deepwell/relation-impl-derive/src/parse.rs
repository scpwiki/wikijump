use crate::case::pascal_to_snake_case;
use crate::types::GenerateMethod;
use crate::util::*;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, Type};

pub struct RelationSettings {
    relation_name: Ident,
    field_name: Ident,
    dest: (Ident, Type),
    from: (Ident, Type),
    data_type: Option<Type>,
    create_fn: GenerateMethod,
    remove_fn: GenerateMethod,
    define_struct: bool,
}

impl Parse for RelationSettings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut dest = None;
        let mut from = None;
        let mut data_type = None;
        let mut create_fn = None;
        let mut remove_fn = None;
        let mut define_struct = None;

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
                    error_if_set!(dest);
                    let field_name: Ident = input.parse()?;
                    let _: Token![:] = input.parse()?;
                    let field_type: Type = input.parse()?;
                    dest = Some((field_name, field_type));
                }

                // Define the "from" name and type
                //
                //  from => blocking_user: User
                "from" => {
                    error_if_set!(from);
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
                    error_if_set!(data_type);
                    let t_type: Type = input.parse()?;
                    data_type = Some(process_type(t_type));
                }

                // Designate how the create method should be generated
                // This key is optional, default is "true".
                //
                //  create_fn => false
                //
                // The following values are accepted:
                // * true  - Implement a public "create_{name}" method.
                // * false - Do not implement a create method,
                //           caller must implement "create_{name}".
                // * fn    - Implement a private "create_{name}_inner method,
                //           caller must implement "create_{name}".
                "create_fn" => {
                    error_if_set!(create_fn);
                    let token = input.lookahead1();
                    let setting = GenerateMethod::parse(input, token)?;
                    create_fn = Some(setting);
                }

                // Designate how the remove method should be generated
                // This key is optional, default is "true".
                // Same accepted values as "create_fn".
                //
                //  remove_fn => true
                "remove_fn" => {
                    error_if_set!(remove_fn);
                    let token = input.lookahead1();
                    let setting = GenerateMethod::parse(input, token)?;
                    remove_fn = Some(setting);
                }

                _ => return Err(make_error(format!("invalid key in macro: {key}"))),
            }
        }

        // Gather fields and return

        // Required fields
        let (relation_name, field_name) =
            name.ok_or_else(|| make_error("no 'name' argument passed"))?;
        let dest = dest.ok_or_else(|| make_error("no 'dest' argument passed"))?;
        let from = from.ok_or_else(|| make_error("no 'from' argument passed"))?;
        // Default fields
        let data_type = data_type.unwrap_or(None);
        let create_fn = create_fn.unwrap_or(GenerateMethod::default());
        let remove_fn = remove_fn.unwrap_or(GenerateMethod::default());
        let define_struct = define_struct.unwrap_or(true);

        Ok(RelationSettings {
            relation_name,
            field_name,
            dest,
            from,
            data_type,
            create_fn,
            remove_fn,
            define_struct,
        })
    }
}
