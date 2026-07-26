use crate::parse::RelationSettings;
use crate::util::make_ident;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::token::Pub;
use syn::{Ident, Type, Visibility};

pub fn expand_stream(
    RelationSettings {
        struct_name,
        field_name,
        dest: (dest_name, dest_type),
        from: (from_name, from_type),
        data_type,
        create_fn,
        remove_fn,
        define_struct,
    }: RelationSettings,
) -> TokenStream {
    let error_name = make_ident(format!("{}Relation", struct_name));

    let create_struct = make_ident(format!("Create{}", struct_name));
    let remove_struct = make_ident(format!("Remove{}", struct_name));

    let remove_method = make_ident(format!("remove_{}", field_name));

    let create_struct = match data_type {
        Some(data_type) => quote! {
            #[derive(Deserialize, Debug, Clone)]
            pub struct #create_struct {
                pub #dest_name: i64,
                pub #from_name: i64,
                pub metadata: #data_type,
                pub created_by: i64,
            }
        },
        None => quote! {
            #[derive(Deserialize, Debug, Clone)]
            pub struct #create_struct {
                pub #dest_name: i64,
                pub #from_name: i64,
                pub created_by: i64,
            }
        },
    };

    let create_method_impl = {
        let (vis, suffix) = if create_fn {
            (public(), "")
        } else {
            (private(), "_inner")
        };

        let create_method = make_ident(format!("create_{field_name}{suffix}"));

        quote! {
            #vis async fn #create_method(
                ctx: &ServiceContext<'_>,
                #create_struct {
                    #dest_name,
                    #from_name,
                    metadata,
                    created_by,
                }: #create_struct,
            ) -> Result<()> {
                let make_error = || Error::new(
                    concat!("failed to create ", stringify!(#struct_name)),
                    ErrorType::#error_name,
                );

                Self::create(
                    ctx,
                    RelationType::#struct_name,
                    RelationObject::#dest_type(#dest_name),
                    RelationObject::#from_type(#from_name),
                    created_by,
                    metadata,
                )
                .await
                .or_raise(make_error)?;

                Ok(())
            }
        }
    };

    let remove_struct = quote! {
        #[derive(Deserialize, Debug, Copy, Clone)]
        pub struct #remove_struct {
            pub #dest_name: i64,
            pub #from_name: i64,
            pub removed_by: i64,
        }
    };

    let remove_method_impl = quote! {
        pub async fn #remove_method(
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
                    relation_type: RelationType::#struct_name,
                    dest: RelationObject::#dest_type(#dest_name),
                    from: RelationObject::#from_type(#from_name),
                },
                removed_by,
            ).await
        }
    };

    todo!()
}

fn generate_get_methods(
    field_name: &str,
    struct_name: &str,
    dest_name: &Ident,
    dest_type: &Type,
    from_name: &Ident,
    from_type: &Type,
) -> TokenStream {
    let get_struct = make_ident(format!("Get{}", struct_name));

    let get_method = make_ident(format!("get_{}", field_name));
    let get_optional_method = make_ident(format!("get_optional_{}", field_name));
    let exists_method = make_ident(format!("{}_exists", field_name));
    let get_history_method = make_ident(format!("get_{}_history", field_name));
    let get_entries_method = make_ident(format!("get_{}_entries", field_name));

    quote! {
        pub async fn #get_method(
            ctx: &ServiceContext<'_>,
            #get_struct {
                #dest_name,
                #from_name,
            }: #get_struct,
        ) -> Result<RelationModel> {
            Self::get(
                RelationReference::Relationship {
                    relation_type: RelationType::#struct_name,
                    dest: RelationObject::#dest_type(#dest_name),
                    from: RelationObject::#from_type(#from_name),
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
                    relation_type: RelationType::#struct_name,
                    dest: RelationObject::#dest_type(#dest_name),
                    from: RelationObject::#from_type(#from_name),
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
                    relation_type: RelationType::#struct_name,
                    dest: RelationObject::#dest_type(#dest_name),
                    from: RelationObject::#from_type(#from_name),
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
                RelationType::#field_name,
                RelationObject::#dest_type(#dest_name),
                RelationObject::#from_type(#from_name),
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
                RelationType::#field_name,
                object,
                direction,
            )
            .await
        }
    }
    .into()
}

#[inline]
fn public() -> Visibility {
    Visibility::Public(Pub(Span::call_site()))
}

#[inline]
fn private() -> Visibility {
    Visibility::Inherited
}
