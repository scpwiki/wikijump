/*
 * services/page_query/compat_select.rs
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

use super::PageQueryService;
use super::prelude::*;
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_revision;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{AliasService, SiteService};
use crate::types::{Action, AliasType, Permission, Reference, Resource};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use std::borrow::Cow;
use std::collections::BTreeSet;
use time::OffsetDateTime;

const MAX_FILTER_VALUES: usize = 100;

#[derive(Deserialize, Debug)]
pub struct SelectPageTags<'a> {
    pub site: Reference<'a>,
    pub categories: Option<Vec<String>>,
    pub pages: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct SelectPages<'a> {
    pub site: Reference<'a>,
    pub pagetype: Option<String>,
    pub categories: Option<Vec<String>>,
    pub tags_any: Option<Vec<String>>,
    pub tags_all: Option<Vec<String>>,
    pub tags_none: Option<Vec<String>>,
    pub parent: Option<String>,
    pub created_by: Option<String>,
    pub rating: Option<String>,
    pub order: Option<String>,
}

impl PageQueryService {
    pub async fn select_tags(
        ctx: &ServiceContext<'_>,
        SelectPageTags {
            site,
            categories,
            pages,
        }: SelectPageTags<'_>,
    ) -> Result<Vec<String>> {
        if categories
            .as_ref()
            .is_some_and(|values| values.len() > MAX_FILTER_VALUES)
            || pages
                .as_ref()
                .is_some_and(|values| values.len() > MAX_FILTER_VALUES)
        {
            return Err(Error::new(
                format!(
                    "page tag filters may contain at most {MAX_FILTER_VALUES} values"
                ),
                ErrorType::Request,
            )
            .into());
        }

        let make_error = || Error::new("failed to select page tags", ErrorType::Page);
        let user_id = ctx.request().user_id().or_raise(|| {
            Error::new(
                "page tag selection requires an authenticated request context",
                ErrorType::PermissionDenied,
            )
        })?;
        let site_id = SiteService::get_id(ctx, site).await.or_raise(make_error)?;
        if matches!(categories, Some(ref categories) if categories.is_empty())
            || matches!(pages, Some(ref pages) if pages.is_empty())
        {
            return Ok(Vec::new());
        }

        let category_ids = match categories {
            None => None,
            Some(categories) => {
                let selected_categories = categories.into_iter().collect::<BTreeSet<_>>();
                let category_ids = PageCategory::find()
                    .filter(page_category::Column::SiteId.eq(site_id))
                    .filter(page_category::Column::Slug.is_in(selected_categories))
                    .select_only()
                    .column(page_category::Column::CategoryId)
                    .into_tuple::<i64>()
                    .all(ctx.transaction())
                    .await
                    .or_raise(make_error)?;

                if category_ids.is_empty() {
                    return Ok(Vec::new());
                }

                Some(category_ids)
            }
        };

        let txn = ctx.transaction();
        let mut page_query = Page::find()
            .filter(page::Column::SiteId.eq(site_id))
            .filter(page::Column::DeletedAt.is_null());

        if let Some(category_ids) = category_ids {
            page_query =
                page_query.filter(page::Column::PageCategoryId.is_in(category_ids));
        }
        if let Some(pages) = pages {
            page_query = page_query.filter(page::Column::Slug.is_in(pages));
        }

        let pages = page_query.all(txn).await.or_raise(make_error)?;
        let mut visible_revision_ids = Vec::with_capacity(pages.len());
        for page in pages {
            let can_view = PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: Some(user_id),
                    site_id,
                    page_reference: Some(Reference::Id(page.page_id)),
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(page.page_category_id)),
                    action: Action::View,
                },
            )
            .await
            .or_raise(make_error)?;

            if can_view && let Some(revision_id) = page.latest_revision_id {
                visible_revision_ids.push(revision_id);
            }
        }

        if visible_revision_ids.is_empty() {
            return Ok(Vec::new());
        }

        let tags = page_revision::Entity::find()
            .filter(page_revision::Column::RevisionId.is_in(visible_revision_ids))
            .select_only()
            .column(page_revision::Column::Tags)
            .into_tuple::<Vec<String>>()
            .all(txn)
            .await
            .or_raise(make_error)?
            .into_iter()
            .flatten()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Ok(tags)
    }

    pub async fn select_pages(
        ctx: &ServiceContext<'_>,
        SelectPages {
            site,
            pagetype,
            categories,
            tags_any,
            tags_all,
            tags_none,
            parent,
            created_by,
            rating,
            order,
        }: SelectPages<'_>,
    ) -> Result<Vec<String>> {
        for (name, values) in [
            ("categories", categories.as_ref()),
            ("tags_any", tags_any.as_ref()),
            ("tags_all", tags_all.as_ref()),
            ("tags_none", tags_none.as_ref()),
        ] {
            if values.is_some_and(|values| values.len() > MAX_FILTER_VALUES) {
                return Err(Error::new(
                    format!("page selection filter {name} may contain at most {MAX_FILTER_VALUES} values"),
                    ErrorType::Request,
                )
                .into());
            }
        }

        if matches!(categories, Some(ref values) if values.is_empty())
            || matches!(tags_any, Some(ref values) if values.is_empty())
        {
            return Ok(Vec::new());
        }
        let tags_all = tags_all.filter(|values| !values.is_empty());
        let tags_none = tags_none.filter(|values| !values.is_empty());

        let make_error = || Error::new("failed to select pages", ErrorType::Page);
        let site_id = SiteService::get_id(ctx, site).await.or_raise(make_error)?;
        let parent = normalize_optional(parent);
        let created_by = normalize_optional(created_by);
        let rating = normalize_optional(rating);

        let page_type = parse_page_select_type(pagetype.as_deref())?;
        let order = parse_page_select_order(order.as_deref())?;
        let rating_filter = rating
            .as_deref()
            .map(parse_page_select_rating)
            .transpose()?;

        let created_by = match created_by {
            None => None,
            Some(created_by) => match resolve_created_by(ctx, &created_by).await? {
                Some(user_id) => Some(user_id),
                None => return Ok(Vec::new()),
            },
        };
        let created_by_ids = created_by.into_iter().collect::<Vec<_>>();

        let categories = owned_selector_values(categories);
        let tags_any = owned_selector_values(tags_any);
        let tags_all = owned_selector_values(tags_all);
        let tags_none = owned_selector_values(tags_none);

        let parent_reference;
        let parent_references;
        let page_parent = match parent.as_deref() {
            None => PageParentSelector::All,
            Some("-") => PageParentSelector::NoParent,
            Some(parent) => {
                parent_reference = Reference::Slug(Cow::Borrowed(parent));
                parent_references = [parent_reference];
                PageParentSelector::HasParents(&parent_references)
            }
        };

        let found = Self::find(
            ctx,
            PageQuery {
                current_page_id: 0,
                current_site_id: site_id,
                queried_site_id: Some(site_id),
                page_type,
                categories: CategoriesSelector {
                    included_categories: if categories.is_empty() {
                        IncludedCategories::All
                    } else {
                        IncludedCategories::List(&categories)
                    },
                    excluded_categories: &[],
                },
                tags: TagCondition {
                    any_present: &tags_any,
                    all_present: &tags_all,
                    none_present: &tags_none,
                },
                page_parent,
                contains_outgoing_links: &[],
                creation_date: DateSelector::FromPresent {
                    start: OffsetDateTime::UNIX_EPOCH,
                },
                update_date: DateSelector::FromPresent {
                    start: OffsetDateTime::UNIX_EPOCH,
                },
                author: if created_by_ids.is_empty() {
                    AuthorSelector::All
                } else {
                    AuthorSelector::Any {
                        user_ids: &created_by_ids,
                        wikidot_snapshot_names: &[],
                    }
                },
                score: &[],
                votes: &[],
                offset: 0,
                range: RangeSelector::Current,
                name: None,
                slug: None,
                slugs: &[],
                data_form_fields: &[],
                order: Some(order),
                candidate_limit: None,
                pagination: PaginationSelector::default(),
                variables: &[],
                fields: FoundPageFields {
                    slug: true,
                    score: rating_filter.is_some(),
                    ..FoundPageFields::default()
                },
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(found
            .pages
            .into_iter()
            .filter(|page| {
                rating_filter
                    .as_ref()
                    .is_none_or(|filter| filter.matches(page.score.unwrap_or(0.0)))
            })
            .filter_map(|page| page.slug)
            .collect())
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_owned())
    })
}

fn owned_selector_values(values: Option<Vec<String>>) -> Vec<Cow<'static, str>> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(Cow::Owned)
        .collect()
}

async fn resolve_created_by(
    ctx: &ServiceContext<'_>,
    created_by: &str,
) -> Result<Option<i64>> {
    if let Ok(user_id) = created_by.parse() {
        return Ok(Some(user_id));
    }

    let make_error = || Error::new("failed to resolve page creator", ErrorType::Page);
    Ok(AliasService::get_optional(ctx, AliasType::User, created_by)
        .await
        .or_raise(make_error)?
        .map(|alias| alias.target_id))
}

#[derive(Debug, Copy, Clone)]
enum PageSelectComparison {
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Copy, Clone)]
struct PageSelectRatingFilter {
    comparison: PageSelectComparison,
    value: f32,
}

impl PageSelectRatingFilter {
    fn matches(self, rating: f32) -> bool {
        match self.comparison {
            PageSelectComparison::GreaterThan => rating > self.value,
            PageSelectComparison::GreaterOrEqual => rating >= self.value,
            PageSelectComparison::LessThan => rating < self.value,
            PageSelectComparison::LessOrEqual => rating <= self.value,
            PageSelectComparison::Equal => (rating - self.value).abs() < f32::EPSILON,
            PageSelectComparison::NotEqual => (rating - self.value).abs() >= f32::EPSILON,
        }
    }
}

fn parse_page_select_type(value: Option<&str>) -> Result<PageTypeSelector> {
    match value.unwrap_or("*").trim().to_ascii_lowercase().as_str() {
        "" | "*" | "all" => Ok(PageTypeSelector::All),
        "normal" | "page" | "pages" => Ok(PageTypeSelector::Normal),
        "hidden" => Ok(PageTypeSelector::Hidden),
        other => Err(Error::new(
            format!("unsupported pages.select pagetype: {other}"),
            ErrorType::Page,
        )
        .into()),
    }
}

fn parse_page_select_rating(value: &str) -> Result<PageSelectRatingFilter> {
    let value = value.trim();
    let (comparison, number) = if let Some(number) = value.strip_prefix(">=") {
        (PageSelectComparison::GreaterOrEqual, number)
    } else if let Some(number) = value.strip_prefix("<=") {
        (PageSelectComparison::LessOrEqual, number)
    } else if let Some(number) = value.strip_prefix("!=") {
        (PageSelectComparison::NotEqual, number)
    } else if let Some(number) = value.strip_prefix("==") {
        (PageSelectComparison::Equal, number)
    } else if let Some(number) = value.strip_prefix('>') {
        (PageSelectComparison::GreaterThan, number)
    } else if let Some(number) = value.strip_prefix('<') {
        (PageSelectComparison::LessThan, number)
    } else if let Some(number) = value.strip_prefix('=') {
        (PageSelectComparison::Equal, number)
    } else {
        (PageSelectComparison::Equal, value)
    };

    let value = number.trim().parse::<f32>().map_err(|_| {
        Error::new(
            format!("invalid pages.select rating filter: {value}"),
            ErrorType::Page,
        )
    })?;
    if !value.is_finite() {
        return Err(Error::new(
            format!("invalid pages.select rating filter: {value}"),
            ErrorType::Page,
        )
        .into());
    }

    Ok(PageSelectRatingFilter { comparison, value })
}

fn parse_page_select_order(value: Option<&str>) -> Result<OrderBySelector> {
    let value = match value.map(str::trim) {
        None | Some("") => "created_at desc",
        Some(value) => value,
    };

    let parts = value.split_whitespace().collect::<Vec<_>>();
    let (field, direction) = match parts.as_slice() {
        [field] => (*field, "asc"),
        [field, direction] => (*field, *direction),
        _ => {
            return Err(Error::new(
                format!("invalid pages.select order expression: {value}"),
                ErrorType::Page,
            )
            .into());
        }
    };

    let property = match field.to_ascii_lowercase().as_str() {
        "created_at" | "created" => OrderProperty::CreatedAt,
        "updated_at" | "updated" => OrderProperty::UpdatedAt,
        "fullname" | "full_name" | "slug" | "name" => OrderProperty::FullSlug,
        "title" => OrderProperty::Title,
        "rating" | "score" => OrderProperty::Score,
        other => {
            return Err(Error::new(
                format!("unsupported pages.select order field: {other}"),
                ErrorType::Page,
            )
            .into());
        }
    };

    let ascending = match direction.to_ascii_lowercase().as_str() {
        "asc" | "ascending" => true,
        "desc" | "descending" => false,
        other => {
            return Err(Error::new(
                format!("unsupported pages.select order direction: {other}"),
                ErrorType::Page,
            )
            .into());
        }
    };

    Ok(OrderBySelector {
        property,
        ascending,
    })
}
