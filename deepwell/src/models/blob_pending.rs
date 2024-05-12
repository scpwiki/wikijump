use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "blob_pending")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pending_blob_id: i64,

    #[sea_orm(column_type = "Text")]
    pub s3_path: String,

    #[sea_orm(column_type = "Text")]
    pub presign_url: String,
}

impl ActiveModelBehavior for ActiveModel {}
