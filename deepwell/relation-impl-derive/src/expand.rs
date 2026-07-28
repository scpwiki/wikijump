use crate::parse::RelationSettings;
use crate::types::{GenerateMethod, RelationObjectType};
use crate::util::make_ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub fn expand_stream(
    RelationSettings {
        struct_name,
        field_name,
        dest: (dest_name, dest_type),
        from: (from_name, from_type),
        data_type,
        create_fn,
        remove_fn,
    }: RelationSettings,
) -> TokenStream {
    // Build context for each helper

    let relation_type = {
        let struct_ident = make_ident(&struct_name);
        quote! { RelationType::#struct_ident }
    };

    let context = GenerationContext {
        field_name: &field_name,
        struct_name: &struct_name,
        relation_type: &relation_type,
        dest_name: &dest_name,
        dest_type,
        from_name: &from_name,
        from_type,
    };

    // Generate each code section

    let GeneratedDefinitions {
        struct_def: get_struct_def,
        method_impl: get_method_impl,
    } = generate_get_methods(context);

    let (create_struct_def, create_method_impl) =
        match generate_create_defs(context, data_type.as_ref(), create_fn) {
            // Enabled, set both
            Some(GeneratedDefinitions {
                struct_def,
                method_impl,
            }) => (Some(struct_def), Some(method_impl)),

            // Disabled, don't insert anything
            None => (None, None),
        };

    let (remove_struct_def, remove_method_impl) =
        match generate_remove_defs(context, remove_fn) {
            // Enabled, set both
            Some(GeneratedDefinitions {
                struct_def,
                method_impl,
            }) => (Some(struct_def), Some(method_impl)),

            // Disabled, don't insert anything
            None => (None, None),
        };

    quote! {
        impl RelationService {
            #get_method_impl
            #create_method_impl
            #remove_method_impl
        }

        #get_struct_def
        #create_struct_def
        #remove_struct_def
    }
}

fn generate_get_methods(
    GenerationContext {
        field_name,
        struct_name,
        relation_type,
        dest_name,
        dest_type,
        from_name,
        from_type,
    }: GenerationContext,
) -> GeneratedDefinitions {
    let get_struct = make_ident(format!("Get{}", struct_name));

    let get_method = make_ident(format!("get_{}", field_name));
    let get_optional_method = make_ident(format!("get_optional_{}", field_name));
    let exists_method = make_ident(format!("{}_exists", field_name));
    let get_history_method = make_ident(format!("get_{}_history", field_name));
    let get_entries_method = make_ident(format!("get_{}_entries", field_name));

    let struct_def = quote! {
        #[derive(Deserialize, Debug, Copy, Clone)]
        pub struct #get_struct {
            pub #dest_name: i64,
            pub #from_name: i64,
        }
    };

    let method_impl = quote! {
        pub async fn #get_method(
            ctx: &ServiceContext<'_>,
            #get_struct {
                #dest_name,
                #from_name,
            }: #get_struct,
        ) -> Result<RelationModel> {
            Self::get(
                ctx,
                RelationReference::Relationship {
                    relation_type: #relation_type,
                    dest: #dest_type(#dest_name),
                    from: #from_type(#from_name),
                },
            )
            .await
        }

        pub async fn #get_optional_method(
            ctx: &ServiceContext<'_>,
            #get_struct {
                #dest_name,
                #from_name,
            }: #get_struct,
        ) -> Result<Option<RelationModel>> {
            Self::get_optional(
                ctx,
                RelationReference::Relationship {
                    relation_type: #relation_type,
                    dest: #dest_type(#dest_name),
                    from: #from_type(#from_name),
                },
            )
            .await
        }

        pub async fn #exists_method(
            ctx: &ServiceContext<'_>,
            #get_struct {
                #dest_name,
                #from_name,
            }: #get_struct,
        ) -> Result<bool> {
            Self::exists(
                ctx,
                RelationReference::Relationship {
                    relation_type: #relation_type,
                    dest: #dest_type(#dest_name),
                    from: #from_type(#from_name),
                },
            )
            .await
        }

        // TODO paginate
        pub async fn #get_history_method(
            ctx: &ServiceContext<'_>,
            #get_struct {
                #dest_name,
                #from_name,
            }: #get_struct,
        ) -> Result<Vec<RelationModel>> {
            Self::get_history(
                ctx,
                #relation_type,
                #dest_type(#dest_name),
                #from_type(#from_name),
            )
            .await
        }

        // TODO paginate
        pub async fn #get_entries_method(
            ctx: &ServiceContext<'_>,
            object: RelationObject,
            direction: RelationDirection,
        ) -> Result<Vec<RelationModel>> {
            Self::get_entries(
                ctx,
                #relation_type,
                object,
                direction,
            )
            .await
        }
    };

    GeneratedDefinitions {
        struct_def,
        method_impl,
    }
}

fn generate_create_defs(
    GenerationContext {
        field_name,
        struct_name,
        relation_type,
        dest_name,
        dest_type,
        from_name,
        from_type,
    }: GenerationContext,
    data_type: Option<&Type>,
    create_fn: GenerateMethod,
) -> Option<GeneratedDefinitions> {
    let (vis, suffix) = create_fn.vis_and_suffix()?;
    let mut create_struct = make_ident(format!("Create{}", struct_name));
    let create_struct_def;
    let mut create_struct_lifetime = None;
    let mut create_struct_inner_def = None;

    match data_type {
        Some(data_type) => {
            create_struct_def = quote! {
                #[derive(Deserialize, Debug, Clone)]
                pub struct #create_struct {
                    pub #dest_name: i64,
                    pub #from_name: i64,
                    pub metadata: #data_type,
                    pub created_by: i64,
                }
            };

            if matches!(create_fn, GenerateMethod::Private) {
                // overwrite since this is the name to use later for the actual fn impl
                create_struct = make_ident(format!("Create{}Inner", struct_name));
                create_struct_lifetime = Some(quote! { <'_> });
                create_struct_inner_def = Some(quote! {
                    #[derive(Debug, Clone)]
                    struct #create_struct<'a> {
                        pub #dest_name: i64,
                        pub #from_name: i64,
                        pub metadata: &'a #data_type,
                        pub created_by: i64,
                    }
                });
            }
        }
        None => {
            create_struct_def = quote! {
                #[derive(Deserialize, Debug, Clone)]
                pub struct #create_struct {
                    pub #dest_name: i64,
                    pub #from_name: i64,
                    pub created_by: i64,
                }
            };
        }
    };

    let create_method_impl = {
        let error_name = make_ident(format!("{}Relation", struct_name));
        let method_name = make_ident(format!("create_{field_name}{suffix}"));

        let struct_decompose;
        let create_call;
        match data_type {
            // Include metadata field
            Some(_) => {
                struct_decompose = quote! {
                    #create_struct {
                        #dest_name,
                        #from_name,
                        metadata,
                        created_by,
                    }: #create_struct #create_struct_lifetime
                };

                create_call = quote! {
                    Self::create(
                        ctx,
                        #relation_type,
                        #dest_type(#dest_name),
                        #from_type(#from_name),
                        created_by,
                        metadata,
                    )
                    .await
                    .or_raise(make_error)?;
                };
            }

            // No metadata field
            //
            // Because the create() method always takes the same
            // number of arguments, that means we have to pass
            // in the empty data ourselves.
            None => {
                struct_decompose = quote! {
                    #create_struct {
                        #dest_name,
                        #from_name,
                        created_by,
                    }: #create_struct
                };

                create_call = quote! {
                    Self::create(
                        ctx,
                        #relation_type,
                        #dest_type(#dest_name),
                        #from_type(#from_name),
                        created_by,
                        &(),
                    )
                    .await
                    .or_raise(make_error)?;
                };
            }
        };

        quote! {
            #vis async fn #method_name(
                ctx: &ServiceContext<'_>,
                #struct_decompose,
            ) -> Result<()> {
                let make_error = || Error::new(
                    concat!("failed to create ", stringify!(#struct_name)),
                    ErrorType::#error_name,
                );

                #create_call

                Ok(())
            }
        }
    };

    Some(GeneratedDefinitions {
        struct_def: quote! {
            #create_struct_def
            #create_struct_inner_def
        },
        method_impl: create_method_impl,
    })
}

fn generate_remove_defs(
    GenerationContext {
        field_name,
        struct_name,
        relation_type,
        dest_name,
        dest_type,
        from_name,
        from_type,
    }: GenerationContext,
    remove_fn: GenerateMethod,
) -> Option<GeneratedDefinitions> {
    let (vis, suffix) = remove_fn.vis_and_suffix()?;
    let remove_struct = make_ident(format!("Remove{}", struct_name));
    let remove_struct_def = quote! {
        #[derive(Deserialize, Debug, Copy, Clone)]
        pub struct #remove_struct {
            pub #dest_name: i64,
            pub #from_name: i64,
            pub removed_by: i64,
        }
    };

    let remove_method_impl = {
        let method_name = make_ident(format!("remove_{field_name}{suffix}"));

        quote! {
            #vis async fn #method_name(
                ctx: &ServiceContext<'_>,
                #remove_struct {
                    #dest_name,
                    #from_name,
                    removed_by,
                }: #remove_struct
            ) -> Result<RelationModel> {
                Self::remove(
                    ctx,
                    RelationReference::Relationship {
                        relation_type: #relation_type,
                        dest: #dest_type(#dest_name),
                        from: #from_type(#from_name),
                    },
                    removed_by,
                ).await
            }
        }
    };

    Some(GeneratedDefinitions {
        struct_def: remove_struct_def,
        method_impl: remove_method_impl,
    })
}

// Helpers

#[derive(Copy, Clone)]
struct GenerationContext<'a> {
    field_name: &'a str,
    struct_name: &'a str,
    relation_type: &'a TokenStream,
    dest_name: &'a Ident,
    dest_type: RelationObjectType,
    from_name: &'a Ident,
    from_type: RelationObjectType,
}

struct GeneratedDefinitions {
    struct_def: TokenStream,
    method_impl: TokenStream,
}
