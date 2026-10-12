//! Import an observed page discussion without granting source actors accounts.

use super::ImportService;
#[cfg(test)]
use super::discussion_structs::DiscussionActor;
use super::discussion_structs::{
    DiscussionCreator, DiscussionSource, ImportDiscussion, ImportDiscussionOutput,
};
use crate::constants::SYSTEM_USER_ID;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::{
    forum_category, forum_group, forum_post, forum_post_revision, forum_thread, page,
    user, wikidot_user,
};
use crate::services::ServiceContext;
use crate::services::TextService;
use crate::services::forum_post_revision::ForumPostRevisionService;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait,
    FromQueryResult, IntoActiveModel, QueryFilter, QuerySelect, Set, Statement, Value,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn invalid(message: impl Into<String>) -> crate::error::ExnError {
    Error::new(message, ErrorType::DatabaseImport).into()
}

fn source_id(id: i64) -> bool {
    (1..=i64::from(i32::MAX)).contains(&id)
}

impl DiscussionSource {
    pub(super) fn validate(&self) -> Result<()> {
        if self.posts.len() > 1000 || self.thread.post_count != self.posts.len() {
            return Err(invalid(
                "discussion snapshot must contain its complete bounded post inventory",
            ));
        }
        if !source_id(self.thread.id)
            || !source_id(self.category.id)
            || self.thread.title.is_empty()
            || self.category.name.is_empty()
            || self.category.group_name.is_empty()
            || self.category.sort_index < 0
            || self.category.group_sort_index < 0
        {
            return Err(invalid("invalid source discussion or category identity"));
        }
        let url = reqwest::Url::parse(&self.reference)
            .map_err(|_| invalid("invalid discussion source reference"))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str() != Some(format!("{}.wikidot.com", self.site).as_str())
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !url
                .path()
                .starts_with(&format!("/forum/t-{}/", self.thread.id))
        {
            return Err(invalid(
                "discussion reference does not identify the source site and thread",
            ));
        }
        if hex::decode(&self.evidence_sha256)
            .ok()
            .is_none_or(|bytes| bytes.len() != 32)
        {
            return Err(invalid("discussion evidence must have a SHA-256 identity"));
        }
        let mut actors = BTreeSet::new();
        for actor in &self.actors {
            if !source_id(actor.id)
                || !actors.insert(actor.id)
                || actor.name.is_empty()
                || actor.slug.is_empty()
                || !actor
                    .slug
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            {
                return Err(invalid("invalid or duplicate source actor identity"));
            }
        }
        let mut used_actors = BTreeSet::new();
        if let DiscussionCreator::User { user_id } = self.thread.creator {
            used_actors.insert(user_id);
        }
        let posts = self
            .posts
            .iter()
            .map(|p| (p.id, p))
            .collect::<BTreeMap<_, _>>();
        if posts.len() != self.posts.len() {
            return Err(invalid("duplicate source post identity"));
        }
        let mut source_bytes = 0usize;
        for post in &self.posts {
            source_bytes += post.wikitext.len();
            if !source_id(post.id) {
                return Err(invalid("invalid source post identity"));
            }
            used_actors.insert(post.user_id);
            used_actors.extend(post.referenced_user_ids.iter().copied());
            let mut observed = BTreeSet::new();
            for user in &post.observed_users {
                if !observed.insert(user.id)
                    || !post.referenced_user_ids.contains(&user.id)
                    || user.name.is_empty()
                    || user.slug.is_empty()
                    || !user.slug.bytes().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-'
                    })
                {
                    return Err(invalid("invalid source revision user observation"));
                }
                if let Some(avatar) = &user.avatar_url {
                    let url = reqwest::Url::parse(avatar)
                        .map_err(|_| invalid("invalid source avatar URL"))?;
                    if !matches!(url.scheme(), "http" | "https")
                        || url.host_str() != Some("www.wikidot.com")
                        || url.path() != "/avatar.php"
                        || !url.username().is_empty()
                        || url.password().is_some()
                        || url
                            .query_pairs()
                            .find(|(key, _)| key == "userid")
                            .is_none_or(|(_, value)| value != user.id.to_string())
                    {
                        return Err(invalid(
                            "source avatar does not identify the observed user",
                        ));
                    }
                }
            }
            if let Some(edit) = &post.edit {
                if edit.created_at < post.created_at {
                    return Err(invalid("source edit predates its post"));
                }
                used_actors.insert(edit.user_id);
            }
            let mut ancestry = BTreeSet::from([post.id]);
            let mut parent = post.parent_id;
            while let Some(id) = parent {
                if !ancestry.insert(id) {
                    return Err(invalid("source post graph contains a cycle"));
                }
                let Some(ancestor) = posts.get(&id) else {
                    return Err(invalid(
                        "source post parent is outside the complete inventory",
                    ));
                };
                parent = ancestor.parent_id;
            }
        }
        if source_bytes > 4 * 1024 * 1024 || actors != used_actors {
            return Err(invalid(
                "source discussion has an incomplete actor inventory or exceeds its source budget",
            ));
        }
        Ok(())
    }
}

async fn execute(ctx: &ServiceContext<'_>, sql: &str, values: Vec<Value>) -> Result<()> {
    ctx.transaction()
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            values,
        ))
        .await
        .or_raise(|| {
            Error::new(
                "discussion import database operation failed",
                ErrorType::DatabaseImport,
            )
        })?;
    Ok(())
}

#[derive(FromQueryResult)]
struct LastObservation {
    observed_at: time::OffsetDateTime,
    manifest_sha256: Vec<u8>,
}
#[derive(FromQueryResult)]
struct MirrorSnapshot {
    source_site: String,
    source_fullname: String,
    comments: i32,
}
#[derive(FromQueryResult)]
struct ImportedRun {
    import_run_id: i64,
}
#[derive(FromQueryResult)]
struct PriorPostSnapshot {
    source_has_edit: bool,
    render_identity_sha256: Vec<u8>,
}

impl ImportService {
    pub async fn add_discussion(
        ctx: &ServiceContext<'_>,
        mut input: ImportDiscussion,
        importing_user_id: i64,
    ) -> Result<ImportDiscussionOutput> {
        input.source.validate()?;
        input.source.actors.sort_by_key(|actor| actor.id);
        input.source.posts.sort_by_key(|post| post.id);
        let source = &input.source;
        let manifest_json = serde_json::to_value(source)
            .map_err(|_| invalid("cannot serialize discussion provenance"))?;
        let manifest_bytes = serde_json::to_vec(&manifest_json)
            .map_err(|_| invalid("cannot hash discussion provenance"))?;
        let manifest_hash = Sha256::digest(&manifest_bytes).to_vec();
        let evidence_hash = hex::decode(&source.evidence_sha256)
            .map_err(|_| invalid("invalid evidence identity"))?;
        execute(ctx, "SELECT pg_advisory_xact_lock(hashtextextended('wikidot-discussion-import:' || $1::TEXT, 0))", vec![input.site_id.into()]).await?;
        let page = page::Entity::find_by_id(input.page_id)
            .lock_exclusive()
            .one(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "cannot load imported discussion page",
                    ErrorType::DatabaseImport,
                )
            })?
            .filter(|p| {
                p.site_id == input.site_id && p.from_wikidot && p.deleted_at.is_none()
            })
            .ok_or_else(|| {
                invalid("discussion target is not an active imported page of this site")
            })?;
        let snapshot = MirrorSnapshot::find_by_statement(Statement::from_sql_and_values(DatabaseBackend::Postgres,
            "SELECT source_site, source_fullname, comments FROM wikidot_page_snapshot WHERE page_id=$1", [Value::from(input.page_id)]))
            .one(ctx.transaction()).await.or_raise(|| Error::new("cannot load mirror source identity", ErrorType::DatabaseImport))?
            .ok_or_else(|| invalid("discussion target has no corpus snapshot"))?;
        if snapshot.source_site != source.site
            || snapshot.source_fullname != source.fullname
            || usize::try_from(snapshot.comments).ok() != Some(source.posts.len())
        {
            return Err(invalid(
                "page and discussion source identities or current post counts differ; refresh the corpus page first",
            ));
        }
        let last = LastObservation::find_by_statement(Statement::from_sql_and_values(DatabaseBackend::Postgres,
            "SELECT r.observed_at, r.manifest_sha256 FROM wikidot_discussion_snapshot s JOIN wikidot_discussion_import_run r ON r.import_run_id=s.last_import_run_id WHERE s.page_id=$1", [Value::from(input.page_id)]))
            .one(ctx.transaction()).await.or_raise(|| Error::new("cannot inspect previous discussion observation", ErrorType::DatabaseImport))?;
        if last.as_ref().is_some_and(|last| {
            source.observed_at < last.observed_at
                || (source.observed_at == last.observed_at
                    && manifest_hash != last.manifest_sha256)
        }) {
            return Err(invalid(
                "discussion observation is older than or contradicts the current snapshot",
            ));
        }
        if page
            .discussion_thread_id
            .is_some_and(|id| id != source.thread.id)
        {
            return Err(invalid("page already points to a different discussion"));
        }
        for actor in &source.actors {
            let profile = wikidot_user::Entity::find_by_id(actor.id as i32)
                .one(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new("cannot inspect source actor", ErrorType::DatabaseImport)
                })?;
            let local = user::Entity::find_by_id(actor.id)
                .one(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new(
                        "cannot inspect actor ID collision",
                        ErrorType::DatabaseImport,
                    )
                })?;
            if (local.is_some() && profile.is_none())
                || profile.as_ref().is_some_and(|p| p.is_deleted)
            {
                return Err(invalid(
                    "source actor ID collides with a local account or a deleted source identity",
                ));
            }
        }
        let prior_thread = forum_thread::Entity::find_by_id(source.thread.id)
            .lock_exclusive()
            .one(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "cannot inspect discussion thread identity",
                    ErrorType::DatabaseImport,
                )
            })?;
        if prior_thread.as_ref().is_some_and(|t| {
            !t.from_wikidot
                || t.site_id != input.site_id
                || t.deleted_at.is_some()
                || t.page_id.is_some_and(|id| id != input.page_id)
                || t.forum_category_id != source.category.id
                || t.created_at != source.thread.created_at
                || t.created_by
                    != match source.thread.creator {
                        DiscussionCreator::System => SYSTEM_USER_ID,
                        DiscussionCreator::User { user_id } => user_id,
                    }
        }) {
            return Err(invalid(
                "source thread collides with a different, deleted, or locally owned thread",
            ));
        }
        let existing_posts = forum_post::Entity::find()
            .filter(forum_post::Column::ForumThreadId.eq(source.thread.id))
            .lock_exclusive()
            .all(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "cannot inspect existing discussion posts",
                    ErrorType::DatabaseImport,
                )
            })?;
        let wanted = source.posts.iter().map(|p| p.id).collect::<BTreeSet<_>>();
        if existing_posts.iter().any(|p| {
            !p.from_wikidot
                || p.site_id != input.site_id
                || p.deleted_at.is_some()
                || !wanted.contains(&p.forum_post_id)
        }) {
            return Err(invalid(
                "discussion import refuses unobserved removals, restores, or locally authored posts",
            ));
        }
        // Check every global post ID before creating any source graph rows.
        for post in &source.posts {
            if let Some(existing) = forum_post::Entity::find_by_id(post.id)
                .lock_exclusive()
                .one(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new("cannot inspect source post ID", ErrorType::DatabaseImport)
                })?
                && (!existing.from_wikidot
                    || existing.site_id != input.site_id
                    || existing.forum_thread_id != source.thread.id
                    || existing.user_id != post.user_id
                    || existing.created_at != post.created_at
                    || existing.deleted_at.is_some())
            {
                return Err(invalid(
                    "source post ID collides with another identity or owner",
                ));
            }
        }
        for actor in &source.actors {
            execute(
                ctx,
                "INSERT INTO known_user(user_id) VALUES($1) ON CONFLICT DO NOTHING",
                vec![actor.id.into()],
            )
            .await?;
            execute(ctx, "INSERT INTO wikidot_identity_observation(user_id,source_sha256,observed_at,name,slug,source_reference) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
                vec![actor.id.into(), manifest_hash.clone().into(), source.observed_at.into(), actor.name.clone().into(), actor.slug.clone().into(), source.reference.clone().into()]).await?;
        }
        let category = forum_category::Entity::find_by_id(source.category.id)
            .one(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "cannot inspect source discussion category",
                    ErrorType::DatabaseImport,
                )
            })?;
        let group_id = if let Some(category) = &category {
            let group = forum_group::Entity::find_by_id(category.forum_group_id)
                .one(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new(
                        "cannot inspect source forum group",
                        ErrorType::DatabaseImport,
                    )
                })?
                .ok_or_else(|| invalid("discussion category has no group"))?;
            if !category.from_wikidot
                || category.site_id != input.site_id
                || category.deleted_at.is_some()
                || !group.from_wikidot
                || group.site_id != input.site_id
                || group.deleted_at.is_some()
                || group.name != source.category.group_name
            {
                return Err(invalid(
                    "source category collides with another forum container",
                ));
            }
            group.forum_group_id
        } else {
            let groups = forum_group::Entity::find()
                .filter(forum_group::Column::SiteId.eq(input.site_id))
                .filter(forum_group::Column::Name.eq(source.category.group_name.clone()))
                .all(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new(
                        "cannot resolve source forum group",
                        ErrorType::DatabaseImport,
                    )
                })?;
            match groups.as_slice() {
                [] => {
                    forum_group::ActiveModel {
                        site_id: Set(input.site_id),
                        created_by: Set(importing_user_id),
                        name: Set(source.category.group_name.clone()),
                        description: Set(source.category.group_description.clone()),
                        visible: Set(source.category.group_visible),
                        sort_index: Set(source.category.group_sort_index),
                        from_wikidot: Set(true),
                        ..Default::default()
                    }
                    .insert(ctx.transaction())
                    .await
                    .or_raise(|| {
                        Error::new("cannot import forum group", ErrorType::DatabaseImport)
                    })?
                    .forum_group_id
                }
                [group] if group.from_wikidot && group.deleted_at.is_none() => {
                    group.forum_group_id
                }
                _ => {
                    return Err(invalid(
                        "source group has an ambiguous or local container",
                    ));
                }
            }
        };
        if category.is_none() {
            forum_category::ActiveModel {
                forum_category_id: Set(source.category.id),
                forum_group_id: Set(group_id),
                site_id: Set(input.site_id),
                created_by: Set(importing_user_id),
                name: Set(source.category.name.clone()),
                description: Set(source.category.description.clone()),
                sort_index: Set(source.category.sort_index),
                from_wikidot: Set(true),
                per_page_discussion: Set(Some(true)),
                ..Default::default()
            }
            .insert(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "cannot import discussion category",
                    ErrorType::DatabaseImport,
                )
            })?;
        }
        execute(ctx, "UPDATE forum_group SET description=$2,visible=$3,sort_index=$4 WHERE forum_group_id=$1 AND (description,visible,sort_index) IS DISTINCT FROM ($2,$3,$4)",
            vec![group_id.into(),source.category.group_description.clone().into(),source.category.group_visible.into(),source.category.group_sort_index.into()]).await?;
        execute(ctx, "UPDATE forum_category SET name=$2,description=$3,sort_index=$4,per_page_discussion=true WHERE forum_category_id=$1 AND (name,description,sort_index,per_page_discussion) IS DISTINCT FROM ($2,$3,$4,true)",
            vec![source.category.id.into(),source.category.name.clone().into(),source.category.description.clone().into(),source.category.sort_index.into()]).await?;
        let creator = match source.thread.creator {
            DiscussionCreator::System => SYSTEM_USER_ID,
            DiscussionCreator::User { user_id } => user_id,
        };
        let created_thread = prior_thread.is_none();
        if created_thread {
            forum_thread::ActiveModel {
                forum_thread_id: Set(source.thread.id),
                forum_category_id: Set(source.category.id),
                forum_group_id: Set(group_id),
                site_id: Set(input.site_id),
                page_id: Set(Some(input.page_id)),
                created_by: Set(creator),
                created_at: Set(source.thread.created_at),
                title: Set(source.thread.title.clone()),
                description: Set(source.thread.description.clone()),
                from_wikidot: Set(true),
                ..Default::default()
            }
            .insert(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "cannot import page discussion thread",
                    ErrorType::DatabaseImport,
                )
            })?;
        } else {
            let mut thread = prior_thread.unwrap().into_active_model();
            thread.page_id = Set(Some(input.page_id));
            thread.title = Set(source.thread.title.clone());
            thread.description = Set(source.thread.description.clone());
            thread.update(ctx.transaction()).await.or_raise(|| {
                Error::new(
                    "cannot refresh imported thread metadata",
                    ErrorType::DatabaseImport,
                )
            })?;
        }
        let mut created_posts = 0;
        let mut created_revisions = 0;
        for source_post in &source.posts {
            let prior = forum_post::Entity::find_by_id(source_post.id)
                .one(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new("cannot load imported post", ErrorType::DatabaseImport)
                })?;
            let post = match prior {
                Some(post) => post,
                None => {
                    created_posts += 1;
                    forum_post::ActiveModel {
                        forum_post_id: Set(source_post.id),
                        parent_post_id: Set(None),
                        forum_thread_id: Set(source.thread.id),
                        forum_category_id: Set(source.category.id),
                        forum_group_id: Set(group_id),
                        site_id: Set(input.site_id),
                        user_id: Set(source_post.user_id),
                        created_at: Set(source_post.created_at),
                        from_wikidot: Set(true),
                        ..Default::default()
                    }
                    .insert(ctx.transaction())
                    .await
                    .or_raise(|| {
                        Error::new("cannot import source post", ErrorType::DatabaseImport)
                    })?
                }
            };
            let previous = match post.latest_revision_id {
                Some(id) => forum_post_revision::Entity::find_by_id(id)
                    .one(ctx.transaction())
                    .await
                    .or_raise(|| {
                        Error::new(
                            "cannot load prior imported revision",
                            ErrorType::DatabaseImport,
                        )
                    })?,
                None => None,
            };
            if post.latest_revision_id.is_some() && previous.is_none() {
                return Err(invalid("imported post points to a missing revision"));
            }
            if previous.as_ref().is_some_and(|r| !r.from_wikidot) {
                return Err(invalid(
                    "discussion import refuses a locally edited revision",
                ));
            }
            let prior_snapshot = PriorPostSnapshot::find_by_statement(Statement::from_sql_and_values(DatabaseBackend::Postgres,
                "SELECT source_has_edit,render_identity_sha256 FROM wikidot_discussion_post_snapshot WHERE forum_post_id=$1 AND revision_id=$2", [Value::from(post.forum_post_id), Value::from(post.latest_revision_id)]))
                .one(ctx.transaction()).await.or_raise(|| Error::new("cannot load source post edit metadata", ErrorType::DatabaseImport))?;
            let revision_user = source_post
                .edit
                .as_ref()
                .map_or(source_post.user_id, |e| e.user_id);
            let revision_time = source_post
                .edit
                .as_ref()
                .map_or(source_post.created_at, |e| e.created_at);
            let source_hash =
                TextService::create(ctx, source_post.wikitext.clone()).await?;
            let identity_hash = Sha256::digest(
                serde_json::to_vec(&source_post.observed_users)
                    .map_err(|_| invalid("cannot hash revision user observations"))?,
            )
            .to_vec();
            let unchanged = previous.as_ref().is_some_and(|r| {
                r.from_wikidot
                    && crate::services::render::compiled_generator_is_current(
                        &r.compiled_generator,
                    )
                    && r.title == source_post.title
                    && r.wikitext_hash == source_hash
                    && r.user_id == revision_user
                    && r.created_at == revision_time
            }) && prior_snapshot.is_some_and(|s| {
                s.source_has_edit == source_post.edit.is_some()
                    && s.render_identity_sha256 == identity_hash
            });
            let revision_id = if unchanged {
                previous.as_ref().unwrap().forum_post_revision_id
            } else {
                let rendered = ForumPostRevisionService::render_imported(
                    ctx,
                    input.site_id,
                    &source_post.title,
                    source_post.wikitext.clone(),
                    &source_post.observed_users,
                )
                .await?;
                let revision = forum_post_revision::ActiveModel {
                    forum_post_id: Set(post.forum_post_id),
                    forum_thread_id: Set(source.thread.id),
                    forum_category_id: Set(source.category.id),
                    forum_group_id: Set(group_id),
                    site_id: Set(input.site_id),
                    user_id: Set(revision_user),
                    created_at: Set(revision_time),
                    revision_number: Set(previous
                        .as_ref()
                        .map_or(0, |p| p.revision_number + 1)),
                    from_wikidot: Set(true),
                    title: Set(source_post.title.clone()),
                    wikitext_hash: Set(source_hash.to_vec()),
                    compiled_html_hash: Set(rendered.compiled_hash.to_vec()),
                    compiled_at: Set(rendered.compiled_at),
                    compiled_generator: Set(rendered.compiled_generator),
                    comments: Set("Observed corpus discussion snapshot".to_owned()),
                    ..Default::default()
                }
                .insert(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new(
                        "cannot append imported post revision",
                        ErrorType::DatabaseImport,
                    )
                })?;
                created_revisions += 1;
                revision.forum_post_revision_id
            };
            let mut active = post.into_active_model();
            active.latest_revision_id = Set(Some(revision_id));
            active.updated_at = Set(source_post.edit.as_ref().map(|e| e.created_at));
            active.update(ctx.transaction()).await.or_raise(|| {
                Error::new(
                    "cannot point to imported post revision",
                    ErrorType::DatabaseImport,
                )
            })?;
            execute(ctx, "INSERT INTO wikidot_discussion_post_snapshot(forum_post_id,revision_id,source_has_edit,render_identity_sha256) VALUES($1,$2,$3,$4) ON CONFLICT(forum_post_id) DO UPDATE SET revision_id=EXCLUDED.revision_id,source_has_edit=EXCLUDED.source_has_edit,render_identity_sha256=EXCLUDED.render_identity_sha256",
                vec![source_post.id.into(), revision_id.into(), source_post.edit.is_some().into(),identity_hash.into()]).await?;
        }
        for post in &source.posts {
            execute(
                ctx,
                "UPDATE forum_post SET parent_post_id=$2 WHERE forum_post_id=$1",
                vec![post.id.into(), post.parent_id.into()],
            )
            .await?;
        }
        let mut page = page.into_active_model();
        page.discussion_thread_id = Set(Some(source.thread.id));
        page.update(ctx.transaction()).await.or_raise(|| {
            Error::new(
                "cannot associate source discussion with its page",
                ErrorType::DatabaseImport,
            )
        })?;
        execute(ctx, "INSERT INTO wikidot_discussion_import_run(site_id,page_id,forum_thread_id,importing_user_id,source_sha256,manifest_sha256,observed_at,source_reference,manifest) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(page_id,manifest_sha256) DO NOTHING",
            vec![input.site_id.into(),input.page_id.into(),source.thread.id.into(),importing_user_id.into(),evidence_hash.into(),manifest_hash.clone().into(),source.observed_at.into(),source.reference.clone().into(),manifest_json.into()]).await?;
        let run = ImportedRun::find_by_statement(Statement::from_sql_and_values(DatabaseBackend::Postgres,
            "SELECT import_run_id FROM wikidot_discussion_import_run WHERE page_id=$1 AND manifest_sha256=$2", [Value::from(input.page_id),Value::from(manifest_hash.clone())]))
            .one(ctx.transaction()).await.or_raise(|| Error::new("cannot retrieve import receipt", ErrorType::DatabaseImport))?.ok_or_else(|| invalid("import receipt missing"))?;
        execute(ctx, "INSERT INTO wikidot_discussion_snapshot(page_id,forum_thread_id,source_site,source_system_creator,source_post_count,last_import_run_id) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(page_id) DO UPDATE SET last_import_run_id=EXCLUDED.last_import_run_id,source_post_count=EXCLUDED.source_post_count,source_system_creator=EXCLUDED.source_system_creator",
            vec![input.page_id.into(),source.thread.id.into(),source.site.clone().into(),matches!(source.thread.creator,DiscussionCreator::System).into(),(source.posts.len() as i32).into(),run.import_run_id.into()]).await?;
        // Native sequences must stay above preserved source IDs. This does not
        // renumber any existing entity or replace any historical revision.
        for (sequence, table, column) in [
            ("known_user_user_id_seq", "known_user", "user_id"),
            (
                "forum_category_forum_category_id_seq",
                "forum_category",
                "forum_category_id",
            ),
            (
                "forum_thread_forum_thread_id_seq",
                "forum_thread",
                "forum_thread_id",
            ),
            (
                "forum_post_forum_post_id_seq",
                "forum_post",
                "forum_post_id",
            ),
        ] {
            execute(ctx, &format!("SELECT setval('{sequence}',GREATEST((SELECT COALESCE(MAX({column}),1) FROM {table}),(SELECT last_value FROM {sequence})),true)"), vec![]).await?;
        }
        Ok(ImportDiscussionOutput {
            import_run_id: run.import_run_id,
            forum_thread_id: source.thread.id,
            manifest_sha256: hex::encode(&manifest_hash),
            created_thread,
            created_posts,
            created_revisions,
            unchanged: last.is_some_and(|last| manifest_hash == last.manifest_sha256),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> DiscussionSource {
        serde_json::from_value(serde_json::json!({
            "site":"source-site","fullname":"ordinary-page","reference":"https://source-site.wikidot.com/forum/t-401/example",
            "evidence_sha256":"a".repeat(64),"observed_at":"2026-10-12T00:00:00Z",
            "category":{"id":301,"name":"Pages","description":"Public pages","group_name":"Discussions","group_description":"","group_visible":true,"group_sort_index":0,"sort_index":0},
            "thread":{"id":401,"title":"Example","description":"Discussion","created_at":"2026-01-01T00:00:00Z","creator":{"type":"system"},"post_count":2},
            "actors":[{"id":201,"name":"First User","slug":"first-user"},{"id":202,"name":"Second User","slug":"second-user"}],
            "posts":[
                {"id":501,"parent_id":null,"user_id":201,"created_at":"2026-01-01T00:00:00Z","edit":null,"title":"First","wikitext":"First body","referenced_user_ids":[202]},
                {"id":502,"parent_id":501,"user_id":202,"created_at":"2026-01-02T00:00:00Z","edit":{"user_id":201,"created_at":"2026-01-03T00:00:00Z"},"title":"Reply","wikitext":"Second body","referenced_user_ids":[]}
            ]
        })).unwrap()
    }
    #[test]
    fn complete_source_graph_accepts_independent_inventory_order() {
        let mut source = source();
        assert!(source.validate().is_ok());
        source.posts.reverse();
        source.actors.reverse();
        assert!(source.validate().is_ok());
    }
    #[test]
    fn complete_source_graph_rejects_missing_and_cyclic_parents() {
        let mut missing = source();
        missing.posts[1].parent_id = Some(999);
        assert!(missing.validate().is_err());
        let mut cyclic = source();
        cyclic.posts[0].parent_id = Some(502);
        assert!(cyclic.validate().is_err());
    }
    #[test]
    fn complete_source_graph_rejects_missing_duplicate_and_unused_actors() {
        let mut missing = source();
        missing.actors.pop();
        assert!(missing.validate().is_err());
        let mut duplicate = source();
        duplicate.actors.push(duplicate.actors[0].clone());
        assert!(duplicate.validate().is_err());
        let mut unused = source();
        unused.actors.push(DiscussionActor {
            id: 203,
            name: "Unused".into(),
            slug: "unused".into(),
        });
        assert!(unused.validate().is_err());
        let mut body = source();
        body.posts[0].referenced_user_ids.push(204);
        assert!(body.validate().is_err());
    }
    #[test]
    fn source_graph_rejects_duplicates_incomplete_counts_and_reversed_edits() {
        let mut duplicate = source();
        duplicate.posts[1].id = 501;
        assert!(duplicate.validate().is_err());
        let mut incomplete = source();
        incomplete.thread.post_count = 3;
        assert!(incomplete.validate().is_err());
        let mut edit = source();
        edit.posts[1].edit.as_mut().unwrap().created_at =
            time::OffsetDateTime::UNIX_EPOCH;
        assert!(edit.validate().is_err());
    }
    #[test]
    fn source_graph_rejects_external_credentials_and_wrong_thread_references() {
        for reference in [
            "https://other.wikidot.com/forum/t-401/example",
            "https://secret@source-site.wikidot.com/forum/t-401/example",
            "https://source-site.wikidot.com/forum/t-402/example",
            "https://source-site.wikidot.com/forum/t-401/example?token=private",
        ] {
            let mut source = source();
            source.reference = reference.into();
            assert!(source.validate().is_err());
        }
    }
    #[test]
    fn bounded_snapshot_rejects_unsupported_shapes_and_excessive_source() {
        let mut oversized = source();
        oversized.posts[0].wikitext = "a".repeat(4 * 1024 * 1024 + 1);
        assert!(oversized.validate().is_err());
        let mut hash = source();
        hash.evidence_sha256 = "not-a-hash".into();
        assert!(hash.validate().is_err());
        let mut id = source();
        id.posts[0].id = i64::from(i32::MAX) + 1;
        assert!(id.validate().is_err());
    }
}
