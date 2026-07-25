/*
 * services/render/list_pages/authors.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Resolving the people behind ListPages rows.
//!
//! A row can name its creator, its last editor, or an imported Wikidot author
//! who has no local user. Turning those into display names means batching
//! lookups across a page of rows and caching them for the rest of the render,
//! which is what these functions do for `rendering.rs`.

use super::super::service::RenderService;
use super::{
    Cow, CurrentPageAuthorSource, ListPagesAuthorCacheKey, ListPagesSnapshotDisplay,
    PageRevisionService, ResolvedListPagesAuthors, WikidotUserDisplay,
    list_pages_author_cache_key,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::user::{self, Entity as UserTable};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::ServiceContext;
use crate::services::page_query::{FoundPageRow, normalize_wikidot_author_name};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, Statement,
    Value,
};
use std::collections::{BTreeMap, BTreeSet};

impl RenderService {
    pub(in crate::services::render) async fn load_wikidot_user_displays(
        ctx: &ServiceContext<'_>,
        pages: &[FoundPageRow],
    ) -> Result<BTreeMap<i64, WikidotUserDisplay>> {
        let make_error = || {
            Error::new(
                "failed to load Wikidot user names for ListPages render",
                ErrorType::Render,
            )
        };

        let user_ids = pages
            .iter()
            .flat_map(|page| [page.created_by, page.updated_by])
            .flatten()
            .collect::<BTreeSet<_>>();

        let wikidot_user_ids = user_ids
            .iter()
            .copied()
            .filter_map(|user_id| match i32::try_from(user_id) {
                Ok(user_id) => Some(user_id),
                Err(error) => {
                    warn!("Skipping Wikidot user ID {user_id} while rendering ListPages: {error}");
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        if user_ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let mut displays = BTreeMap::new();
        if !wikidot_user_ids.is_empty() {
            let users = WikidotUser::find()
                .filter(wikidot_user::Column::UserId.is_in(wikidot_user_ids.clone()))
                .all(ctx.transaction())
                .await
                .or_raise(make_error)?;

            displays.extend(users.into_iter().filter_map(|user| {
                let name = user.name.or_else(|| user.slug.clone())?;
                Some((
                    i64::from(user.user_id),
                    WikidotUserDisplay {
                        user_id: i64::from(user.user_id),
                        name,
                        slug: user.slug,
                        wikidot_profile: true,
                    },
                ))
            }));
        }

        let missing_user_ids = user_ids
            .into_iter()
            .filter(|user_id| !displays.contains_key(user_id))
            .collect::<Vec<_>>();
        if !missing_user_ids.is_empty() {
            let users = UserTable::find()
                .filter(user::Column::UserId.is_in(missing_user_ids))
                .all(ctx.transaction())
                .await
                .or_raise(make_error)?;

            displays.extend(users.into_iter().map(|user| {
                (
                    user.user_id,
                    WikidotUserDisplay {
                        user_id: user.user_id,
                        name: user.name,
                        slug: Some(user.slug),
                        wikidot_profile: false,
                    },
                )
            }));
        }

        Ok(displays)
    }

    pub(in crate::services::render) async fn load_list_pages_snapshot_displays(
        ctx: &ServiceContext<'_>,
        pages: &[FoundPageRow],
    ) -> Result<BTreeMap<i64, ListPagesSnapshotDisplay>> {
        #[derive(FromQueryResult, Debug)]
        struct SnapshotDisplayRow {
            page_id: i64,
            source_created_at: time::OffsetDateTime,
            source_updated_at: time::OffsetDateTime,
            created_by_name: Option<String>,
            updated_by_name: Option<String>,
            comments: i32,
            commented_at: Option<time::OffsetDateTime>,
            commented_by_name: Option<String>,
            rating_votes: Option<i64>,
            parent_fullname: Option<String>,
            source_revision_count: i32,
        }

        let page_ids = pages
            .iter()
            .map(|page| page.page_id)
            .collect::<BTreeSet<_>>();
        if page_ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let make_error = || {
            Error::new(
                "failed to load imported Wikidot snapshot metadata for ListPages render",
                ErrorType::Render,
            )
        };
        let values = page_ids
            .iter()
            .map(|page_id| format!("({page_id})"))
            .collect::<Vec<_>>()
            .join(", ");
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "WITH input(page_id) AS (VALUES {values}) \
                 SELECT snapshot.page_id, snapshot.source_created_at, snapshot.source_updated_at, \
                        snapshot.created_by_name, snapshot.updated_by_name, snapshot.comments, \
                        snapshot.commented_at, snapshot.commented_by_name, \
                        snapshot.parent_fullname, snapshot.source_revision_count, \
                        CASE \
                            WHEN snapshot.meta_json ->> 'votes_count' ~ '^[0-9]{{1,19}}$' \
                                 AND (length(snapshot.meta_json ->> 'votes_count') < 19 \
                                      OR snapshot.meta_json ->> 'votes_count' <= '9223372036854775807') \
                            THEN (snapshot.meta_json ->> 'votes_count')::bigint \
                            ELSE NULL \
                        END AS rating_votes \
                 FROM input \
                 JOIN wikidot_page_snapshot snapshot ON snapshot.page_id = input.page_id",
            ),
        );

        SnapshotDisplayRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)
            .map(|rows| {
                rows.into_iter()
                    .map(
                        |SnapshotDisplayRow {
                             page_id,
                             source_created_at,
                             source_updated_at,
                             created_by_name,
                             updated_by_name,
                             comments,
                             commented_at,
                             commented_by_name,
                             rating_votes,
                             parent_fullname,
                             source_revision_count,
                         }| {
                            (
                                page_id,
                                ListPagesSnapshotDisplay {
                                    created_at: source_created_at,
                                    updated_at: source_updated_at,
                                    created_by_name,
                                    updated_by_name,
                                    comments,
                                    commented_at,
                                    commented_by_name,
                                    rating_votes,
                                    parent_fullname,
                                    source_revision_count,
                                },
                            )
                        },
                    )
                    .collect()
            })
    }

    pub(in crate::services::render) async fn resolve_list_pages_authors_cached(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        author_names: &[Cow<'static, str>],
        author_filter_present: bool,
        exclude_current_page_author: bool,
        cache: &mut BTreeMap<ListPagesAuthorCacheKey, ResolvedListPagesAuthors>,
    ) -> Result<ResolvedListPagesAuthors> {
        let mut key = list_pages_author_cache_key(author_names, author_filter_present);
        key.negated = exclude_current_page_author;
        if let Some(resolved) = cache.get(&key) {
            return Ok(resolved.clone());
        }
        let resolved = Self::resolve_list_pages_authors(
            ctx,
            current_site_id,
            current_page_id,
            author_names,
            author_filter_present,
            exclude_current_page_author,
        )
        .await?;
        cache.insert(key, resolved.clone());
        Ok(resolved)
    }

    pub(in crate::services::render) async fn resolve_list_pages_authors(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        author_names: &[Cow<'static, str>],
        author_filter_present: bool,
        exclude_current_page_author: bool,
    ) -> Result<ResolvedListPagesAuthors> {
        if !author_filter_present {
            return Ok(ResolvedListPagesAuthors::All);
        }

        let mut snapshot_names = BTreeSet::new();
        let mut user_ids = BTreeSet::new();
        let mut include_current_page_author = exclude_current_page_author;
        for author in author_names {
            if author.as_ref() == "=" {
                include_current_page_author = true;
            } else {
                let author = normalize_wikidot_author_name(author);
                if !author.is_empty() {
                    snapshot_names.insert(author);
                }
            }
        }

        if include_current_page_author {
            let current_page_author = Self::load_current_page_author_source(
                ctx,
                current_site_id,
                current_page_id,
            )
            .await?;
            match current_page_author {
                Some(CurrentPageAuthorSource {
                    snapshot_present: true,
                    created_by_name: Some(created_by_name),
                    ..
                }) => {
                    let created_by_name = normalize_wikidot_author_name(&created_by_name);
                    if !created_by_name.is_empty() {
                        snapshot_names.insert(created_by_name);
                    }
                }
                Some(CurrentPageAuthorSource {
                    snapshot_present: true,
                    created_by_name: None,
                    ..
                })
                | Some(CurrentPageAuthorSource {
                    from_wikidot: true,
                    snapshot_present: false,
                    ..
                })
                | None => {}
                Some(CurrentPageAuthorSource {
                    from_wikidot: false,
                    snapshot_present: false,
                    ..
                }) => {
                    if let Some(revision) = PageRevisionService::get_earliest_optional(
                        ctx,
                        current_site_id,
                        current_page_id,
                    )
                    .await?
                    {
                        user_ids.insert(revision.user_id);
                    }
                }
            }
        }

        user_ids.extend(Self::load_wikidot_author_ids(ctx, &snapshot_names).await?);
        if exclude_current_page_author {
            // An exclusion that resolved to nobody would silently widen the
            // query back to every page, so the caller preserves the module.
            return Ok(ResolvedListPagesAuthors::NotAny {
                user_ids: user_ids.into_iter().collect(),
                wikidot_snapshot_names: snapshot_names
                    .into_iter()
                    .map(Cow::Owned)
                    .collect(),
            });
        }
        if user_ids.is_empty() && snapshot_names.is_empty() {
            Ok(ResolvedListPagesAuthors::None)
        } else {
            Ok(ResolvedListPagesAuthors::Any {
                user_ids: user_ids.into_iter().collect(),
                wikidot_snapshot_names: snapshot_names
                    .into_iter()
                    .map(Cow::Owned)
                    .collect(),
            })
        }
    }

    pub(in crate::services::render) async fn load_wikidot_author_ids(
        ctx: &ServiceContext<'_>,
        wanted: &BTreeSet<String>,
    ) -> Result<Vec<i64>> {
        if wanted.is_empty() {
            return Ok(Vec::new());
        }

        let make_error = || {
            Error::new(
                "failed to load Wikidot author IDs for ListPages render",
                ErrorType::Render,
            )
        };
        let users = WikidotUser::find()
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;

        let author_ids = users
            .into_iter()
            .filter(|user| {
                user.name.as_ref().is_some_and(|name| {
                    wanted.contains(&normalize_wikidot_author_name(name))
                }) || user.slug.as_ref().is_some_and(|slug| {
                    wanted.contains(&normalize_wikidot_author_name(slug))
                })
            })
            .map(|user| i64::from(user.user_id))
            .collect::<Vec<_>>();

        Ok(author_ids)
    }

    pub(in crate::services::render) async fn load_current_page_author_source(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
    ) -> Result<Option<CurrentPageAuthorSource>> {
        let make_error = || {
            Error::new(
                "failed to load current page author provenance for ListPages render",
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_sql_and_values(
            txn.get_database_backend(),
            "SELECT page.from_wikidot, snapshot.page_id IS NOT NULL AS snapshot_present, snapshot.created_by_name \
             FROM page \
             LEFT JOIN wikidot_page_snapshot snapshot ON snapshot.page_id = page.page_id \
             WHERE page.site_id = $1 AND page.page_id = $2 AND page.deleted_at IS NULL",
            [Value::from(current_site_id), Value::from(current_page_id)],
        );

        CurrentPageAuthorSource::find_by_statement(statement)
            .one(txn)
            .await
            .or_raise(make_error)
    }
}
