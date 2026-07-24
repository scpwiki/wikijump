/*
 * endpoints/page.rs
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

use super::prelude::*;
use crate::models::file::Model as FileModel;
use crate::models::page::{self, Entity as Page, Model as PageModel};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_revision;
use crate::services::TextService;
use crate::services::file::{GetFileOutput, GetPageFiles};
use crate::services::page::{
    CreatePage, CreatePageOutput, DeletePage, DeletePageOutput, EditPage, EditPageOutput,
    GetDeletedPageOutput, GetPageAnyDetails, GetPageOutput, GetPageReference,
    GetPageReferenceDetails, GetPageScoreOutput, GetPageSlug, MovePage, MovePageOutput,
    PageEditPermissionOutput, RestorePage, RestorePageOutput, RollbackPage,
    SetPageLayout,
};
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, DateSelector, FoundPageFields,
    IncludedCategories, OrderBySelector, OrderProperty, PageParentSelector, PageQuery,
    PageQueryService, PageTypeSelector, PaginationSelector, RangeSelector, TagCondition,
};
use crate::services::page_revision::RerenderType;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::{
    Action, AliasType, Bytes, FileOrder, PageDetails, PageId, Permission, Reference,
    RerenderDepth, Resource,
};
use crate::utils::get_category_name;
use futures::future::try_join_all;
use regex::Regex;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;
use time::OffsetDateTime;

static WIKIDOT_LIST_PAGES_SET_PAIR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<span class="set (?P<name_class>[^"]+)"><span class="name">(?P<name>.*?)</span></span><span class="set (?P<value_class>[^"]+)"><span class="value">(?P<value>.*?)</span></span>"#,
    )
    .expect("Wikidot ListPages set-pair expression is valid")
});

#[derive(Deserialize)]
struct WikidotListPagesModuleInput {
    site_id: i64,
    module_body: String,
    parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotListPagesModuleOutput {
    pub body: String,
}

pub async fn wikidot_list_pages_module(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<WikidotListPagesModuleOutput> {
    let input: WikidotListPagesModuleInput = parse!(params, Page);
    let output = RenderService::render_wikidot_list_pages_module(
        ctx,
        input.site_id,
        input.module_body,
        &input.parameters,
    )
    .await
    .or_raise(|| {
        Error::new(
            format!(
                "failed to render Wikidot ListPages module in site ID {}",
                input.site_id,
            ),
            ErrorType::Page,
        )
    })?;

    Ok(WikidotListPagesModuleOutput {
        body: normalize_wikidot_list_pages_set_pairs(&output.html_output.body),
    })
}

/// FTML renders adjacent inline spans as sibling nodes in this module shape.
/// wikidot.py's ListPages parser instead treats the name and value spans as one
/// `set` record, so restore that documented connector-only wire shape here.
fn normalize_wikidot_list_pages_set_pairs(body: &str) -> String {
    WIKIDOT_LIST_PAGES_SET_PAIR_REGEX
        .replace_all(body, |captures: &regex::Captures<'_>| {
            let name_class = captures
                .name("name_class")
                .expect("set-pair name class capture exists")
                .as_str();
            let value_class = captures
                .name("value_class")
                .expect("set-pair value class capture exists")
                .as_str();
            if name_class != value_class {
                return captures
                    .get(0)
                    .expect("set-pair full capture exists")
                    .as_str()
                    .to_owned();
            }
            format!(
                r#"<span class="set {name_class}"><span class="name">{}</span><span class="value">{}</span></span>"#,
                captures
                    .name("name")
                    .expect("set-pair name capture exists")
                    .as_str(),
                captures
                    .name("value")
                    .expect("set-pair value capture exists")
                    .as_str(),
            )
        })
        .into_owned()
}

pub async fn page_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreatePageOutput> {
    let input: CreatePage = parse!(params, Page);
    info!("Creating new page in site ID {}", input.site_id);

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page create actor", ErrorType::Page)
        })?;
    ensure_page_create_permission(ctx, input.site_id, &input.slug, actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page create permission", ErrorType::Page)
        })?;

    PageService::create(ctx, input)
        .await
        .or_raise(|| Error::new("failed to create page", ErrorType::Page))
}

pub async fn page_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetPageOutput>> {
    let GetPageReferenceDetails {
        site_id,
        page: reference,
        details,
    } = parse!(params, Page);

    info!("Getting page {reference:?} in site ID {site_id}");

    let make_error = || Error::new("failed to get page", ErrorType::Page);

    let page = PageService::get_optional(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    match page {
        None => Ok(None),
        Some(page) => build_page_output(ctx, page, details)
            .await
            .or_raise(make_error),
    }
}

pub async fn page_get_direct(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetPageOutput>> {
    let GetPageAnyDetails {
        site_id,
        page_id,
        details,
        allow_deleted,
    } = parse!(params, Page);

    info!("Getting page ID {page_id} in site ID {site_id}");

    let make_error = || {
        Error::new(
            format!("failed to get page ID {} in site ID {}", page_id, site_id),
            ErrorType::Page,
        )
    };

    let page = PageService::get_direct_optional(ctx, page_id, allow_deleted)
        .await
        .or_raise(make_error)?;

    match page {
        None => Ok(None),
        Some(page) => build_page_output(ctx, page, details)
            .await
            .or_raise(make_error),
    }
}

pub async fn page_get_deleted(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<GetDeletedPageOutput>> {
    let GetPageSlug { site_id, slug } = parse!(params, Page);
    let slug2 = slug.clone();

    let make_error = || {
        Error::new(
            format!(
                "failed to get deleted page slug '{}' in site ID {}",
                slug2, site_id
            ),
            ErrorType::Page,
        )
    };

    info!("Getting deleted page {slug} in site ID {site_id}");
    let get_deleted_page = PageService::get_deleted_by_slug(ctx, site_id, &slug)
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|page| build_page_deleted_output(ctx, page));

    let result = try_join_all(get_deleted_page)
        .await
        .or_raise(make_error)?
        .into_iter()
        .flatten()
        .collect();

    Ok(result)
}

pub async fn page_get_score(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetPageScoreOutput> {
    let GetPageReference {
        site_id,
        page: reference,
    } = parse!(params, Page);

    info!("Getting score for page {reference:?} in site ID {site_id}");

    let make_error = || Error::new("failed to get page score", ErrorType::Page);

    let page_id = PageService::get_id(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    let score = ScoreService::score(ctx, page_id)
        .await
        .or_raise(make_error)?;

    Ok(GetPageScoreOutput { page_id, score })
}

pub async fn page_get_files(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<GetFileOutput>> {
    let GetPageFiles {
        page_id,
        site_id,
        deleted,
    } = parse!(params, Page);

    info!("Getting files for page ID {page_id} in site ID {site_id}");

    let make_error = || Error::new("failed to get files for page", ErrorType::Page);

    ensure_page_view_permission(ctx, site_id, page_id)
        .await
        .or_raise(make_error)?;

    let get_page_files = FileService::get_all(
        ctx,
        site_id,
        page_id,
        deleted.to_option().copied(),
        FileOrder::default(),
    )
    .await
    .or_raise(make_error)?
    .into_iter()
    .map(|file| build_page_file_output(ctx, file));

    let result = try_join_all(get_page_files)
        .await
        .or_raise(make_error)?
        .into_iter()
        .flatten()
        .collect();

    Ok(result)
}

pub async fn page_tags_select(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<String>> {
    const MAX_FILTER_VALUES: usize = 100;

    #[derive(Deserialize, Debug)]
    struct Input<'a> {
        site: Reference<'a>,
        categories: Option<Vec<String>>,
        pages: Option<Vec<String>>,
    }

    let Input {
        site,
        categories,
        pages,
    } = parse!(params, Page);

    if categories
        .as_ref()
        .is_some_and(|values| values.len() > MAX_FILTER_VALUES)
        || pages
            .as_ref()
            .is_some_and(|values| values.len() > MAX_FILTER_VALUES)
    {
        return Err(Error::new(
            format!("page tag filters may contain at most {MAX_FILTER_VALUES} values"),
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
    info!("Selecting page tags in site ID {site_id}");

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
                .or_raise(make_error)?
                .into_iter()
                .collect::<Vec<_>>();

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
        page_query = page_query.filter(page::Column::PageCategoryId.is_in(category_ids));
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

pub async fn page_select(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<String>> {
    const MAX_FILTER_VALUES: usize = 100;

    #[derive(Deserialize, Debug)]
    struct Input<'a> {
        site: Reference<'a>,
        pagetype: Option<String>,
        categories: Option<Vec<String>>,
        tags_any: Option<Vec<String>>,
        tags_all: Option<Vec<String>>,
        tags_none: Option<Vec<String>>,
        parent: Option<String>,
        created_by: Option<String>,
        rating: Option<String>,
        order: Option<String>,
    }

    let Input {
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
    } = parse!(params, Page);

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
    info!("Selecting XML-RPC page list in site ID {site_id}");

    let normalize_optional = |value: Option<String>| {
        value.and_then(|value| {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_owned())
        })
    };
    let parent = normalize_optional(parent);
    let created_by = normalize_optional(created_by);
    let rating = normalize_optional(rating);

    let page_type = parse_page_select_type(pagetype.as_deref())?;
    let order = parse_page_select_order(order.as_deref())?;
    let rating_filter = match rating {
        Some(rating) => Some(parse_page_select_rating(&rating)?),
        None => None,
    };

    let created_by = match created_by {
        None => None,
        Some(created_by) => {
            let user_id = resolve_page_select_created_by(ctx, &created_by).await?;
            match user_id {
                Some(user_id) => Some(user_id),
                None => return Ok(Vec::new()),
            }
        }
    };
    let created_by_ids = created_by.into_iter().collect::<Vec<_>>();

    let categories = categories
        .unwrap_or_default()
        .into_iter()
        .map(Cow::Owned)
        .collect::<Vec<_>>();
    let tags_any = tags_any
        .unwrap_or_default()
        .into_iter()
        .map(Cow::Owned)
        .collect::<Vec<_>>();
    let tags_all = tags_all
        .unwrap_or_default()
        .into_iter()
        .map(Cow::Owned)
        .collect::<Vec<_>>();
    let tags_none = tags_none
        .unwrap_or_default()
        .into_iter()
        .map(Cow::Owned)
        .collect::<Vec<_>>();

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

    let found = PageQueryService::find(
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
                untagged: false,
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

    let pages = found
        .pages
        .into_iter()
        .filter(|page| {
            rating_filter
                .as_ref()
                .is_none_or(|filter| filter.matches(page.score.unwrap_or(0.0)))
        })
        .filter_map(|page| page.slug)
        .collect();

    Ok(pages)
}

async fn resolve_page_select_created_by(
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

pub async fn page_edit(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: EditPage = parse!(params, Page);
    info!("Editing page {:?} in site ID {}", input.page, input.site_id);

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page edit actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| Error::new("failed to check edit permission", ErrorType::Page))?;

    PageService::edit(ctx, input)
        .await
        .or_raise(|| Error::new("failed to edit page", ErrorType::Page))
}

pub async fn page_edit_permission(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<PageEditPermissionOutput> {
    let can_edit = PageService::check_user_permission(
        ctx,
        // TODO: Permission context is no longer used, just left here to not break other functions.
        // Remove this when it's removed from the function signature.
        &CheckPermissionContext {
            user_id: None,
            site_id: -1,
            page_reference: None,
        },
        Action::Edit,
    )
    .await
    .or_raise(|| Error::new("failed to check page edit permission", ErrorType::Page))?;

    Ok(PageEditPermissionOutput { can_edit })
}

pub async fn page_delete(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<DeletePageOutput> {
    let input: DeletePage = parse!(params, Page);
    info!(
        "Deleting page {:?} in site ID {}",
        input.page, input.site_id,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page delete actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page delete permission", ErrorType::Page)
        })?;

    PageService::delete(ctx, input)
        .await
        .or_raise(|| Error::new("failed to delete page", ErrorType::Page))
}

pub async fn page_move(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<MovePageOutput> {
    let input: MovePage = parse!(params, Page);
    info!(
        "Moving page {:?} in site ID {} to {}",
        input.page, input.site_id, input.new_slug,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page move actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page move permission", ErrorType::Page)
        })?;
    ensure_page_create_permission(ctx, input.site_id, &input.new_slug, actor_user_id)
        .await
        .or_raise(|| {
            Error::new(
                "failed to check page move destination permission",
                ErrorType::Page,
            )
        })?;

    PageService::r#move(ctx, input)
        .await
        .or_raise(|| Error::new("failed to move page", ErrorType::Page))
}

pub async fn page_rerender(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: PageId = parse!(params, Page);
    info!(
        "Re-rendering page ID {} in site ID {}",
        input.page_id, input.site_id,
    );
    PageRevisionService::rerender(
        ctx,
        input,
        RerenderDepth::default(),
        RerenderType::Full,
    )
    .await
    .or_raise(|| Error::new("failed to rerender page", ErrorType::Page))
}

pub async fn page_restore(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RestorePageOutput> {
    let input: RestorePage = parse!(params, Page);

    info!(
        "Un-deleting page ID {} in site ID {}",
        input.site_id, input.page_id,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page restore actor", ErrorType::Page)
        })?;
    let original_category_id = ensure_deleted_page_edit_permission(
        ctx,
        input.site_id,
        input.page_id,
        actor_user_id,
    )
    .await
    .or_raise(|| {
        Error::new("failed to check page restore permission", ErrorType::Page)
    })?;
    if let Some(ref slug) = input.slug {
        ensure_page_create_permission(ctx, input.site_id, slug, actor_user_id)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check page restore destination permission",
                    ErrorType::Page,
                )
            })?;
    } else {
        ensure_page_permission(
            ctx,
            input.site_id,
            None,
            Some(Reference::Id(original_category_id)),
            actor_user_id,
            Action::Create,
            "restore",
        )
        .await
        .or_raise(|| {
            Error::new(
                "failed to check page restore destination permission",
                ErrorType::Page,
            )
        })?;
    }

    PageService::restore(ctx, input)
        .await
        .or_raise(|| Error::new("failed to restore (undelete) page", ErrorType::Page))
}

pub async fn page_rollback(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<EditPageOutput>> {
    let input: RollbackPage = parse!(params, Page);

    info!(
        "Rolling back page {:?} in site ID {} to revision number {}",
        input.page, input.site_id, input.revision_number,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new(
                "failed to authenticate page rollback actor",
                ErrorType::Page,
            )
        })?;
    ensure_page_edit_permission(ctx, input.site_id, input.page.clone(), actor_user_id)
        .await
        .or_raise(|| {
            Error::new("failed to check page rollback permission", ErrorType::Page)
        })?;

    PageService::rollback(ctx, input)
        .await
        .or_raise(|| Error::new("failed to rollback page", ErrorType::Page))
}

pub async fn page_set_layout(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: SetPageLayout = parse!(params, Page);

    info!(
        "Setting layout override for page ID {} in site ID {} to layout {} (set by user ID {})",
        input.page_id,
        input.site_id,
        match input.layout {
            Some(layout) => layout.value(),
            None => "none (default)",
        },
        input.user_id,
    );

    let actor_user_id = require_authenticated_mutation_actor(ctx, input.user_id)
        .or_raise(|| {
            Error::new("failed to authenticate page layout actor", ErrorType::Page)
        })?;
    ensure_page_edit_permission(
        ctx,
        input.site_id,
        Reference::Id(input.page_id),
        actor_user_id,
    )
    .await
    .or_raise(|| Error::new("failed to check page layout permission", ErrorType::Page))?;

    PageService::set_layout(ctx, input)
        .await
        .or_raise(|| Error::new("failed to set layout for page", ErrorType::Page))
}

fn require_authenticated_mutation_actor(
    ctx: &ServiceContext<'_>,
    attribution_user_id: i64,
) -> Result<i64> {
    let request_user_id = ctx.request().user_id().or_raise(|| {
        Error::new(
            "page mutation requires an authenticated request context",
            ErrorType::PermissionDenied,
        )
    })?;

    if request_user_id == attribution_user_id {
        Ok(request_user_id)
    } else {
        Err(Error::new(
            "page mutation user does not match authenticated request user",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn ensure_page_create_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    slug: &str,
    user_id: i64,
) -> Result<()> {
    let category_slug = get_category_name(slug);
    let category = CategoryService::get_optional(
        ctx,
        site_id,
        Reference::Slug(Cow::Borrowed(category_slug)),
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to load page category for create permission check",
            ErrorType::Permission,
        )
    })?;

    let resource_category = category.map(|category| Reference::Id(category.category_id));
    ensure_page_permission(
        ctx,
        site_id,
        None,
        resource_category,
        user_id,
        Action::Create,
        "create",
    )
    .await
}

async fn ensure_page_edit_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Reference<'a>,
    user_id: i64,
) -> Result<()> {
    let page = PageService::get(ctx, site_id, page_reference.clone())
        .await
        .or_raise(|| {
            Error::new(
                "failed to load page for edit permission check",
                ErrorType::Permission,
            )
        })?;

    ensure_page_permission(
        ctx,
        site_id,
        Some(Reference::Id(page.page_id)),
        Some(Reference::Id(page.page_category_id)),
        user_id,
        Action::Edit,
        "edit",
    )
    .await
}

async fn ensure_deleted_page_edit_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    user_id: i64,
) -> Result<i64> {
    let page = PageService::get_direct(ctx, page_id, true)
        .await
        .or_raise(|| {
            Error::new(
                "failed to load deleted page for edit permission check",
                ErrorType::Permission,
            )
        })?;

    if page.site_id != site_id {
        return Err(Error::new(
            "deleted page is not in the requested site",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    ensure_page_permission(
        ctx,
        site_id,
        Some(Reference::Id(page.page_id)),
        Some(Reference::Id(page.page_category_id)),
        user_id,
        Action::Edit,
        "edit",
    )
    .await?;

    Ok(page.page_category_id)
}

async fn ensure_page_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Option<Reference<'a>>,
    resource_category: Option<Reference<'a>>,
    user_id: i64,
    action: Action,
    action_name: &str,
) -> Result<()> {
    let can_mutate = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: Some(user_id),
            site_id,
            page_reference,
        },
        Permission {
            resource_type: Resource::Page,
            resource_category,
            action,
        },
    )
    .await
    .or_raise(|| Error::new("failed to check page permission", ErrorType::Permission))?;

    if can_mutate {
        Ok(())
    } else {
        Err(Error::new(
            format!("user does not have permission to {action_name} this page"),
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn ensure_page_view_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
) -> Result<()> {
    let page = PageService::get(ctx, site_id, Reference::Id(page_id))
        .await
        .or_raise(|| {
            Error::new(
                "failed to load page for view permission check",
                ErrorType::Permission,
            )
        })?;

    let can_view = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: ctx.request().user_id,
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
    .or_raise(|| {
        Error::new(
            "failed to check page view permission",
            ErrorType::Permission,
        )
    })?;

    if can_view {
        Ok(())
    } else {
        Err(Error::new(
            "user does not have permission to view this page",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

async fn build_page_output(
    ctx: &ServiceContext<'_>,
    page: PageModel,
    details: PageDetails,
) -> Result<Option<GetPageOutput>> {
    let make_error = || Error::new("failed to build page output", ErrorType::Page);

    // Get page revision
    let revision = PageRevisionService::get_latest(ctx, page.site_id, page.page_id)
        .await
        .or_raise(make_error)?;

    // Get category slug from ID
    let category =
        CategoryService::get(ctx, page.site_id, Reference::from(page.page_category_id))
            .await
            .or_raise(make_error)?;

    // Get text data, if requested
    let (wikitext, compiled_body_html, compiled_body_styles) = join!(
        TextService::get_conditional(ctx, details.wikitext, &revision.wikitext_hash),
        TextService::get_conditional(
            ctx,
            details.compiled_html,
            &revision.compiled_body_html_hash,
        ),
        TextService::get_conditional_option(
            ctx,
            details.compiled_html,
            &revision.compiled_body_styles_hash,
        ),
    );
    let (wikitext, compiled_body_html, compiled_body_styles) =
        raise_multiple!(wikitext, compiled_body_html, compiled_body_styles; make_error);
    let compiled_body_styles = if details.compiled_html {
        Some(
            compiled_body_styles
                .map(|styles| serde_json::from_str(&styles))
                .transpose()
                .or_raise(make_error)?
                .unwrap_or_default(),
        )
    } else {
        None
    };

    // Calculate score and determine layout
    let (rating, layout) = join!(
        ScoreService::score(ctx, page.page_id),
        SettingsService::get_layout(ctx, page.site_id, Some(page.page_id)),
    );
    let (rating, layout) = raise_multiple!(rating, layout; make_error);

    // Build result struct
    Ok(Some(GetPageOutput {
        page_id: page.page_id,
        page_created_at: page.created_at,
        page_updated_at: page.updated_at,
        page_deleted_at: page.deleted_at,
        page_revision_count: revision.revision_number + 1,
        site_id: page.site_id,
        page_category_id: category.category_id,
        page_category_slug: category.slug,
        discussion_thread_id: page.discussion_thread_id,
        revision_id: revision.revision_id,
        revision_type: revision.revision_type,
        revision_created_at: revision.created_at,
        revision_number: revision.revision_number,
        revision_user_id: revision.user_id,
        wikitext,
        compiled_body_html,
        compiled_body_styles,
        compiled_at: revision.compiled_at,
        compiled_generator: revision.compiled_generator,
        revision_comments: revision.comments,
        hidden_fields: revision.hidden,
        title: revision.title,
        alt_title: revision.alt_title,
        slug: revision.slug,
        tags: revision.tags,
        rating,
        layout,
    }))
}

async fn build_page_deleted_output(
    ctx: &ServiceContext<'_>,
    page: PageModel,
) -> Result<Option<GetDeletedPageOutput>> {
    let make_error = || {
        Error::new(
            "failed to build page output for a deleted page",
            ErrorType::Page,
        )
    };

    // Get page revision
    let revision = PageRevisionService::get_latest(ctx, page.site_id, page.page_id)
        .await
        .or_raise(make_error)?;

    // Calculate score and determine layout
    let rating = ScoreService::score(ctx, page.page_id)
        .await
        .or_raise(make_error)?;

    // Build result struct
    Ok(Some(GetDeletedPageOutput {
        page_id: page.page_id,
        page_created_at: page.created_at,
        page_updated_at: page.updated_at,
        page_deleted_at: page.deleted_at.expect("Page should be deleted"),
        page_revision_count: revision.revision_number,
        site_id: page.site_id,
        discussion_thread_id: page.discussion_thread_id,
        hidden_fields: revision.hidden,
        title: revision.title,
        alt_title: revision.alt_title,
        slug: revision.slug,
        tags: revision.tags,
        rating,
    }))
}

async fn build_page_file_output(
    ctx: &ServiceContext<'_>,
    file: FileModel,
) -> Result<Option<GetFileOutput>> {
    let make_error = || {
        Error::new(
            "failed to build output for a file on a page",
            ErrorType::Page,
        )
    };

    // Get file revision
    let revision =
        FileRevisionService::get_latest(ctx, file.site_id, file.page_id, file.file_id)
            .await
            .or_raise(make_error)?;

    // Build result struct
    Ok(Some(GetFileOutput {
        file_id: file.file_id,
        file_created_at: file.created_at,
        file_updated_at: file.updated_at,
        file_deleted_at: file.deleted_at,
        page_id: file.page_id,
        revision_id: revision.revision_id,
        revision_type: revision.revision_type,
        revision_created_at: revision.created_at,
        revision_number: revision.revision_number,
        revision_user_id: revision.user_id,
        name: file.name,
        data: None,
        mime: revision.mime,
        size: revision.size,
        s3_hash: Bytes::from(revision.s3_hash),
        revision_comments: revision.comments,
        hidden_fields: revision.hidden,
    }))
}
