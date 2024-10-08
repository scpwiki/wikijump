//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "page_connection")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub from_page_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub to_page_id: i64,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub connection_type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: TimeDateTimeWithTimeZone,
    #[serde(with = "time::serde::rfc3339::option")]
    pub updated_at: Option<TimeDateTimeWithTimeZone>,
    pub count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::page::Entity",
        from = "Column::FromPageId",
        to = "super::page::Column::PageId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Page2,
    #[sea_orm(
        belongs_to = "super::page::Entity",
        from = "Column::ToPageId",
        to = "super::page::Column::PageId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Page1,
}

impl ActiveModelBehavior for ActiveModel {}
