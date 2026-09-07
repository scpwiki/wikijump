//! Dummy version of the infrastructure used by `RelationService`.
//! For the `trybuild` tests.

pub use serde::{Serialize, Deserialize};

use serde_json::Value as JsonValue;
use time::OffsetDateTime;

#[derive(Debug)]
pub struct Error;

#[derive(Debug)]
pub struct ServiceContext<'a>;

#[derive(Debug)]
pub struct RelationService;

impl RelationService {}

#[derive(Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RelationType {
    SiteUser,
    SiteBan,
    SiteApplication,
    SiteMember,
    PageStar,
    PageWatch,
    PageAttribution,
    UserFollow,
    UserContact,
    UserContactRequest,
    UserBlock,
    UserBotOwner,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RelationObject {
    Site(i64),
    User(i64),
    Page(i64),
    File(i64),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RelationObjectType {
    Site,
    User,
    Page,
    File,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RelationObjectTypes {
    pub dest: RelationObjectType,
    pub from: RelationObjectType,
}

#[derive(Debug)]
pub struct RelationModel {
    pub relation_id: i64,
    pub relation_type: RelationType,
    pub dest_type: RelationObjectType,
    pub dest_id: i64,
    pub from_type: RelationObjectType,
    pub from_id: i64,
    pub metadata: JsonValue,
    pub created_by: i64,
    pub created_at: OffsetDateTime,
    pub overwritten_by: Option<i64>,
    pub overwritten_at: Option<OffsetDateTime>,
    pub deleted_by: Option<i64>,
    pub deleted_at: Option<OffsetDateTime>,
}
