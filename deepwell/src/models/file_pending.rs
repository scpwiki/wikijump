use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "file_pending")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pending_file_id: i64,

    #[sea_orm(column_type = "Text")]
    pub s3_path: String,

    #[sea_orm(column_type = "Text")]
    pub presign_url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::file::Entity",
        from = "Column::FileId",
        to = "super::file::Column::FileId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    File,
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
