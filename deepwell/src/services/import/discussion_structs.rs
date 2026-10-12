//! A complete, provenance-bound current page discussion snapshot.

use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportDiscussion {
    pub site_id: i64,
    pub page_id: i64,
    pub source: DiscussionSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionSource {
    pub site: String,
    pub fullname: String,
    pub reference: String,
    pub evidence_sha256: String,
    #[serde(with = "time::serde::rfc3339")]
    pub observed_at: OffsetDateTime,
    pub category: DiscussionCategory,
    pub thread: DiscussionThread,
    pub actors: Vec<DiscussionActor>,
    pub posts: Vec<DiscussionPost>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionCategory {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub group_name: String,
    pub group_description: String,
    pub group_visible: bool,
    pub group_sort_index: i32,
    pub sort_index: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionThread {
    pub id: i64,
    pub title: String,
    pub description: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub creator: DiscussionCreator,
    pub post_count: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiscussionCreator {
    System,
    User { user_id: i64 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionActor {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionPost {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub user_id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub edit: Option<DiscussionPostEdit>,
    pub title: String,
    pub wikitext: String,
    /// Source identities explicitly observed in the rendered post body.
    pub referenced_user_ids: Vec<i64>,
    #[serde(default)]
    pub observed_users: Vec<DiscussionBodyUser>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionPostEdit {
    pub user_id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize)]
pub struct ImportDiscussionOutput {
    pub import_run_id: i64,
    pub forum_thread_id: i64,
    pub manifest_sha256: String,
    pub created_thread: bool,
    pub created_posts: usize,
    pub created_revisions: usize,
    pub unchanged: bool,
}

/// Identity retained in this source revision's cached rendered user reference.
/// It may predate a rename visible in the current post author metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiscussionBodyUser {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub avatar_url: Option<String>,
}
