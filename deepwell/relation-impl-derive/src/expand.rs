use crate::parse::RelationSettings;
use crate::util::make_ident;
use proc_macro::TokenStream;
use quote::quote;

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
    let get_struct = make_ident(format!("Get{}", struct_name));
    let remove_struct = make_ident(format!("Remove{}", struct_name));

    let get_method = make_ident(format!("get_{}", field_name));
    let get_optional_method = make_ident(format!("get_optional_{}", field_name));
    let get_history_method = make_ident(format!("get_{}_history", field_name));
    let get_entries_method = make_ident(format!("get_{}_entries", field_name));
    let exists_method = make_ident(format!("{}_exists", field_name));
    let remove_method = make_ident(format!("remove_{}", field_name));

    quote! {
        impl RelationService {
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

            // TODO paginate
            #[allow(dead_code)] // TEMP
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
            #[allow(dead_code)] // TEMP
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
    }
    .into()
}
