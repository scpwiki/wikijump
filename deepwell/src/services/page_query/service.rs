/*
 * services/page_query/service.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#![allow(dead_code, unused_variables)] // TEMP

use super::structs::{
    AuthorSelector, CategoriesSelector, ComparisonOperation, DataFormSelector,
    DateSelector, DateTimeResolution, FoundPageFields, FoundPageRow, FoundPages,
    IncludedCategories, MAX_PAGE_QUERY_SCORE_SELECTORS, OrderBySelector, OrderProperty,
    PageParentSelector, PageQuery, PageQueryResultEnvelope, PageQueryResultMetadata,
    PageTypeSelector, PaginationSelector, ScoreSelector, TagCondition,
    normalize_wikidot_author_name, parse_static_wikidot_data_form_values,
    static_wikidot_data_form_matches,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_connection::{self, Entity as PageConnection};
use crate::models::page_parent::{self, Entity as PageParent};
use crate::models::{page_revision, text};
use crate::services::ServiceContext;
use crate::services::score::ScoreValue;
use crate::services::{PageService, ParentService, ScoreService};
use crate::types::Reference;
use sea_orm::DatabaseTransaction;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use sea_query::extension::postgres::PgBinOper;
use sea_query::{Expr, Query, SimpleExpr, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct PageQueryProjection {
    pages: Vec<page::Model>,
    fields: FoundPageFields,
    order: OrderBySelector,
    offset: u32,
    pagination: PaginationSelector,
    candidate_count: Option<usize>,
    cap_exceeded: bool,
    sql_limit_offset_applied: bool,
    filtering_deferred_to_rust: bool,
    ordering_deferred_to_rust: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ScoreFilterCacheValue {
    Integer(i64),
    Float(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ScoreFilterCacheKey {
    site_id: i64,
    selectors: Vec<(u8, ScoreFilterCacheValue)>,
}

impl ScoreFilterCacheKey {
    fn new(site_id: i64, selectors: &[ScoreSelector]) -> Self {
        Self {
            site_id,
            selectors: selectors
                .iter()
                .map(|selector| {
                    let value = match selector.score {
                        ScoreValue::Integer(value) => {
                            ScoreFilterCacheValue::Integer(value)
                        }
                        ScoreValue::Float(value) => {
                            ScoreFilterCacheValue::Float(value.to_bits())
                        }
                    };
                    let comparison = match selector.comparison {
                        ComparisonOperation::GreaterThan => 0,
                        ComparisonOperation::LessThan => 1,
                        ComparisonOperation::GreaterOrEqualThan => 2,
                        ComparisonOperation::LessOrEqualThan => 3,
                        ComparisonOperation::Equal => 4,
                        ComparisonOperation::NotEqual => 5,
                    };
                    (comparison, value)
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScoreFilterCacheLookup {
    FirstUse,
    RepeatedUnmaterialized,
    Materialized(ScoreFilterMembership),
    Uncacheable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScoreFilterMembership {
    Included(Vec<i64>),
    Excluded(Vec<i64>),
}

impl ScoreFilterMembership {
    fn len(&self) -> usize {
        match self {
            Self::Included(page_ids) | Self::Excluded(page_ids) => page_ids.len(),
        }
    }
}

// Keeps each request-local ID array bounded while accommodating the 24,430-page EN corpus.
const MAX_CACHED_SCORE_FILTER_PAGE_IDS: usize = 50_000;
// Bounds aggregate request-local score-cache retention across distinct score filters.
const MAX_TOTAL_CACHED_SCORE_FILTER_PAGE_IDS: usize = 100_000;

/// Request-local cache for broad score predicates shared by multiple ListPages queries.
/// It caches only qualifying IDs; each caller still applies its own filters and ordering.
#[derive(Debug, Default)]
pub(crate) struct PageQueryScoreFilterCache {
    seen: BTreeSet<ScoreFilterCacheKey>,
    memberships: BTreeMap<ScoreFilterCacheKey, ScoreFilterMembership>,
    cached_page_ids: usize,
    uncacheable: BTreeSet<ScoreFilterCacheKey>,
}

#[derive(Debug, Default)]
pub(crate) struct PageQueryScoreFilterSession {
    seen: BTreeSet<ScoreFilterCacheKey>,
}

impl PageQueryScoreFilterSession {
    fn register_use(&mut self, key: &ScoreFilterCacheKey) -> bool {
        self.seen.insert(key.clone())
    }
}

impl PageQueryScoreFilterCache {
    fn materialized_membership(
        &self,
        key: &ScoreFilterCacheKey,
    ) -> Option<ScoreFilterMembership> {
        if self.uncacheable.contains(key) {
            return None;
        }
        self.memberships.get(key).cloned()
    }

    fn lookup(
        &mut self,
        key: &ScoreFilterCacheKey,
        register_logical_use: bool,
    ) -> ScoreFilterCacheLookup {
        if self.uncacheable.contains(key) {
            return ScoreFilterCacheLookup::Uncacheable;
        }
        if let Some(membership) = self.memberships.get(key) {
            return ScoreFilterCacheLookup::Materialized(membership.clone());
        }
        if self.seen.contains(key) {
            if register_logical_use {
                return ScoreFilterCacheLookup::RepeatedUnmaterialized;
            }
            return ScoreFilterCacheLookup::FirstUse;
        }
        if register_logical_use {
            self.seen.insert(key.clone());
        } else {
            debug_assert!(false, "a score key must be registered on its first batch");
        }
        ScoreFilterCacheLookup::FirstUse
    }

    fn insert(&mut self, key: ScoreFilterCacheKey, membership: ScoreFilterMembership) {
        debug_assert!(membership.len() <= MAX_CACHED_SCORE_FILTER_PAGE_IDS);
        let membership_len = membership.len();
        let replaced_len = self
            .memberships
            .get(&key)
            .map_or(0, ScoreFilterMembership::len);
        let new_total = self.cached_page_ids - replaced_len + membership_len;
        if new_total > MAX_TOTAL_CACHED_SCORE_FILTER_PAGE_IDS {
            self.mark_uncacheable(key);
            return;
        }

        self.memberships.insert(key, membership);
        self.cached_page_ids = new_total;
    }

    fn mark_uncacheable(&mut self, key: ScoreFilterCacheKey) {
        if let Some(membership) = self.memberships.remove(&key) {
            self.cached_page_ids -= membership.len();
        }
        self.uncacheable.insert(key);
    }
}

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn find(
        ctx: &ServiceContext<'_>,
        query: PageQuery<'_>,
    ) -> Result<FoundPages> {
        Ok(Self::find_with_metadata(ctx, query).await?.pages)
    }

    pub async fn find_with_metadata(
        ctx: &ServiceContext<'_>,
        query: PageQuery<'_>,
    ) -> Result<PageQueryResultEnvelope> {
        Self::find_with_metadata_cached(ctx, query, None, None).await
    }

    pub(crate) async fn find_with_metadata_cached(
        ctx: &ServiceContext<'_>,
        query: PageQuery<'_>,
        score_filter_cache: Option<&mut PageQueryScoreFilterCache>,
        score_filter_session: Option<&mut PageQueryScoreFilterSession>,
    ) -> Result<PageQueryResultEnvelope> {
        let queried_site_id = query.queried_site_id.unwrap_or(query.current_site_id);
        let PageQuery {
            current_page_id,
            current_site_id,
            queried_site_id: _,
            page_type,
            categories:
                CategoriesSelector {
                    included_categories,
                    excluded_categories,
                },
            tags:
                TagCondition {
                    any_present: any_tags,
                    all_present: all_tags,
                    none_present: no_tags,
                    untagged,
                },
            page_parent,
            contains_outgoing_links,
            creation_date,
            update_date,
            author,
            score,
            votes,
            offset,
            range,
            name,
            slug,
            slugs,
            data_form_fields,
            order,
            candidate_limit,
            pagination,
            variables,
            fields,
        } = query;
        let (category_mode, included_category_count) = match &included_categories {
            IncludedCategories::All => ("all", 0),
            IncludedCategories::List(categories) => ("list", categories.len()),
        };
        let (parent_mode, parent_count) = match &page_parent {
            PageParentSelector::All => ("all", 0),
            PageParentSelector::NoParent => ("none", 0),
            PageParentSelector::SameParents => ("same", 0),
            PageParentSelector::DifferentParents => ("different", 0),
            PageParentSelector::ChildOf => ("children", 0),
            PageParentSelector::HasParents(parents) => ("list", parents.len()),
        };
        let order = order.unwrap_or_default();
        debug!(
            "Building ListPages query: site_id={queried_site_id}, page_type={page_type:?}, category_mode={category_mode}, included_category_count={included_category_count}, excluded_category_count={}, parent_mode={parent_mode}, parent_count={parent_count}, outgoing_link_count={}, exact_slug_count={}, name_pattern={}, any_tag_count={}, all_tag_count={}, excluded_tag_count={}, untagged={untagged}, score_filter_count={}, data_form_filter_count={}, order={:?}, ascending={}, offset={offset}, limit={:?}, candidate_limit={candidate_limit:?}",
            excluded_categories.len(),
            contains_outgoing_links.len(),
            slugs.len() + usize::from(slug.is_some()),
            name.is_some(),
            any_tags.len(),
            all_tags.len(),
            no_tags.len(),
            score.len(),
            data_form_fields.len(),
            order.property,
            order.ascending,
            pagination.limit,
        );

        let make_error =
            || Error::new("failed to create ListPages query", ErrorType::PageQuery);

        if score.len() > MAX_PAGE_QUERY_SCORE_SELECTORS {
            return Err(Error::new(
                "ListPages score selector limit exceeded",
                ErrorType::PageQuery,
            )
            .into());
        }
        let score = score.to_vec();

        let txn = ctx.transaction();
        if !score.is_empty() {
            // These queries deliberately switch between candidate-correlated and
            // site-wide score plans. PostgreSQL's generic prepared plan loses the
            // selector and candidate cardinalities after repeated executions and
            // can make a six-module ListPages render several times slower. Keep
            // the choice local to this render transaction and leave non-score
            // queries on the server default.
            txn.execute_unprepared("SET LOCAL plan_cache_mode = force_custom_plan")
                .await
                .or_raise(make_error)?;
        }
        let mut condition = Condition::all();

        condition = condition.add(page::Column::SiteId.eq(queried_site_id));

        let hidden_condition = Expr::cust_with_expr(
            r#"regexp_replace($1, '^.*:', '') LIKE '\_%' ESCAPE '\'"#,
            Expr::col((Page, page::Column::Slug)),
        );
        match page_type {
            PageTypeSelector::Hidden => {
                condition = condition.add(hidden_condition);
            }
            PageTypeSelector::Normal => {
                condition = condition.add(hidden_condition.not());
            }
            PageTypeSelector::All => {}
        }

        // Categories (included and excluded)
        let page_category_condition = match included_categories {
            // If all categories are selected (using an asterisk or by only specifying excluded categories),
            // then filter only by site_id and exclude the specified excluded categories.
            IncludedCategories::All => page::Column::PageCategoryId.in_subquery(
                Query::select()
                    .column(page_category::Column::CategoryId)
                    .from(PageCategory)
                    .and_where(page_category::Column::SiteId.eq(queried_site_id))
                    .and_where(
                        page_category::Column::Slug
                            .is_not_in(excluded_categories.iter().map(|c| c.as_ref())),
                    )
                    .to_owned(),
            ),

            // If a specific list of categories is provided, filter by site_id, inclusion in the
            // specified included categories, and exclude the specified excluded categories.
            //
            // NOTE: Exclusion can only have an effect in this query if it is *also* included.
            //       Although by definition this is the same as not including the category in the
            //       included categories to begin with, it is still accounted for to preserve
            //       backwards-compatibility with poorly-constructed ListPages modules.
            IncludedCategories::List(included_categories) => page::Column::PageCategoryId
                .in_subquery(
                    Query::select()
                        .column(page_category::Column::CategoryId)
                        .from(PageCategory)
                        .and_where(page_category::Column::SiteId.eq(queried_site_id))
                        .and_where(
                            page_category::Column::Slug
                                .is_in(included_categories.iter().map(|c| c.as_ref())),
                        )
                        .and_where(
                            page_category::Column::Slug.is_not_in(
                                excluded_categories.iter().map(|c| c.as_ref()),
                            ),
                        )
                        .to_owned(),
                ),
        };
        condition = condition.add(page_category_condition);

        let page_parent_condition = match page_parent {
            PageParentSelector::All => None,
            PageParentSelector::NoParent => Some(
                page::Column::PageId.not_in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .to_owned(),
                ),
            ),

            PageParentSelector::SameParents => Some(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(
                            page_parent::Column::ParentPageId.is_in(
                                current_parent_ids(ctx, current_site_id, current_page_id)
                                    .await
                                    .or_raise(make_error)?,
                            ),
                        )
                        .to_owned(),
                ),
            ),

            PageParentSelector::DifferentParents => Some(
                page::Column::PageId.not_in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(
                            page_parent::Column::ParentPageId.is_in(
                                current_parent_ids(ctx, current_site_id, current_page_id)
                                    .await
                                    .or_raise(make_error)?,
                            ),
                        )
                        .to_owned(),
                ),
            ),

            PageParentSelector::ChildOf => Some(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.eq(current_page_id))
                        .to_owned(),
                ),
            ),

            // Wikidot's parent selector is any-of rather than all-of.
            PageParentSelector::HasParents(parents) => {
                let parent_ids = PageService::get_pages(ctx, queried_site_id, parents)
                    .await
                    .or_raise(make_error)?
                    .into_iter()
                    .map(|page| page.page_id);

                Some(
                    page::Column::PageId.in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.is_in(parent_ids),
                            )
                            .to_owned(),
                    ),
                )
            }
        };
        if let Some(page_parent_condition) = page_parent_condition {
            condition = condition.add(page_parent_condition);
        }

        // Slug
        if let Some(slug) = slug {
            if !slugs.is_empty() {
                return Err(Error::new(
                    "page query cannot combine singular and plural slug selectors",
                    ErrorType::PageQuery,
                )
                .into());
            }
            let slug = slug.as_ref();
            condition = condition.add(page::Column::Slug.eq(slug));
        }
        if !slugs.is_empty() {
            condition = condition
                .add(page::Column::Slug.is_in(slugs.iter().map(|slug| slug.as_ref())));
        }
        if let Some(name) = name {
            let pattern = wikidot_name_pattern(name.as_ref());
            condition = condition.add(page::Column::Slug.like(pattern));
        }

        // Initial page author. Local pages use the user ID on their earliest available revision. Corpus imports intentionally keep the Wikidot display name in wikidot_page_snapshot instead of fabricating local users, so the two representations are combined with OR semantics.
        match author {
            AuthorSelector::All => {}
            AuthorSelector::None => {
                condition = condition.add(SimpleExpr::Custom("FALSE".into()));
            }
            AuthorSelector::Any {
                user_ids,
                wikidot_snapshot_names,
            } => {
                let normalized_snapshot_names = wikidot_snapshot_names
                    .iter()
                    .map(|name| normalize_wikidot_author_name(name))
                    .filter(|name| !name.is_empty())
                    .collect::<Vec<_>>();
                let mut author_condition = Condition::any();
                let mut has_author_condition = false;

                if !user_ids.is_empty() {
                    let placeholders = postgres_bind_placeholders(user_ids.len());
                    author_condition = author_condition.add(Expr::cust_with_values(
                        format!(
                            "EXISTS (SELECT 1 FROM page_revision pr WHERE pr.page_id = page.page_id AND pr.user_id IN ({placeholders}) AND pr.revision_id = (SELECT pr2.revision_id FROM page_revision pr2 WHERE pr2.page_id = page.page_id ORDER BY pr2.revision_number ASC, pr2.revision_id ASC LIMIT 1))"
                        ),
                        user_ids.iter().copied(),
                    ));
                    has_author_condition = true;
                }

                if !normalized_snapshot_names.is_empty() {
                    let placeholders =
                        postgres_bind_placeholders(normalized_snapshot_names.len());
                    author_condition = author_condition.add(Expr::cust_with_values(
                        format!(
                            "EXISTS (SELECT 1 FROM wikidot_page_snapshot snapshot WHERE snapshot.page_id = page.page_id AND replace(replace(lower(btrim(snapshot.created_by_name)), '_', '-'), ' ', '-') IN ({placeholders}))"
                        ),
                        normalized_snapshot_names,
                    ));
                    has_author_condition = true;
                }

                if has_author_condition {
                    condition = condition.add(author_condition);
                } else {
                    condition = condition.add(SimpleExpr::Custom("FALSE".into()));
                }
            }
            AuthorSelector::NotAny {
                user_ids,
                wikidot_snapshot_names,
            } => {
                let normalized_snapshot_names = wikidot_snapshot_names
                    .iter()
                    .map(|name| normalize_wikidot_author_name(name))
                    .filter(|name| !name.is_empty())
                    .collect::<Vec<_>>();

                if !user_ids.is_empty() {
                    let placeholders = postgres_bind_placeholders(user_ids.len());
                    condition = condition.add(Expr::cust_with_values(
                        format!(
                            "NOT EXISTS (SELECT 1 FROM page_revision pr WHERE pr.page_id = page.page_id AND pr.user_id IN ({placeholders}) AND pr.revision_id = (SELECT pr2.revision_id FROM page_revision pr2 WHERE pr2.page_id = page.page_id ORDER BY pr2.revision_number ASC, pr2.revision_id ASC LIMIT 1))"
                        ),
                        user_ids.iter().copied(),
                    ));
                }

                if !normalized_snapshot_names.is_empty() {
                    let placeholders =
                        postgres_bind_placeholders(normalized_snapshot_names.len());
                    condition = condition.add(Expr::cust_with_values(
                        format!(
                            "NOT EXISTS (SELECT 1 FROM wikidot_page_snapshot snapshot WHERE snapshot.page_id = page.page_id AND replace(replace(lower(btrim(snapshot.created_by_name)), '_', '-'), ' ', '-') IN ({placeholders}))"
                        ),
                        normalized_snapshot_names,
                    ));
                }
            }
        }

        // Contains-link
        //
        // Selects pages that have an outgoing link (`from_page_id`)
        // to a specified page (`to_page_id`). An empty selector means
        // no link constraint; adding an empty subquery here makes every
        // ordinary ListPages query return no rows.
        if !contains_outgoing_links.is_empty() {
            condition = condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_connection::Column::FromPageId)
                        .from(PageConnection)
                        .and_where({
                            let incoming_ids = PageService::get_pages(
                                ctx,
                                queried_site_id,
                                contains_outgoing_links,
                            )
                            .await
                            .or_raise(make_error)?
                            .into_iter()
                            .map(|page| page.page_id);

                            page_connection::Column::ToPageId.is_in(incoming_ids)
                        })
                        .to_owned(),
                ),
            );
        }

        condition = condition.add(date_selector_condition(
            page::Column::CreatedAt,
            creation_date,
        ));
        condition = condition.add(date_selector_condition(
            page::Column::UpdatedAt,
            update_date,
        ));
        if !votes.is_empty() {
            return Err(Error::new(
                "ListPages vote-count filtering is not implemented",
                ErrorType::PageQuery,
            )
            .into());
        }

        // Build the final query
        let mut query = Page::find()
            .filter(page::Column::DeletedAt.is_null())
            .filter(condition);
        let needs_tag_filter = !all_tags.is_empty()
            || !any_tags.is_empty()
            || !no_tags.is_empty()
            || untagged;
        let needs_revision_join = needs_tag_filter
            || matches!(
                order.property,
                OrderProperty::Title | OrderProperty::AltTitle | OrderProperty::Size
            );
        if needs_tag_filter {
            query = query.join(JoinType::Join, page::Relation::PageRevision.def());
        } else if needs_revision_join {
            query = query.join(JoinType::LeftJoin, page::Relation::PageRevision.def());
        }

        // Tag filtering. Tags live on the current page revision, so this joins through
        // page.latest_revision_id -> page_revision.revision_id before applying array predicates.
        for tag in all_tags {
            query = query.filter(
                Expr::col(page_revision::Column::Tags)
                    .binary(PgBinOper::Contains, Expr::val(vec![tag.to_string()])),
            );
        }

        if !any_tags.is_empty() {
            query = query.filter(
                Expr::col(page_revision::Column::Tags).binary(
                    PgBinOper::Overlap,
                    Expr::val(
                        any_tags
                            .iter()
                            .map(|tag| tag.to_string())
                            .collect::<Vec<_>>(),
                    ),
                ),
            );
        }

        for tag in no_tags {
            query = query.filter(
                Expr::col(page_revision::Column::Tags)
                    .binary(PgBinOper::Contains, Expr::val(vec![tag.to_string()]))
                    .not(),
            );
        }

        if untagged {
            query = query.filter(SimpleExpr::Custom(
                "cardinality(page_revision.tags) = 0".into(),
            ));
        }

        query = apply_score_filters(
            txn,
            query,
            queried_site_id,
            &score,
            score_filter_cache,
            score_filter_session,
        )
        .await?;

        // Add on at the query-level (ORDER BY, LIMIT)
        let score_order = matches!(order.property, OrderProperty::Score);
        {
            use sea_orm::query::Order;
            use sea_query::func::Func;

            let OrderBySelector {
                property,
                ascending,
            } = order;

            let order = if ascending { Order::Asc } else { Order::Desc };

            match property {
                OrderProperty::PageSlug => {
                    let expr = Expr::cust_with_expr(
                        "regexp_replace(regexp_replace($1, '^.*:', ''), '[^[:alnum:]]', '', 'g')",
                        Expr::col((Page, page::Column::Slug)),
                    );
                    query = query
                        .order_by(expr, order.clone())
                        .order_by(Expr::col((Page, page::Column::Slug)), order);
                }
                OrderProperty::FullSlug => {
                    query = query.order_by(page::Column::Slug, order);
                }
                OrderProperty::Title => {
                    query = query.order_by(page_revision::Column::Title, order);
                }
                OrderProperty::AltTitle => {
                    query = query.order_by(page_revision::Column::AltTitle, order);
                }
                OrderProperty::CreatedBy => {
                    let expr = SimpleExpr::Custom(
                        "(SELECT pr.user_id FROM page_revision pr WHERE pr.page_id = page.page_id ORDER BY pr.revision_number ASC, pr.revision_id ASC LIMIT 1)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::CreatedAt => {
                    query = query.order_by(page::Column::CreatedAt, order);
                }
                OrderProperty::UpdatedAt => {
                    query = query.order_by(page::Column::UpdatedAt, order);
                }
                OrderProperty::Size => {
                    query = query.join(
                        if needs_tag_filter {
                            JoinType::Join
                        } else {
                            JoinType::LeftJoin
                        },
                        page_revision::Relation::Text1.def(),
                    );
                    let expr = SimpleExpr::Custom("text.character_count".into());
                    query = query.order_by(expr, order);
                }
                OrderProperty::Score => {}
                OrderProperty::Votes => {
                    let expr = SimpleExpr::Custom(
                        "COALESCE((SELECT COUNT(*) FROM page_vote pv WHERE pv.page_id = page.page_id AND pv.deleted_at IS NULL AND pv.disabled_at IS NULL), 0)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::Revisions => {
                    let expr = SimpleExpr::Custom(
                        "COALESCE((SELECT COUNT(*) FROM page_revision pr WHERE pr.page_id = page.page_id), 0)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::Comments => {
                    let expr = SimpleExpr::Custom(
                        "COALESCE((SELECT COUNT(*) FROM forum_post fp JOIN forum_thread ft ON fp.forum_thread_id = ft.forum_thread_id WHERE ft.page_id = page.page_id AND fp.deleted_at IS NULL AND ft.deleted_at IS NULL), 0)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::Random => {
                    let expr = SimpleExpr::FunctionCall(Func::random());
                    query = query.order_by(expr, order);
                }
                OrderProperty::DataFormFieldName => {
                    return Err(Error::new(
                        "ListPages data form field ordering is not implemented",
                        ErrorType::PageQuery,
                    )
                    .into());
                }
            };
            if !matches!(property, OrderProperty::Random | OrderProperty::Score) {
                if !matches!(property, OrderProperty::PageSlug | OrderProperty::FullSlug)
                {
                    query = query.order_by(page::Column::Slug, Order::Asc);
                }
                query = query.order_by(page::Column::PageId, Order::Asc);
            }
        }

        let filtering_deferred_to_rust = !data_form_fields.is_empty();
        let ordering_deferred_to_rust = score_order;
        let defer_offset_limit = ordering_deferred_to_rust || filtering_deferred_to_rust;
        let sql_limit_offset_applied =
            !defer_offset_limit && (offset > 0 || pagination.limit.is_some());
        if !defer_offset_limit {
            if offset > 0 {
                query = query.offset(u64::from(offset));
            }
            if let Some(limit) = pagination.limit {
                query = query.limit(limit);
            }
        } else if let Some(candidate_limit) = candidate_limit {
            query = query.limit(candidate_limit);
        }

        // TODO pagination
        //      the "reverse" field means that, for each page, it is reversed.
        //
        //      this does not affect the overall ORDER BY
        //      for instance, imagine we are selecting from the positive integers
        //      if the pagination limit is 5 and the order is ascending, but reverse = true,
        //      then this means we get pages like:
        //
        //      1. [ 4,  3,  2,  1,  0]
        //      2. [ 9,  8,  7,  6,  5]
        //      3. [14, 13, 12, 11, 10]

        // Execute it!
        let mut pages = query.all(txn).await.or_raise(make_error)?;
        let candidate_count = Some(pages.len());
        // Both deferred paths resolve in Rust over the fetched candidate set, so
        // a scan that filled its bound may be missing rows that belong in the
        // result. The caller preserves the module rather than rendering a list
        // sorted or filtered from a truncated candidate set.
        let cap_exceeded = defer_offset_limit
            && candidate_limit
                .and_then(|limit| usize::try_from(limit).ok())
                .is_some_and(|limit| pages.len() >= limit);
        if !data_form_fields.is_empty() {
            pages = filter_pages_by_data_form_fields(ctx, pages, data_form_fields)
                .await
                .or_raise(make_error)?;
        }

        project_page_query_results(
            ctx,
            PageQueryProjection {
                pages,
                fields,
                order,
                offset,
                pagination,
                candidate_count,
                cap_exceeded,
                sql_limit_offset_applied,
                filtering_deferred_to_rust,
                ordering_deferred_to_rust,
            },
        )
        .await
        .or_raise(make_error)
    }
}

// Resolving this list ahead of the main query avoids the full sequential scan produced by
// PostgreSQL's equivalent page_parent self-join.
async fn current_parent_ids(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    current_page_id: i64,
) -> Result<Vec<i64>> {
    ParentService::get_parents(ctx, current_site_id, Reference::Id(current_page_id))
        .await
        .map(|parents| {
            parents
                .into_iter()
                .map(|parent| parent.parent_page_id)
                .collect()
        })
}

async fn apply_score_filters(
    txn: &DatabaseTransaction,
    query: sea_orm::Select<page::Entity>,
    queried_site_id: i64,
    score: &[ScoreSelector],
    mut cache: Option<&mut PageQueryScoreFilterCache>,
    session: Option<&mut PageQueryScoreFilterSession>,
) -> Result<sea_orm::Select<page::Entity>> {
    if score.is_empty() {
        return Ok(query);
    }

    let key = ScoreFilterCacheKey::new(queried_site_id, score);
    if let Some(membership) = cache
        .as_deref()
        .and_then(|cache| cache.materialized_membership(&key))
    {
        return Ok(query.filter(score_membership_condition(membership)));
    }

    let observed_candidates = query
        .clone()
        .select_only()
        .column(page::Column::PageId)
        .distinct()
        .limit((MAX_CORRELATED_SCORE_CANDIDATES + 1) as u64)
        .into_tuple::<i64>()
        .all(txn)
        .await
        .or_raise(|| {
            Error::new(
                "failed to plan ListPages score filter",
                ErrorType::PageQuery,
            )
        })?
        .len();

    match score_filter_plan_from_probe(queried_site_id, observed_candidates) {
        ScoreFilterPlan::CandidateCorrelated => Ok(query.filter(
            score_selectors_condition(score, ScoreFilterPlan::CandidateCorrelated),
        )),
        ScoreFilterPlan::SiteWide { site_id } => {
            let key = ScoreFilterCacheKey::new(site_id, score);
            let register_logical_use = session
                .map(|session| session.register_use(&key))
                .unwrap_or(true);
            let lookup = cache
                .as_deref_mut()
                .map(|cache| cache.lookup(&key, register_logical_use));

            match lookup {
                Some(ScoreFilterCacheLookup::Materialized(membership)) => {
                    Ok(query.filter(score_membership_condition(membership)))
                }
                Some(ScoreFilterCacheLookup::RepeatedUnmaterialized) => {
                    match materialize_score_membership(txn, score, site_id).await? {
                        Some(membership) => {
                            cache
                                .expect("score cache should still be available")
                                .insert(key, membership.clone());
                            Ok(query.filter(score_membership_condition(membership)))
                        }
                        None => {
                            cache
                                .expect("score cache should still be available")
                                .mark_uncacheable(key);
                            Ok(query.filter(score_selectors_condition(
                                score,
                                ScoreFilterPlan::SiteWide { site_id },
                            )))
                        }
                    }
                }
                Some(ScoreFilterCacheLookup::FirstUse)
                | Some(ScoreFilterCacheLookup::Uncacheable)
                | None => Ok(query.filter(score_selectors_condition(
                    score,
                    ScoreFilterPlan::SiteWide { site_id },
                ))),
            }
        }
    }
}

async fn project_page_query_results(
    ctx: &ServiceContext<'_>,
    PageQueryProjection {
        mut pages,
        fields,
        order,
        offset,
        pagination,
        candidate_count,
        cap_exceeded,
        sql_limit_offset_applied,
        filtering_deferred_to_rust,
        ordering_deferred_to_rust,
    }: PageQueryProjection,
) -> Result<PageQueryResultEnvelope> {
    let txn = ctx.transaction();
    let make_error =
        || Error::new("failed to project ListPages query", ErrorType::PageQuery);

    let mut page_ids = pages.iter().map(|page| page.page_id).collect::<Vec<_>>();
    let score_by_page_id: BTreeMap<i64, f32> =
        if (fields.score || ordering_deferred_to_rust) && !page_ids.is_empty() {
            ScoreService::scores_bulk(ctx, &page_ids)
                .await
                .or_raise(make_error)?
                .into_iter()
                .map(|(page_id, score)| (page_id, score_to_f32(score)))
                .collect()
        } else {
            BTreeMap::new()
        };

    let defer_offset_limit = ordering_deferred_to_rust || filtering_deferred_to_rust;
    if defer_offset_limit {
        if ordering_deferred_to_rust {
            pages.sort_by(|left, right| {
                let left_score =
                    score_by_page_id.get(&left.page_id).copied().unwrap_or(0.0);
                let right_score =
                    score_by_page_id.get(&right.page_id).copied().unwrap_or(0.0);
                let ordering = left_score
                    .partial_cmp(&right_score)
                    .unwrap_or(Ordering::Equal);
                let ordering = if order.ascending {
                    ordering
                } else {
                    ordering.reverse()
                };
                ordering
                    .then_with(|| left.slug.cmp(&right.slug))
                    .then_with(|| left.page_id.cmp(&right.page_id))
            });
        }
        if offset > 0 {
            let skip = (offset as usize).min(pages.len());
            pages.drain(..skip);
        }
        if let Some(limit) = pagination.limit {
            pages.truncate(std::cmp::Ord::min(limit, usize::MAX as u64) as usize);
        }
        page_ids = pages.iter().map(|page| page.page_id).collect();
    }

    let revision_fields_requested =
        fields.title || fields.alt_title || fields.tags || fields.updated_by;
    let revisions_by_id: BTreeMap<i64, page_revision::Model> =
        if revision_fields_requested {
            let revision_ids = pages
                .iter()
                .filter_map(|page| page.latest_revision_id)
                .collect::<Vec<_>>();

            if revision_ids.is_empty() {
                BTreeMap::new()
            } else {
                page_revision::Entity::find()
                    .filter(page_revision::Column::RevisionId.is_in(revision_ids))
                    .all(txn)
                    .await
                    .or_raise(make_error)?
                    .into_iter()
                    .map(|revision| (revision.revision_id, revision))
                    .collect()
            }
        } else {
            BTreeMap::new()
        };

    let created_by_by_page_id: BTreeMap<i64, i64> =
        if fields.created_by && !page_ids.is_empty() {
            let mut created_by_by_page_id = BTreeMap::new();
            for (page_id, user_id) in page_revision::Entity::find()
                .select_only()
                .column(page_revision::Column::PageId)
                .column(page_revision::Column::UserId)
                .filter(page_revision::Column::PageId.is_in(page_ids))
                .order_by_asc(page_revision::Column::PageId)
                .order_by_asc(page_revision::Column::RevisionNumber)
                .order_by_asc(page_revision::Column::RevisionId)
                .into_tuple::<(i64, i64)>()
                .all(txn)
                .await
                .or_raise(make_error)?
            {
                created_by_by_page_id.entry(page_id).or_insert(user_id);
            }
            created_by_by_page_id
        } else {
            BTreeMap::new()
        };

    let rows = pages
        .into_iter()
        .map(|page| {
            let revision = page
                .latest_revision_id
                .and_then(|revision_id| revisions_by_id.get(&revision_id));

            FoundPageRow {
                page_id: page.page_id,
                site_id: page.site_id,
                slug: fields.slug.then_some(page.slug),
                page_category_id: fields
                    .page_category_id
                    .then_some(page.page_category_id),
                page_revision_id: fields
                    .page_revision_id
                    .then_some(page.latest_revision_id)
                    .flatten(),
                created_at: fields.created_at.then_some(page.created_at),
                updated_at: fields.updated_at.then_some(page.updated_at).flatten(),
                title: fields
                    .title
                    .then(|| revision.map(|revision| revision.title.clone()))
                    .flatten(),
                alt_title: fields
                    .alt_title
                    .then(|| revision.and_then(|revision| revision.alt_title.clone()))
                    .flatten(),
                tags: fields
                    .tags
                    .then(|| revision.map(|revision| revision.tags.clone()))
                    .flatten(),
                created_by: fields
                    .created_by
                    .then(|| created_by_by_page_id.get(&page.page_id).copied())
                    .flatten(),
                updated_by: fields
                    .updated_by
                    .then(|| revision.map(|revision| revision.user_id))
                    .flatten(),
                score: fields
                    .score
                    .then(|| score_by_page_id.get(&page.page_id).copied().or(Some(0.0)))
                    .flatten(),
            }
        })
        .collect();

    let pages = FoundPages { pages: rows };
    if filtering_deferred_to_rust || ordering_deferred_to_rust || cap_exceeded {
        return Ok(PageQueryResultEnvelope::deferred(
            pages,
            candidate_count,
            filtering_deferred_to_rust,
            ordering_deferred_to_rust,
            cap_exceeded,
        ));
    }

    Ok(PageQueryResultEnvelope {
        pages,
        metadata: PageQueryResultMetadata {
            candidate_count,
            cap_exceeded,
            sql_limit_offset_applied,
            filtering_deferred_to_rust,
            ordering_deferred_to_rust,
            exact_count_safe: true,
            unsupported_reason: None,
        },
    })
}

fn wikidot_name_pattern(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '*' | '%' => pattern.push('%'),
            '_' => pattern.push_str("\\_"),
            '\\' => pattern.push_str("\\\\"),
            _ => pattern.push(character),
        }
    }
    pattern
}

fn date_selector_condition(column: page::Column, selector: DateSelector) -> Condition {
    match selector {
        DateSelector::FromPresent { start }
            if start == time::OffsetDateTime::UNIX_EPOCH =>
        {
            Condition::all()
        }
        DateSelector::FromPresent { start } => Condition::all().add(column.gte(start)),
        DateSelector::Span {
            timestamp,
            resolution,
            comparison,
        } => {
            let (start, end) = date_span_bounds(timestamp, resolution);
            match (comparison, end) {
                (ComparisonOperation::GreaterThan, Some(end)) => {
                    Condition::all().add(column.gte(end))
                }
                (ComparisonOperation::GreaterThan, None) => {
                    Condition::all().add(Expr::cust("FALSE"))
                }
                (ComparisonOperation::LessThan, _) => {
                    Condition::all().add(column.lt(start))
                }
                (ComparisonOperation::GreaterOrEqualThan, _) => {
                    Condition::all().add(column.gte(start))
                }
                (ComparisonOperation::LessOrEqualThan, Some(end)) => {
                    Condition::all().add(column.lt(end))
                }
                (ComparisonOperation::LessOrEqualThan, None) => {
                    Condition::all().add(column.is_not_null())
                }
                (ComparisonOperation::Equal, Some(end)) => {
                    Condition::all().add(column.gte(start)).add(column.lt(end))
                }
                (ComparisonOperation::Equal, None) => {
                    Condition::all().add(column.gte(start))
                }
                (ComparisonOperation::NotEqual, Some(end)) => {
                    Condition::any().add(column.lt(start)).add(column.gte(end))
                }
                (ComparisonOperation::NotEqual, None) => {
                    Condition::all().add(column.lt(start))
                }
            }
        }
    }
}

fn date_span_bounds(
    timestamp: time::OffsetDateTime,
    resolution: DateTimeResolution,
) -> (time::OffsetDateTime, Option<time::OffsetDateTime>) {
    let start = match resolution {
        DateTimeResolution::Second => timestamp.replace_nanosecond(0).unwrap(),
        DateTimeResolution::Minute => timestamp
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap(),
        DateTimeResolution::Hour => timestamp
            .replace_minute(0)
            .unwrap()
            .replace_second(0)
            .unwrap()
            .replace_nanosecond(0)
            .unwrap(),
        DateTimeResolution::Day => timestamp
            .date()
            .with_time(time::Time::MIDNIGHT)
            .assume_offset(timestamp.offset()),
        DateTimeResolution::Month => {
            time::Date::from_calendar_date(timestamp.year(), timestamp.month(), 1)
                .unwrap()
                .with_time(time::Time::MIDNIGHT)
                .assume_offset(timestamp.offset())
        }
        DateTimeResolution::Year => {
            time::Date::from_calendar_date(timestamp.year(), time::Month::January, 1)
                .unwrap()
                .with_time(time::Time::MIDNIGHT)
                .assume_offset(timestamp.offset())
        }
    };
    let end = match resolution {
        DateTimeResolution::Second => start.checked_add(time::Duration::SECOND),
        DateTimeResolution::Minute => start.checked_add(time::Duration::MINUTE),
        DateTimeResolution::Hour => start.checked_add(time::Duration::HOUR),
        DateTimeResolution::Day => start.checked_add(time::Duration::DAY),
        DateTimeResolution::Month => {
            let (year, month) = if start.month() == time::Month::December {
                (start.year().saturating_add(1), time::Month::January)
            } else {
                (start.year(), start.month().next())
            };
            time::Date::from_calendar_date(year, month, 1)
                .ok()
                .map(|date| {
                    date.with_time(time::Time::MIDNIGHT)
                        .assume_offset(start.offset())
                })
        }
        DateTimeResolution::Year => time::Date::from_calendar_date(
            start.year().saturating_add(1),
            time::Month::January,
            1,
        )
        .ok()
        .map(|date| {
            date.with_time(time::Time::MIDNIGHT)
                .assume_offset(start.offset())
        }),
    };
    (start, end)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ScoreFilterPlan {
    SiteWide { site_id: i64 },
    CandidateCorrelated,
}

const MAX_CORRELATED_SCORE_CANDIDATES: usize = 512;

fn score_filter_plan_from_probe(
    site_id: i64,
    observed_candidates: usize,
) -> ScoreFilterPlan {
    if observed_candidates <= MAX_CORRELATED_SCORE_CANDIDATES {
        ScoreFilterPlan::CandidateCorrelated
    } else {
        ScoreFilterPlan::SiteWide { site_id }
    }
}

fn score_comparison_operator(comparison: ComparisonOperation) -> &'static str {
    match comparison {
        ComparisonOperation::GreaterThan => ">",
        ComparisonOperation::LessThan => "<",
        ComparisonOperation::GreaterOrEqualThan => ">=",
        ComparisonOperation::LessOrEqualThan => "<=",
        ComparisonOperation::Equal => "=",
        ComparisonOperation::NotEqual => "!=",
    }
}

fn score_selector_value(selector: &ScoreSelector) -> Value {
    match selector.score {
        ScoreValue::Integer(value) => Value::BigInt(Some(value)),
        ScoreValue::Float(value) => Value::Double(Some(value)),
    }
}

fn score_selectors_condition(
    selectors: &[ScoreSelector],
    plan: ScoreFilterPlan,
) -> SimpleExpr {
    debug_assert!(!selectors.is_empty());
    match plan {
        ScoreFilterPlan::SiteWide { site_id } => {
            let conditions = selectors
                .iter()
                .enumerate()
                .map(|(index, selector)| {
                    format!(
                        "filtered_score.effective_score {} ${}",
                        score_comparison_operator(selector.comparison),
                        index + 2,
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND ");
            let values = std::iter::once(Value::BigInt(Some(site_id)))
                .chain(selectors.iter().map(score_selector_value))
                .collect::<Vec<_>>();
            Expr::cust_with_values(
                format!(
                    "page.page_id IN (\
                        SELECT filtered_score.page_id \
                        FROM (\
                            SELECT scored_page.page_id, \
                                CASE \
                                    WHEN COALESCE(score_category.rating_type, score_default_category.rating_type, 'plus_minus') = 'stars' \
                                    THEN COALESCE(AVG(score_vote.value) FILTER (WHERE score_vote.rating_system = 'stars'), 0) \
                                    ELSE COALESCE(score_snapshot.imported_rating, 0) + COALESCE(SUM(score_vote.value) FILTER (WHERE score_vote.rating_system = 'points' AND (score_snapshot.imported_rating IS NULL OR score_vote.from_wikidot = FALSE)), 0) \
                                END AS effective_score \
                            FROM page scored_page \
                            JOIN page_category score_category \
                                ON score_category.category_id = scored_page.page_category_id \
                            LEFT JOIN page_category score_default_category \
                                ON score_default_category.site_id = scored_page.site_id \
                                AND score_default_category.slug = '_default' \
                            LEFT JOIN wikidot_page_snapshot score_snapshot \
                                ON score_snapshot.page_id = scored_page.page_id \
                            LEFT JOIN page_vote score_vote \
                                ON score_vote.page_id = scored_page.page_id \
                                AND score_vote.deleted_at IS NULL \
                                AND score_vote.disabled_at IS NULL \
                            WHERE scored_page.site_id = $1 \
                                AND scored_page.deleted_at IS NULL \
                            GROUP BY scored_page.page_id, score_snapshot.imported_rating, score_category.rating_type, score_default_category.rating_type\
                        ) filtered_score \
                        WHERE {conditions}\
                    )"
                ),
                values,
            )
        }
        ScoreFilterPlan::CandidateCorrelated => {
            let conditions = selectors
                .iter()
                .enumerate()
                .map(|(index, selector)| {
                    format!(
                        "filtered_score.effective_score {} ${}",
                        score_comparison_operator(selector.comparison),
                        index + 1,
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND ");
            let values = selectors
                .iter()
                .map(score_selector_value)
                .collect::<Vec<_>>();
            Expr::cust_with_values(
                format!(
                    "EXISTS (\
                        SELECT 1 \
                        FROM (\
                            SELECT CASE \
                                WHEN COALESCE(score_category.rating_type, score_default_category.rating_type, 'plus_minus') = 'stars' \
                                THEN COALESCE(AVG(score_vote.value) FILTER (WHERE score_vote.rating_system = 'stars'), 0) \
                                ELSE COALESCE(score_snapshot.imported_rating, 0) + COALESCE(SUM(score_vote.value) FILTER (WHERE score_vote.rating_system = 'points' AND (score_snapshot.imported_rating IS NULL OR score_vote.from_wikidot = FALSE)), 0) \
                            END AS effective_score \
                            FROM page scored_page \
                            JOIN page_category score_category \
                                ON score_category.category_id = scored_page.page_category_id \
                            LEFT JOIN page_category score_default_category \
                                ON score_default_category.site_id = scored_page.site_id \
                                AND score_default_category.slug = '_default' \
                            LEFT JOIN wikidot_page_snapshot score_snapshot \
                                ON score_snapshot.page_id = scored_page.page_id \
                            LEFT JOIN page_vote score_vote \
                                ON score_vote.page_id = scored_page.page_id \
                                AND score_vote.deleted_at IS NULL \
                                AND score_vote.disabled_at IS NULL \
                            WHERE scored_page.page_id = page.page_id \
                            GROUP BY scored_page.page_id, score_snapshot.imported_rating, score_category.rating_type, score_default_category.rating_type\
                        ) filtered_score \
                        WHERE {conditions}\
                    )"
                ),
                values,
            )
        }
    }
}

fn score_membership_condition(membership: ScoreFilterMembership) -> SimpleExpr {
    let (operator, page_ids) = match membership {
        ScoreFilterMembership::Included(page_ids) => ("$1 = ANY($2)", page_ids),
        ScoreFilterMembership::Excluded(page_ids) => ("$1 != ALL($2)", page_ids),
    };
    Expr::cust_with_exprs(
        operator,
        [Expr::col((Page, page::Column::PageId)), Expr::val(page_ids)],
    )
}

fn zero_satisfies_score_selector(selector: &ScoreSelector) -> bool {
    match selector.score {
        ScoreValue::Integer(value) => match selector.comparison {
            ComparisonOperation::GreaterThan => 0 > value,
            ComparisonOperation::LessThan => 0 < value,
            ComparisonOperation::GreaterOrEqualThan => 0 >= value,
            ComparisonOperation::LessOrEqualThan => 0 <= value,
            ComparisonOperation::Equal => value == 0,
            ComparisonOperation::NotEqual => value != 0,
        },
        ScoreValue::Float(value) if value.is_finite() => match selector.comparison {
            ComparisonOperation::GreaterThan => 0.0 > value,
            ComparisonOperation::LessThan => 0.0 < value,
            ComparisonOperation::GreaterOrEqualThan => 0.0 >= value,
            ComparisonOperation::LessOrEqualThan => 0.0 <= value,
            ComparisonOperation::Equal => value == 0.0,
            ComparisonOperation::NotEqual => value != 0.0,
        },
        ScoreValue::Float(_) => false,
    }
}

fn zero_satisfies_score_selectors(selectors: &[ScoreSelector]) -> bool {
    selectors.iter().all(zero_satisfies_score_selector)
}

fn score_membership_polarity_order(selectors: &[ScoreSelector]) -> [bool; 2] {
    let prefer_excluded = zero_satisfies_score_selectors(selectors);
    [prefer_excluded, !prefer_excluded]
}

async fn score_membership_page_ids(
    txn: &DatabaseTransaction,
    selectors: &[ScoreSelector],
    site_id: i64,
    excluded: bool,
) -> Result<Option<Vec<i64>>> {
    let score_condition =
        score_selectors_condition(selectors, ScoreFilterPlan::SiteWide { site_id });
    let page_ids = Page::find()
        .select_only()
        .column(page::Column::PageId)
        .filter(page::Column::SiteId.eq(site_id))
        .filter(page::Column::DeletedAt.is_null())
        .filter(if excluded {
            score_condition.not()
        } else {
            score_condition
        })
        .limit((MAX_CACHED_SCORE_FILTER_PAGE_IDS + 1) as u64)
        .into_tuple::<i64>()
        .all(txn)
        .await
        .or_raise(|| {
            Error::new(
                "failed to materialize ListPages score filter",
                ErrorType::PageQuery,
            )
        })?;
    Ok(bounded_score_page_ids(page_ids))
}

async fn materialize_score_membership(
    txn: &DatabaseTransaction,
    selectors: &[ScoreSelector],
    site_id: i64,
) -> Result<Option<ScoreFilterMembership>> {
    for excluded in score_membership_polarity_order(selectors) {
        if let Some(page_ids) =
            score_membership_page_ids(txn, selectors, site_id, excluded).await?
        {
            return Ok(Some(if excluded {
                ScoreFilterMembership::Excluded(page_ids)
            } else {
                ScoreFilterMembership::Included(page_ids)
            }));
        }
    }
    Ok(None)
}

fn bounded_score_page_ids(page_ids: Vec<i64>) -> Option<Vec<i64>> {
    if page_ids.len() > MAX_CACHED_SCORE_FILTER_PAGE_IDS {
        None
    } else {
        Some(page_ids)
    }
}

#[cfg(test)]
fn score_selector_condition(
    selector: &ScoreSelector,
    plan: ScoreFilterPlan,
) -> SimpleExpr {
    score_selectors_condition(std::slice::from_ref(selector), plan)
}

fn postgres_bind_placeholders(count: usize) -> String {
    (1..=count)
        .map(|index| format!("${index}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn score_to_f32(score: ScoreValue) -> f32 {
    match score {
        ScoreValue::Integer(value) => value as f32,
        ScoreValue::Float(value) => value as f32,
    }
}

async fn filter_pages_by_data_form_fields(
    ctx: &ServiceContext<'_>,
    pages: Vec<page::Model>,
    selectors: &[DataFormSelector<'_>],
) -> Result<Vec<page::Model>> {
    let make_error = || {
        Error::new(
            "failed to filter ListPages data form selectors",
            ErrorType::PageQuery,
        )
    };
    let revision_ids = pages
        .iter()
        .filter_map(|page| page.latest_revision_id)
        .collect::<Vec<_>>();
    if revision_ids.is_empty() {
        return Ok(Vec::new());
    }

    let revisions_by_id = page_revision::Entity::find()
        .filter(page_revision::Column::RevisionId.is_in(revision_ids))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|revision| (revision.revision_id, revision.wikitext_hash))
        .collect::<BTreeMap<_, _>>();

    let hashes = revisions_by_id.values().cloned().collect::<Vec<_>>();
    let text_by_hash = text::Entity::find()
        .filter(text::Column::Hash.is_in(hashes))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|text| (text.hash, text.contents))
        .collect::<BTreeMap<_, _>>();

    Ok(pages
        .into_iter()
        .filter(|page| {
            let values = page
                .latest_revision_id
                .and_then(|revision_id| revisions_by_id.get(&revision_id))
                .and_then(|hash| text_by_hash.get(hash))
                .map(|wikitext| parse_static_wikidot_data_form_values(wikitext))
                .unwrap_or_default();

            static_wikidot_data_form_matches(&values, selectors)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_CACHED_SCORE_FILTER_PAGE_IDS, MAX_CORRELATED_SCORE_CANDIDATES,
        MAX_TOTAL_CACHED_SCORE_FILTER_PAGE_IDS, PageQueryScoreFilterCache,
        PageQueryScoreFilterSession, ScoreFilterCacheKey, ScoreFilterCacheLookup,
        ScoreFilterMembership, ScoreFilterPlan, bounded_score_page_ids, date_span_bounds,
        score_filter_plan_from_probe, score_membership_condition,
        score_membership_polarity_order, score_selector_condition,
        score_selectors_condition, wikidot_name_pattern,
    };
    use crate::models::page;
    use crate::services::page_query::{
        ComparisonOperation, DateTimeResolution, ScoreSelector,
    };
    use crate::services::score::ScoreValue;
    use sea_orm::{
        DatabaseBackend, EntityTrait, ExprTrait, QueryFilter, QueryOrder, QueryTrait,
        Value,
    };
    use sea_query::{SimpleExpr, func::Func};

    #[test]
    fn wikidot_name_patterns_translate_both_wildcard_spellings() {
        assert_eq!(wikidot_name_pattern("scp-*"), "scp-%");
        assert_eq!(wikidot_name_pattern("fragment:part%"), "fragment:part%");
        assert_eq!(wikidot_name_pattern("literal_name"), "literal\\_name");
    }

    #[test]
    fn month_date_spans_use_calendar_boundaries() {
        let timestamp = time::Date::from_calendar_date(2026, time::Month::June, 17)
            .unwrap()
            .with_time(time::Time::from_hms(12, 34, 56).unwrap())
            .assume_utc();
        let (start, end) = date_span_bounds(timestamp, DateTimeResolution::Month);
        let end = end.expect("ordinary month should have a representable upper bound");

        assert_eq!(
            start.date(),
            time::Date::from_calendar_date(2026, time::Month::June, 1).unwrap(),
        );
        assert_eq!(
            end.date(),
            time::Date::from_calendar_date(2026, time::Month::July, 1).unwrap(),
        );
        assert_eq!(start.time(), time::Time::MIDNIGHT);
        assert_eq!(end.time(), time::Time::MIDNIGHT);
    }

    #[test]
    fn maximum_year_date_spans_use_an_open_upper_bound() {
        let timestamp = time::Date::from_calendar_date(9999, time::Month::December, 31)
            .unwrap()
            .with_time(time::Time::from_hms(23, 59, 59).unwrap())
            .assume_utc();

        for resolution in [
            DateTimeResolution::Second,
            DateTimeResolution::Day,
            DateTimeResolution::Month,
            DateTimeResolution::Year,
        ] {
            let (_, end) = date_span_bounds(timestamp, resolution);
            assert_eq!(end, None, "resolution {resolution:?}");
        }

        let equal = page::Entity::find()
            .filter(super::date_selector_condition(
                page::Column::CreatedAt,
                crate::services::page_query::DateSelector::Span {
                    timestamp,
                    resolution: DateTimeResolution::Year,
                    comparison: ComparisonOperation::Equal,
                },
            ))
            .build(DatabaseBackend::Postgres);
        assert!(equal.sql.contains("\"created_at\" >="), "{}", equal.sql);
        assert!(!equal.sql.contains("\"created_at\" <"), "{}", equal.sql);

        let greater = page::Entity::find()
            .filter(super::date_selector_condition(
                page::Column::CreatedAt,
                crate::services::page_query::DateSelector::Span {
                    timestamp,
                    resolution: DateTimeResolution::Year,
                    comparison: ComparisonOperation::GreaterThan,
                },
            ))
            .build(DatabaseBackend::Postgres);
        assert!(greater.sql.contains("FALSE"), "{}", greater.sql);
    }

    #[test]
    fn broad_score_filter_aggregates_site_votes_once() {
        let selector = ScoreSelector {
            score: ScoreValue::Integer(90),
            comparison: ComparisonOperation::Equal,
        };
        let statement = page::Entity::find()
            .filter(score_selector_condition(
                &selector,
                ScoreFilterPlan::SiteWide { site_id: 6_000_006 },
            ))
            .build(DatabaseBackend::Postgres);

        assert!(
            statement
                .sql
                .contains("page.page_id IN (SELECT filtered_score.page_id")
        );
        assert!(statement.sql.contains("WHERE scored_page.site_id = $1"));
        assert!(
            statement
                .sql
                .contains("GROUP BY scored_page.page_id, score_snapshot.imported_rating, score_category.rating_type, score_default_category.rating_type")
        );
        assert!(statement.sql.contains("score_vote.rating_system = 'stars'"));
        assert!(
            statement
                .sql
                .contains("score_vote.rating_system = 'points'")
        );
        assert!(
            statement
                .sql
                .contains("WHERE filtered_score.effective_score = $2")
        );
        assert!(
            !statement
                .sql
                .contains("WHERE snapshot.page_id = page.page_id")
        );
    }

    #[test]
    fn score_filter_preserves_integer_and_float_bind_types() {
        const SITE_ID: i64 = 6_000_006;
        const INTEGER_THRESHOLD: i64 = 9_007_199_254_740_993;

        for (plan, expected_values) in [
            (
                ScoreFilterPlan::CandidateCorrelated,
                vec![Value::BigInt(Some(INTEGER_THRESHOLD))],
            ),
            (
                ScoreFilterPlan::SiteWide { site_id: SITE_ID },
                vec![
                    Value::BigInt(Some(SITE_ID)),
                    Value::BigInt(Some(INTEGER_THRESHOLD)),
                ],
            ),
        ] {
            let selector = ScoreSelector {
                score: ScoreValue::Integer(INTEGER_THRESHOLD),
                comparison: ComparisonOperation::Equal,
            };
            let statement = page::Entity::find()
                .filter(score_selector_condition(&selector, plan))
                .build(DatabaseBackend::Postgres);

            assert_eq!(statement.values.unwrap().0, expected_values);
        }

        for (plan, expected_values) in [
            (
                ScoreFilterPlan::CandidateCorrelated,
                vec![Value::Double(Some(1.5))],
            ),
            (
                ScoreFilterPlan::SiteWide { site_id: SITE_ID },
                vec![Value::BigInt(Some(SITE_ID)), Value::Double(Some(1.5))],
            ),
        ] {
            let selector = ScoreSelector {
                score: ScoreValue::Float(1.5),
                comparison: ComparisonOperation::Equal,
            };
            let statement = page::Entity::find()
                .filter(score_selector_condition(&selector, plan))
                .build(DatabaseBackend::Postgres);

            assert_eq!(statement.values.unwrap().0, expected_values);
        }
    }

    #[test]
    fn score_filter_plan_uses_capped_probe_boundary() {
        for observed_candidates in [0, 78, 171, 183, MAX_CORRELATED_SCORE_CANDIDATES] {
            assert_eq!(
                score_filter_plan_from_probe(6_000_006, observed_candidates),
                ScoreFilterPlan::CandidateCorrelated,
            );
        }

        for observed_candidates in [MAX_CORRELATED_SCORE_CANDIDATES + 1, 20_492, 24_436] {
            assert_eq!(
                score_filter_plan_from_probe(6_000_006, observed_candidates),
                ScoreFilterPlan::SiteWide { site_id: 6_000_006 },
            );
        }
    }

    #[test]
    fn selectively_prefiltered_score_filter_correlates_to_candidate() {
        let selector = ScoreSelector {
            score: ScoreValue::Integer(-4),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        };
        let statement = page::Entity::find()
            .filter(score_selector_condition(
                &selector,
                ScoreFilterPlan::CandidateCorrelated,
            ))
            .build(DatabaseBackend::Postgres);

        assert!(statement.sql.contains("EXISTS (SELECT 1 FROM (SELECT"));
        assert!(
            statement
                .sql
                .contains("WHERE scored_page.page_id = page.page_id")
        );
        assert!(
            statement
                .sql
                .contains("WHERE filtered_score.effective_score >= $1")
        );
        assert!(!statement.sql.contains("WHERE scored_page.site_id = $1"));
    }

    #[test]
    fn repeated_score_selectors_share_one_aggregate_and_preserve_bind_order() {
        const SITE_ID: i64 = 6_000_006;
        let selectors = [
            ScoreSelector {
                score: ScoreValue::Integer(-4),
                comparison: ComparisonOperation::GreaterOrEqualThan,
            },
            ScoreSelector {
                score: ScoreValue::Float(1.5),
                comparison: ComparisonOperation::LessThan,
            },
        ];

        for (plan, expected_values, expected_conditions) in [
            (
                ScoreFilterPlan::CandidateCorrelated,
                vec![Value::BigInt(Some(-4)), Value::Double(Some(1.5))],
                "filtered_score.effective_score >= $1 AND filtered_score.effective_score < $2",
            ),
            (
                ScoreFilterPlan::SiteWide { site_id: SITE_ID },
                vec![
                    Value::BigInt(Some(SITE_ID)),
                    Value::BigInt(Some(-4)),
                    Value::Double(Some(1.5)),
                ],
                "filtered_score.effective_score >= $2 AND filtered_score.effective_score < $3",
            ),
        ] {
            let statement = page::Entity::find()
                .filter(score_selectors_condition(&selectors, plan))
                .build(DatabaseBackend::Postgres);

            assert_eq!(statement.sql.matches("SUM(score_vote.value)").count(), 1);
            assert_eq!(statement.sql.matches("AVG(score_vote.value)").count(), 1);
            assert_eq!(statement.sql.matches("FROM page scored_page").count(), 1);
            assert!(
                statement.sql.contains(expected_conditions),
                "{}",
                statement.sql
            );
            assert_eq!(statement.values.unwrap().0, expected_values);
        }
    }

    #[test]
    fn repeated_site_wide_score_key_materializes_once() {
        let selectors = [ScoreSelector {
            score: ScoreValue::Integer(30),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let key = ScoreFilterCacheKey::new(6_000_006, &selectors);
        let mut cache = PageQueryScoreFilterCache::default();

        assert_eq!(cache.lookup(&key, true), ScoreFilterCacheLookup::FirstUse);
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::RepeatedUnmaterialized,
        );
        cache.insert(key.clone(), ScoreFilterMembership::Included(vec![11, 22]));
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::Materialized(ScoreFilterMembership::Included(vec![
                11, 22
            ]),),
        );
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::Materialized(ScoreFilterMembership::Included(vec![
                11, 22
            ]),),
        );
    }

    #[test]
    fn score_filter_session_counts_batches_as_one_logical_use() {
        let selectors = [ScoreSelector {
            score: ScoreValue::Integer(-10),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        }];
        let key = ScoreFilterCacheKey::new(6_000_006, &selectors);
        let mut cache = PageQueryScoreFilterCache::default();
        let mut first_module = PageQueryScoreFilterSession::default();

        assert!(first_module.register_use(&key));
        assert_eq!(cache.lookup(&key, true), ScoreFilterCacheLookup::FirstUse);
        assert!(!first_module.register_use(&key));
        assert_eq!(cache.lookup(&key, false), ScoreFilterCacheLookup::FirstUse);

        let mut second_module = PageQueryScoreFilterSession::default();
        assert!(second_module.register_use(&key));
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::RepeatedUnmaterialized,
        );
    }

    #[test]
    fn score_cache_separates_sites_comparisons_and_numeric_types() {
        let integer = [ScoreSelector {
            score: ScoreValue::Integer(30),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let float = [ScoreSelector {
            score: ScoreValue::Float(30.0),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let strict = [ScoreSelector {
            score: ScoreValue::Integer(30),
            comparison: ComparisonOperation::LessThan,
        }];
        let mut cache = PageQueryScoreFilterCache::default();

        for key in [
            ScoreFilterCacheKey::new(1, &integer),
            ScoreFilterCacheKey::new(2, &integer),
            ScoreFilterCacheKey::new(1, &float),
            ScoreFilterCacheKey::new(1, &strict),
        ] {
            assert_eq!(cache.lookup(&key, true), ScoreFilterCacheLookup::FirstUse);
        }
    }

    #[test]
    fn cached_score_memberships_use_one_typed_array_predicate() {
        for (membership, operator, page_ids) in [
            (
                ScoreFilterMembership::Included(Vec::<i64>::new()),
                "= ANY",
                Vec::<i64>::new(),
            ),
            (
                ScoreFilterMembership::Included(vec![11, 22]),
                "= ANY",
                vec![11_i64, 22],
            ),
            (
                ScoreFilterMembership::Excluded(Vec::<i64>::new()),
                "!= ALL",
                Vec::<i64>::new(),
            ),
            (
                ScoreFilterMembership::Excluded(vec![11, 22]),
                "!= ALL",
                vec![11_i64, 22],
            ),
        ] {
            let statement = page::Entity::find()
                .filter(score_membership_condition(membership))
                .build(DatabaseBackend::Postgres);

            assert!(
                statement
                    .sql
                    .contains(&format!("\"page\".\"page_id\" {operator}($1)")),
                "{}",
                statement.sql,
            );
            assert_eq!(statement.values.unwrap().0, vec![Value::from(page_ids)]);
            assert!(!statement.sql.contains("random"));
        }
    }

    #[test]
    fn score_membership_prefers_the_side_containing_fewer_zero_scores() {
        let broad = [ScoreSelector {
            score: ScoreValue::Integer(-10),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        }];
        let narrow = [ScoreSelector {
            score: ScoreValue::Integer(10),
            comparison: ComparisonOperation::GreaterThan,
        }];
        let bounded_range = [
            ScoreSelector {
                score: ScoreValue::Float(-0.5),
                comparison: ComparisonOperation::GreaterThan,
            },
            ScoreSelector {
                score: ScoreValue::Float(0.5),
                comparison: ComparisonOperation::LessThan,
            },
        ];

        assert_eq!(score_membership_polarity_order(&broad), [true, false]);
        assert_eq!(score_membership_polarity_order(&narrow), [false, true]);
        assert_eq!(
            score_membership_polarity_order(&bounded_range),
            [true, false],
        );
    }

    #[test]
    fn excluded_score_membership_negates_the_complete_selector_conjunction() {
        let selectors = [
            ScoreSelector {
                score: ScoreValue::Integer(-10),
                comparison: ComparisonOperation::GreaterOrEqualThan,
            },
            ScoreSelector {
                score: ScoreValue::Integer(30),
                comparison: ComparisonOperation::LessOrEqualThan,
            },
        ];
        let statement = page::Entity::find()
            .filter(
                score_selectors_condition(
                    &selectors,
                    ScoreFilterPlan::SiteWide { site_id: 6_000_006 },
                )
                .not(),
            )
            .build(DatabaseBackend::Postgres);

        assert!(
            statement.sql.contains("NOT (page.page_id IN"),
            "{}",
            statement.sql,
        );
        assert!(
            statement.sql.contains(
                "effective_score >= $2 AND filtered_score.effective_score <= $3"
            )
        );
        assert_eq!(
            statement.values.unwrap().0,
            vec![
                Value::BigInt(Some(6_000_006)),
                Value::BigInt(Some(-10)),
                Value::BigInt(Some(30))
            ],
        );
    }

    #[test]
    fn materialized_score_ids_are_available_before_probe_without_state_updates() {
        let selectors = [ScoreSelector {
            score: ScoreValue::Integer(-10),
            comparison: ComparisonOperation::GreaterOrEqualThan,
        }];
        let key = ScoreFilterCacheKey::new(6_000_006, &selectors);
        let mut cache = PageQueryScoreFilterCache::default();
        let session = PageQueryScoreFilterSession::default();

        assert_eq!(cache.materialized_membership(&key), None);
        assert_eq!(cache.lookup(&key, true), ScoreFilterCacheLookup::FirstUse);
        assert_eq!(cache.materialized_membership(&key), None);
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::RepeatedUnmaterialized,
        );
        assert_eq!(cache.materialized_membership(&key), None);
        cache.insert(key.clone(), ScoreFilterMembership::Excluded(vec![11, 22]));
        let seen_before = cache.seen.clone();

        let membership = cache
            .materialized_membership(&key)
            .expect("materialized membership should bypass the candidate probe");
        let statement = page::Entity::find()
            .filter(score_membership_condition(membership))
            .build(DatabaseBackend::Postgres);

        assert!(statement.sql.contains("\"page\".\"page_id\" != ALL($1)"));
        assert_eq!(
            statement.values.unwrap().0,
            vec![Value::from(vec![11_i64, 22])]
        );
        assert_eq!(cache.seen, seen_before);
        assert!(session.seen.is_empty());
    }

    #[test]
    fn cached_score_ids_do_not_replace_independent_random_ordering() {
        let statement = page::Entity::find()
            .filter(score_membership_condition(ScoreFilterMembership::Included(
                vec![11, 22],
            )))
            .order_by_desc(SimpleExpr::FunctionCall(Func::random()))
            .build(DatabaseBackend::Postgres);

        assert!(statement.sql.contains("\"page\".\"page_id\" = ANY($1)"));
        assert!(statement.sql.contains("ORDER BY RANDOM() DESC"));
    }

    #[test]
    fn correlated_score_plan_remains_outside_the_site_wide_cache() {
        let cache = PageQueryScoreFilterCache::default();
        assert_eq!(
            score_filter_plan_from_probe(6_000_006, MAX_CORRELATED_SCORE_CANDIDATES),
            ScoreFilterPlan::CandidateCorrelated,
        );
        assert!(cache.seen.is_empty());
        assert!(cache.memberships.is_empty());
    }

    #[test]
    fn score_cache_id_limit_accepts_boundary_and_rejects_limit_plus_one() {
        let boundary = vec![0; MAX_CACHED_SCORE_FILTER_PAGE_IDS];
        assert_eq!(
            bounded_score_page_ids(boundary).map(|page_ids| page_ids.len()),
            Some(MAX_CACHED_SCORE_FILTER_PAGE_IDS),
        );
        assert!(
            bounded_score_page_ids(vec![0; MAX_CACHED_SCORE_FILTER_PAGE_IDS + 1])
                .is_none()
        );
    }

    #[test]
    fn score_cache_total_id_limit_marks_new_keys_uncacheable() {
        let first_selectors = [ScoreSelector {
            score: ScoreValue::Integer(30),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let second_selectors = [ScoreSelector {
            score: ScoreValue::Integer(31),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let first_key = ScoreFilterCacheKey::new(6_000_006, &first_selectors);
        let second_key = ScoreFilterCacheKey::new(6_000_006, &second_selectors);
        let mut cache = PageQueryScoreFilterCache::default();

        cache.insert(
            first_key.clone(),
            ScoreFilterMembership::Included(vec![1; MAX_CACHED_SCORE_FILTER_PAGE_IDS]),
        );
        cache.insert(
            second_key.clone(),
            ScoreFilterMembership::Included(vec![2; MAX_CACHED_SCORE_FILTER_PAGE_IDS]),
        );
        let overflow_selectors = [ScoreSelector {
            score: ScoreValue::Integer(32),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let overflow_key = ScoreFilterCacheKey::new(6_000_006, &overflow_selectors);
        cache.insert(
            overflow_key.clone(),
            ScoreFilterMembership::Included(vec![3]),
        );

        assert_eq!(
            cache.cached_page_ids,
            MAX_TOTAL_CACHED_SCORE_FILTER_PAGE_IDS
        );
        assert!(cache.memberships.contains_key(&first_key));
        assert!(cache.memberships.contains_key(&second_key));
        assert_eq!(
            cache.lookup(&overflow_key, true),
            ScoreFilterCacheLookup::Uncacheable
        );
        assert_eq!(cache.materialized_membership(&overflow_key), None);
    }

    #[test]
    fn uncacheable_score_key_stays_on_the_site_wide_fallback() {
        let selectors = [ScoreSelector {
            score: ScoreValue::Integer(30),
            comparison: ComparisonOperation::LessOrEqualThan,
        }];
        let key = ScoreFilterCacheKey::new(6_000_006, &selectors);
        let mut cache = PageQueryScoreFilterCache::default();

        assert_eq!(cache.lookup(&key, true), ScoreFilterCacheLookup::FirstUse);
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::RepeatedUnmaterialized,
        );
        cache.insert(key.clone(), ScoreFilterMembership::Included(vec![11, 22]));
        cache.mark_uncacheable(key.clone());
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::Uncacheable
        );
        assert_eq!(
            cache.lookup(&key, true),
            ScoreFilterCacheLookup::Uncacheable
        );
        assert_eq!(cache.materialized_membership(&key), None);
        assert!(cache.memberships.is_empty());
    }
}
