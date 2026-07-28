/*
 * services/render/list_pages.rs
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

mod ajax;
pub(super) mod authors;
pub(super) mod content_sections;
mod current_page;
mod data_forms;
mod feed;
mod generated_html;
mod pagination;
mod parents;
mod presentation;
mod rendering;
pub(super) mod scanner;
pub(super) mod substitution;
pub(super) mod template;

#[cfg(test)]
pub(super) use self::ajax::AJAX_MODULE_LITERAL_MARKER_PREFIX;
pub(super) use self::ajax::{
    build_wikidot_list_pages_module_source, protect_ajax_module_literal_markers,
};
pub(super) use self::current_page::{
    count_pages_scan_requires_preservation, count_pages_unbounded_total,
    list_pages_content_query_target, list_pages_row_scan_target,
    page_query_cap_requires_original_module, should_render_current_page_list_pages_row,
};
#[cfg(test)]
pub(super) use self::current_page::{
    current_page_info_list_pages_row, requested_page_info_score,
};
pub(super) use self::data_forms::load_list_pages_data_form_definitions;
pub(super) use self::generated_html::{
    preserve_list_pages_following_paragraph_boundary, register_generated_list_pages_html,
    url_offset_list_pages_content_bytes,
};
pub(super) use self::pagination::{list_pages_feed_info_html, push_list_pages_pager};
#[cfg(test)]
pub(super) use self::parents::ListPagesParentDisplay;
#[cfg(test)]
pub(super) use self::presentation::{
    format_list_pages_created_at, list_pages_tag_link_href, render_list_pages_tags,
    substitute_list_pages_variables,
};
pub(super) use self::presentation::{
    is_list_pages_visible_tag, is_tag_cloud_visible_tag, list_pages_created_by_unix,
    list_pages_parent_fullname, list_pages_revision_count,
    render_list_pages_wikidot_user, render_tag_cloud_box,
    restore_list_pages_literal_ellipsis_markers, substitute_count_pages_variables,
};
pub(super) use self::rendering::{
    CountPagesExpansionOptions, ListPagesBlockRenderResult, ListPagesContentCache,
    ListPagesExpansion, ListPagesExpansionBudget, ListPagesExpansionOptions,
    ListPagesPageContext,
};
pub(super) use self::substitution::{
    CurrentPageAuthorSource, ExactNameListPagesBatchKey, ListPagesArguments,
    ListPagesAuthorCacheKey, ListPagesBatchDisplayRequirements, ListPagesBatchDisplays,
    ListPagesRuntimeDisplay, ListPagesSnapshotDisplay, ListPagesSubstitutionContext,
    ResolvedListPagesAuthors, WikidotUserDisplay, count_pages_capture_is_literal,
    count_pages_exact_count_render_diagnostics, count_pages_required_tag_batch_result,
    count_pages_required_tag_batch_selector, count_pages_should_remain_literal,
    exact_name_list_pages_batch_key, list_pages_argument_error,
    list_pages_author_cache_key, list_pages_has_unsupported_page_type_selector,
    list_pages_has_unsupported_parent_selector, list_pages_static_parent_fullname,
    parse_list_pages_arguments, parse_list_pages_arguments_with_url,
    substitute_list_pages_rating_only, substitute_list_pages_variables_with_fragments,
    union_found_page_fields, unsupported_list_pages_replacement,
};
#[cfg(test)]
pub(super) use self::substitution::{
    ListPagesOffsetOrigin, list_pages_body_is_no_visible_tracking_markup,
    list_pages_body_uses_content_variable, list_pages_body_variables_supported,
    parse_list_pages_date_selector,
};

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::{self, Entity as Page};
use crate::services::ServiceContext;
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, DateSelector, FoundPageFields,
    IncludedCategories, OrderBySelector, OrderProperty, PageParentSelector, PageQuery,
    PageTypeSelector, PaginationSelector, RangeSelector, TagCondition,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::score::ScoreValue;
use crate::services::{PageQueryService, PageRevisionService, ScoreService};
use crate::types::Reference;
use crate::types::{Action, PageId, Permission, Resource};
use regex::Regex;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use time::{Month, OffsetDateTime};
use wikidot_normalize::normalize;

// This intentionally recognizes only bounded, proven ListPages shapes. Generic
// ListPages parsing remains follow-up work; unsupported complete modules are
// preserved for now so existing pages keep their current behavior.
const CONTENT_VARIABLE: &str = "%%content%%";
const CREATED_AT_VARIABLE: &str = "created_at";
const DEFAULT_CATEGORY: &str = "*";
const DEFAULT_LIMIT: u64 = 20;
const RATING_VARIABLE: &str = "rating";
const FRAGMENT_CATEGORY: &str = "fragment";
const MAX_CHILD_CONTENT_EXPANSIONS: usize = 512;
const MAX_SUPPORTED_LIMIT: u64 = 50;
const MAX_SUPPORTED_OFFSET: u32 = 1_000;
const MAX_LIST_PAGES_MODULES: usize = 256;
const MAX_LIST_PAGES_ATTRIBUTES_BYTES: usize = 8 * 1024;
const MAX_LIST_PAGES_BODY_BYTES: usize = 64 * 1024;
const MODULE_OPEN: &str = "[[module";
const MODULE_CLOSE: &str = "[[/module]]";

static MODULE_ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?P<name>[A-Za-z_][A-Za-z0-9_-]*)[ \t]*=[ \t]*"(?P<value>[^"]*)""#)
        .expect("module attribute regular expression should compile")
});

#[derive(Debug)]
struct ListPagesOccurrence {
    start: usize,
    end: usize,
    specification: Option<SupportedListPages>,
}

#[derive(Debug)]
enum SupportedListPages {
    GeneralChildContent {
        body_template: String,
        category: String,
        offset: u32,
        limit: u64,
        order: OrderBySelector,
        page_type: PageTypeSelector,
    },
    NamedPageMetadata {
        body_template: String,
        requested_name: String,
        normalized_slug: String,
    },
}

#[derive(Debug)]
struct NamedPageMetadata {
    page_id: i64,
    created_at: OffsetDateTime,
    score: ScoreValue,
}

pub(super) async fn expand_list_pages(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    page_id: &PageId,
) -> Result<String> {
    if !wikitext.contains(MODULE_OPEN) {
        return Ok(wikitext);
    }

    let occurrences = find_occurrences(&wikitext)?;
    if occurrences.is_empty() {
        if wikitext.contains("ListPages") {
            warn!(
                "Page ID {} contains ListPages text, but no complete block was recognized",
                page_id.page_id,
            );
        }
        return Ok(wikitext);
    }

    let named_slugs = collect_named_slugs(&occurrences);
    let named_pages = resolve_named_pages(ctx, page_id.site_id, &named_slugs).await?;
    let mut remaining_child_content = MAX_CHILD_CONTENT_EXPANSIONS;

    let mut expanded = String::with_capacity(wikitext.len());
    let mut cursor = 0;

    for ListPagesOccurrence {
        start,
        end,
        specification,
    } in occurrences
    {
        expanded.push_str(&wikitext[cursor..start]);

        match specification {
            Some(SupportedListPages::GeneralChildContent {
                body_template,
                category,
                offset,
                limit,
                order,
                page_type,
            }) => {
                if remaining_child_content == 0 {
                    warn!(
                        "Skipping ListPages child-content block on page ID {} because the per-page expansion budget is exhausted",
                        page_id.page_id,
                    );
                    cursor = end;
                    continue;
                }
                let limit = limit.min(remaining_child_content as u64);
                for content in select_child_content_pages(
                    ctx, page_id, &category, offset, limit, order, page_type,
                )
                .await?
                {
                    remaining_child_content -= 1;
                    expanded.push_str(&body_template.replace(CONTENT_VARIABLE, &content));
                }
            }
            Some(SupportedListPages::NamedPageMetadata {
                body_template,
                requested_name,
                normalized_slug,
            }) => match named_pages.get(&normalized_slug) {
                Some(metadata) => {
                    debug!(
                        "ListPages exact-name '{}' selected page ID {} for page ID {}",
                        requested_name, metadata.page_id, page_id.page_id,
                    );
                    expanded.push_str(&render_named_body(&body_template, metadata)?);
                }
                None => {
                    debug!(
                        "ListPages exact-name '{}' found no visible page in site ID {} for page ID {}",
                        requested_name, page_id.site_id, page_id.page_id,
                    );
                }
            },
            None => {
                warn!(
                    "Leaving unsupported ListPages block unchanged on page ID {}",
                    page_id.page_id,
                );
                expanded.push_str(&wikitext[start..end]);
            }
        }

        cursor = end;
    }

    expanded.push_str(&wikitext[cursor..]);
    Ok(expanded)
}

fn collect_named_slugs(occurrences: &[ListPagesOccurrence]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut slugs = Vec::new();

    for occurrence in occurrences {
        if let Some(SupportedListPages::NamedPageMetadata {
            normalized_slug, ..
        }) = &occurrence.specification
            && seen.insert(normalized_slug.clone())
        {
            slugs.push(normalized_slug.clone());
        }
    }

    slugs
}

fn find_occurrences(wikitext: &str) -> Result<Vec<ListPagesOccurrence>> {
    let mut occurrences = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = wikitext[cursor..].find(MODULE_OPEN) {
        let start = cursor + relative_start;
        let Some(opening_end) = wikitext[start..].find("]]").map(|offset| start + offset)
        else {
            return Err(render_error("Unclosed module opening in ListPages scan").into());
        };

        let opening_source = &wikitext[start + MODULE_OPEN.len()..opening_end];
        let (module_name, attributes) = split_module_opening(opening_source);
        if module_name != "ListPages" {
            cursor = opening_end + 2;
            continue;
        }

        if occurrences.len() >= MAX_LIST_PAGES_MODULES {
            return Err(render_error(format!(
                "ListPages module count exceeds limit of {}",
                MAX_LIST_PAGES_MODULES,
            ))
            .into());
        }
        if attributes.len() > MAX_LIST_PAGES_ATTRIBUTES_BYTES {
            return Err(render_error(format!(
                "ListPages attributes exceed {} bytes",
                MAX_LIST_PAGES_ATTRIBUTES_BYTES,
            ))
            .into());
        }

        let body_start = opening_end + 2;
        let (closing_start, end) = find_matching_module_close(wikitext, body_start)?;
        let body = &wikitext[body_start..closing_start];
        if body.len() > MAX_LIST_PAGES_BODY_BYTES {
            return Err(render_error(format!(
                "ListPages body exceeds {} bytes",
                MAX_LIST_PAGES_BODY_BYTES,
            ))
            .into());
        }

        occurrences.push(ListPagesOccurrence {
            start,
            end,
            specification: parse_supported_specification(attributes, body)?,
        });
        cursor = end;
    }

    Ok(occurrences)
}

fn split_module_opening(source: &str) -> (&str, &str) {
    let trimmed = source.trim();
    let Some(split_at) = trimmed.find(char::is_whitespace) else {
        return (trimmed, "");
    };

    let module_name = &trimmed[..split_at];
    let attributes = trimmed[split_at..].trim();
    (module_name, attributes)
}

fn find_matching_module_close(
    wikitext: &str,
    mut cursor: usize,
) -> Result<(usize, usize)> {
    let mut depth = 1usize;

    loop {
        let next_open = wikitext[cursor..]
            .find(MODULE_OPEN)
            .map(|offset| cursor + offset);
        let next_close = wikitext[cursor..]
            .find(MODULE_CLOSE)
            .map(|offset| cursor + offset);

        match (next_open, next_close) {
            (_, None) => return Err(render_error("Unclosed ListPages module").into()),
            (Some(open), Some(close)) if open < close => {
                let Some(opening_end) =
                    wikitext[open..].find("]]").map(|offset| open + offset)
                else {
                    return Err(render_error(
                        "Unclosed nested module opening in ListPages body",
                    )
                    .into());
                };
                depth += 1;
                cursor = opening_end + 2;
            }
            (_, Some(close)) => {
                depth -= 1;
                let end = close + MODULE_CLOSE.len();
                if depth == 0 {
                    return Ok((close, end));
                }
                cursor = end;
            }
        }
    }
}

fn parse_supported_specification(
    attribute_source: &str,
    body: &str,
) -> Result<Option<SupportedListPages>> {
    let Some(attributes) = parse_attributes(attribute_source) else {
        return Ok(None);
    };

    if attributes.keys().all(|name| {
        matches!(
            *name,
            "category" | "limit" | "offset" | "order" | "pagetype" | "parent"
        )
    }) && attributes.get("parent").copied() == Some(".")
        && body.contains(CONTENT_VARIABLE)
        && child_content_body_variables_supported(body)?
    {
        let Some(offset) = parse_offset(attributes.get("offset").copied()) else {
            return Ok(None);
        };
        let Some(limit) = parse_limit(attributes.get("limit").copied()) else {
            return Ok(None);
        };
        let Some(order) =
            parse_order(attributes.get("order").copied().unwrap_or("created_at"))
        else {
            return Ok(None);
        };
        let Some(page_type) =
            parse_page_type(attributes.get("pagetype").copied().unwrap_or("normal"))
        else {
            return Ok(None);
        };

        return Ok(Some(SupportedListPages::GeneralChildContent {
            body_template: body.to_owned(),
            category: attributes
                .get("category")
                .copied()
                .unwrap_or(DEFAULT_CATEGORY)
                .to_owned(),
            offset,
            limit,
            order,
            page_type,
        }));
    }

    if attributes.len() == 1
        && let Some(requested_name) = attributes.get("name")
    {
        let mut normalized_slug = (*requested_name).to_owned();
        normalize(&mut normalized_slug);
        if normalized_slug.is_empty() {
            return Ok(None);
        }
        match ensure_named_body_variables_are_supported(body) {
            Ok(()) => {}
            Err(error)
                if error
                    .to_string()
                    .contains("Unsupported ListPages exact-name variable") =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        }
        return Ok(Some(SupportedListPages::NamedPageMetadata {
            body_template: body.to_owned(),
            requested_name: (*requested_name).to_owned(),
            normalized_slug,
        }));
    }

    Ok(None)
}

fn parse_limit(value: Option<&str>) -> Option<u64> {
    let limit = match value {
        Some(value) => value.parse().ok()?,
        None => DEFAULT_LIMIT,
    };
    (1..=MAX_SUPPORTED_LIMIT).contains(&limit).then_some(limit)
}

fn parse_offset(value: Option<&str>) -> Option<u32> {
    let value = value.unwrap_or("0");
    let fallback = value.strip_prefix("@URL|").unwrap_or(value);
    let offset = fallback.parse().ok()?;
    (offset <= MAX_SUPPORTED_OFFSET).then_some(offset)
}

fn parse_order(value: &str) -> Option<OrderBySelector> {
    let mut parts = value.split_whitespace();
    let property = match parts.next()? {
        "created_at" => OrderProperty::CreatedAt,
        "updated_at" => OrderProperty::UpdatedAt,
        "name" | "fullname" | "slug" => OrderProperty::FullSlug,
        "random" => OrderProperty::Random,
        _ => return None,
    };
    let ascending = match parts.next() {
        None | Some("asc") => true,
        Some("desc") => false,
        Some(_) => return None,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(OrderBySelector {
        property,
        ascending,
    })
}

fn parse_page_type(value: &str) -> Option<PageTypeSelector> {
    match value {
        "all" => Some(PageTypeSelector::All),
        "hidden" => Some(PageTypeSelector::Hidden),
        "normal" => Some(PageTypeSelector::Normal),
        _ => None,
    }
}

fn parse_attributes(source: &str) -> Option<HashMap<&str, &str>> {
    let mut attributes = HashMap::new();
    let mut cursor = 0;

    for captures in MODULE_ATTRIBUTE.captures_iter(source) {
        let full_match = captures
            .get(0)
            .expect("attribute capture should contain the full match");

        if !source[cursor..full_match.start()].trim().is_empty() {
            return None;
        }

        let name = captures
            .name("name")
            .expect("attribute capture should contain a name")
            .as_str();
        let value = captures
            .name("value")
            .expect("attribute capture should contain a value")
            .as_str();

        if attributes.insert(name, value).is_some() {
            return None;
        }

        cursor = full_match.end();
    }

    if !source[cursor..].trim().is_empty() {
        return None;
    }

    Some(attributes)
}

fn ensure_named_body_variables_are_supported(body: &str) -> Result<()> {
    let mut cursor = 0;
    while let Some(start_relative) = body[cursor..].find("%%") {
        let start = cursor + start_relative;
        let variable_start = start + 2;
        let Some(end_relative) = body[variable_start..].find("%%") else {
            return Err(
                render_error("Unclosed ListPages variable in exact-name body").into(),
            );
        };
        let end = variable_start + end_relative;
        let variable = &body[variable_start..end];
        if variable != CREATED_AT_VARIABLE && variable != RATING_VARIABLE {
            return Err(render_error(format!(
                "Unsupported ListPages exact-name variable: {}",
                variable,
            ))
            .into());
        }
        cursor = end + 2;
    }

    Ok(())
}

fn child_content_body_variables_supported(body: &str) -> Result<bool> {
    let mut cursor = 0;
    while let Some(start_relative) = body[cursor..].find("%%") {
        let start = cursor + start_relative;
        let variable_start = start + 2;
        let Some(end_relative) = body[variable_start..].find("%%") else {
            return Ok(false);
        };
        let end = variable_start + end_relative;
        if &body[variable_start..end] != "content" {
            return Ok(false);
        }
        cursor = end + 2;
    }

    Ok(true)
}

async fn resolve_named_pages(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    normalized_slugs: &[String],
) -> Result<HashMap<String, NamedPageMetadata>> {
    if normalized_slugs.is_empty() {
        return Ok(HashMap::new());
    }

    let txn = ctx.transaction();
    let make_error = || {
        Error::new(
            "failed to resolve ListPages exact-name pages",
            ErrorType::Render,
        )
    };
    let pages = Page::find()
        .filter(page::Column::SiteId.eq(site_id))
        .filter(page::Column::DeletedAt.is_null())
        .filter(page::Column::Slug.is_in(normalized_slugs.iter().cloned()))
        .all(txn)
        .await
        .or_raise(make_error)?;

    let mut resolved = HashMap::new();
    for page in pages {
        let anonymously_viewable = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: None,
                site_id: page.site_id,
                page_reference: Some(Reference::Id(page.page_id)),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(page.page_category_id)),
                action: Action::View,
            },
        )
        .await?;

        if !anonymously_viewable {
            debug!(
                "Skipping ListPages exact-name page ID {} because it is not safe to cache for anonymous viewers",
                page.page_id,
            );
            continue;
        }

        let score = ScoreService::score(ctx, page.page_id).await?;
        resolved.insert(
            page.slug.clone(),
            NamedPageMetadata {
                page_id: page.page_id,
                created_at: page.created_at,
                score,
            },
        );
    }

    Ok(resolved)
}

fn render_named_body(body: &str, page: &NamedPageMetadata) -> Result<String> {
    ensure_named_body_variables_are_supported(body)?;
    Ok(body
        .replace(
            "%%created_at%%",
            &format_created_at_wikidot(page.created_at),
        )
        .replace("%%rating%%", &format_score(&page.score)))
}

fn format_created_at_wikidot(value: OffsetDateTime) -> String {
    let month = match value.month() {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    };

    format!(
        "{:02} {} {:04} {:02}:{:02}",
        value.day(),
        month,
        value.year(),
        value.hour(),
        value.minute(),
    )
}

fn format_score(score: &ScoreValue) -> String {
    match score {
        ScoreValue::Integer(value) => value.to_string(),
        ScoreValue::Float(value) => value.to_string(),
    }
}

async fn select_child_content_pages(
    ctx: &ServiceContext<'_>,
    page_id: &PageId,
    category: &str,
    offset: u32,
    limit: u64,
    order: OrderBySelector,
    page_type: PageTypeSelector,
) -> Result<Vec<String>> {
    let included_categories = [Cow::Borrowed(category)];
    let included_categories = if category == DEFAULT_CATEGORY {
        IncludedCategories::All
    } else {
        IncludedCategories::List(&included_categories)
    };
    let unbounded_date = DateSelector::FromPresent {
        start: OffsetDateTime::UNIX_EPOCH,
    };

    let found = PageQueryService::find(
        ctx,
        PageQuery {
            current_page_id: page_id.page_id,
            current_site_id: page_id.site_id,
            queried_site_id: None,
            page_type,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &[],
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &[],
                none_present: &[],
                untagged: false,
            },
            page_parent: PageParentSelector::ChildOf,
            contains_outgoing_links: &[],
            creation_date: unbounded_date,
            update_date: unbounded_date,
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset,
            range: RangeSelector::Others,
            name: None,
            slug: None,
            slugs: &[],
            data_form_fields: &[],
            order: Some(order),
            candidate_limit: None,
            pagination: PaginationSelector {
                limit: Some(limit),
                per_page: limit
                    .try_into()
                    .expect("supported ListPages limit should fit in u8"),
                reversed: false,
            },
            variables: &[],
            fields: FoundPageFields {
                page_category_id: true,
                ..FoundPageFields::default()
            },
        },
    )
    .await?;

    if found.pages.is_empty() {
        debug!(
            "ListPages found no child page for page ID {}",
            page_id.page_id,
        );
        return Ok(Vec::new());
    }

    let mut selected_wikitext = Vec::with_capacity(found.pages.len());
    for selected in found.pages {
        if selected.site_id != page_id.site_id {
            error!(
                "ListPages selected child page ID {} from site ID {}, but parent page ID {} is in site ID {}",
                selected.page_id, selected.site_id, page_id.page_id, page_id.site_id,
            );
            return Err(Error::new(
                "ListPages selected a child page from the wrong site",
                ErrorType::Render,
            )
            .into());
        }

        let page_category_id = selected
            .page_category_id
            .expect("ListPages query requested selected page category IDs");
        let anonymously_viewable = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: None,
                site_id: selected.site_id,
                page_reference: Some(Reference::Id(selected.page_id)),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(page_category_id)),
                action: Action::View,
            },
        )
        .await?;

        if !anonymously_viewable {
            warn!(
                "Skipping ListPages child page ID {} for page ID {} because it is not safe to cache for anonymous viewers",
                selected.page_id, page_id.page_id,
            );
            continue;
        }

        debug!(
            "ListPages selected child page ID {} for page ID {}",
            selected.page_id, page_id.page_id,
        );

        selected_wikitext.push(
            PageRevisionService::get_wikitext(
                ctx,
                selected.site_id,
                Reference::Id(selected.page_id),
            )
            .await?,
        );
    }

    Ok(selected_wikitext)
}

fn render_error(message: impl Into<String>) -> Error {
    Error::new(message.into(), ErrorType::Render)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_existing_scp8980_shape() {
        let specification = parse_supported_specification(
            r#" parent="." category="fragment" order="created_at" limit="1" offset="@URL|0""#,
            CONTENT_VARIABLE,
        )
        .expect("existing SCP-8980 ListPages shape should not error")
        .expect("existing SCP-8980 ListPages shape should remain supported");

        let SupportedListPages::GeneralChildContent {
            body_template,
            category,
            offset,
            limit,
            order,
            page_type,
        } = specification
        else {
            panic!("SCP-8980 ListPages shape should parse as child content");
        };

        assert_eq!(category, "fragment");
        assert_eq!(offset, 0);
        assert_eq!(limit, 1);
        assert_eq!(order.property, OrderProperty::CreatedAt);
        assert!(order.ascending);
        assert_eq!(page_type, PageTypeSelector::Normal);
        assert_eq!(body_template, CONTENT_VARIABLE);
    }

    #[test]
    fn parses_wrapped_content_with_safe_query_attributes() {
        let specification = parse_supported_specification(
            r#" parent="." category="fragment" order="updated_at desc" limit="2" offset="@URL|1" pagetype="all""#,
            "before %%content%% after",
        )
        .expect("wrapped content with safe attributes should not error")
        .expect("wrapped content with safe attributes should be supported");

        let SupportedListPages::GeneralChildContent {
            body_template,
            category,
            offset,
            limit,
            order,
            page_type,
        } = specification
        else {
            panic!("wrapped content should parse as child content");
        };

        assert_eq!(category, "fragment");
        assert_eq!(offset, 1);
        assert_eq!(limit, 2);
        assert_eq!(order.property, OrderProperty::UpdatedAt);
        assert!(!order.ascending);
        assert_eq!(page_type, PageTypeSelector::All);
        assert_eq!(body_template, "before %%content%% after");
    }

    #[test]
    fn rejects_unknown_or_unsafe_content_attributes() {
        for (attributes, body) in [
            (r#" category="fragment""#, CONTENT_VARIABLE),
            (r#" parent="other" category="fragment""#, CONTENT_VARIABLE),
            (
                r#" parent="." category="fragment" unknown="x""#,
                CONTENT_VARIABLE,
            ),
            (
                r#" parent="." category="fragment" limit="51""#,
                CONTENT_VARIABLE,
            ),
            (
                r#" parent="." category="fragment" order="title""#,
                CONTENT_VARIABLE,
            ),
            (
                r#" parent="." category="fragment" offset="1001""#,
                CONTENT_VARIABLE,
            ),
            (r#" parent="." category="fragment""#, "no content variable"),
            (
                r#" parent="." category="fragment""#,
                "%%title%% :: %%content%%",
            ),
            (
                r#" parent="." category="fragment""#,
                "before %%content after",
            ),
        ] {
            assert!(
                parse_supported_specification(attributes, body)
                    .expect("unsupported attributes should not be fatal")
                    .is_none()
            );
        }
    }

    #[test]
    fn keeps_named_metadata_shape_available() {
        let specification = parse_supported_specification(
            r#" name="scp-173""#,
            "Created %%created_at%% with rating %%rating%%",
        )
        .expect("named metadata shape should not error")
        .expect("named metadata shape should be supported");

        assert!(matches!(
            specification,
            SupportedListPages::NamedPageMetadata { .. }
        ));
    }

    #[test]
    fn recognizes_live_list_pages_argument_errors() {
        for (head, has_current_page, expected) in [
            (r#"range="others""#, false, Some("Invalid range argument.")),
            (r#"range="others""#, true, None),
            (r#"range="bogus""#, true, Some("Invalid range argument.")),
            (
                r#"pagetype="bogus""#,
                false,
                Some("Invalid pagetype attribute."),
            ),
            (r#"rating="bad""#, false, Some("Invalid rating argument.")),
            (r#"votes="bad""#, false, Some("Invalid votes argument.")),
        ] {
            assert_eq!(
                list_pages_argument_error(head, has_current_page),
                expected,
                "{head:?}",
            );
        }
        assert!(parse_list_pages_arguments(r#"rating="<>5""#).is_some());
        assert_eq!(
            list_pages_static_parent_fullname(r#"parent="system:start""#),
            Some("system:start"),
        );
        assert_eq!(list_pages_static_parent_fullname(r#"parent=".""#), None,);
        let arguments = parse_list_pages_arguments(r#"parent="system:start""#)
            .expect("a named parent should become a query selector");
        assert_eq!(
            arguments.static_parent_fullname.as_deref(),
            Some("system:start"),
        );
        assert!(!list_pages_has_unsupported_parent_selector(
            r#"parent="system:start""#,
        ));

        let arguments =
            parse_list_pages_arguments(r#"limit="" perPage="" separate="" wrapper="""#)
                .expect("empty live defaults should parse");
        assert_eq!(arguments.limit, None);
        assert_eq!(arguments.count_pages_per_page, None);
        assert!(arguments.separate);
        assert!(arguments.wrapper);

        let arguments =
            parse_list_pages_arguments(r#"limit="999999999" perPage="999999999""#)
                .expect("large live limits should parse");
        assert_eq!(arguments.limit, Some(999_999_999));
        assert_eq!(arguments.count_pages_per_page, Some(250));

        let arguments = parse_list_pages_arguments(r#"tags="=" parent="-=""#)
            .expect("documented current-tag and parent selectors should parse");
        assert!(arguments.same_visible_tags);
        assert_eq!(arguments.page_parent, PageParentSelector::DifferentParents,);
        assert!(!arguments.unsupported_list_pages_filter);
        assert!(!list_pages_has_unsupported_parent_selector(
            r#"parent="-=""#,
        ));
        let arguments = parse_list_pages_arguments(r#"tag="==""#)
            .expect("documented exact current-tag selector should parse");
        assert!(arguments.exact_visible_tags);

        let arguments = parse_list_pages_arguments(
            r#"created_at="=" updated_at="=" rating="=" votes="=""#,
        )
        .expect("current-page date and rating selectors should parse");
        assert!(arguments.creation_date_current_page);
        assert!(arguments.update_date_current_page);
        assert!(arguments.score_equals_current_page);
        assert!(arguments.votes_equals_current_page);
        assert!(!arguments.unsupported_list_pages_filter);

        for head in [
            r#"created_at="last hour""#,
            r#"created_at="older than month""#,
            r#"created_at="older than 2""#,
            r#"updated_at="newer than week""#,
        ] {
            assert!(
                parse_list_pages_arguments(head).is_some(),
                "{head:?} should parse",
            );
        }
        assert!(
            parse_list_pages_arguments(r#"created_at="not-a-date""#).is_some(),
            "live ignores an invalid date selector",
        );

        let arguments = parse_list_pages_arguments(
            r#"category="fragment" parent="." order="name"" limit="1" offset="@URL|0""#,
        )
        .expect("live-tokenized interior quote in order value should parse");
        assert_eq!(arguments.categories, vec![Cow::Borrowed("fragment")]);
        assert_eq!(arguments.page_parent, PageParentSelector::ChildOf);
        assert_eq!(arguments.order, Some(OrderBySelector::default()));
        assert_eq!(arguments.limit, Some(1));

        let arguments =
            parse_list_pages_arguments(r#"tags="+scp rating="<0" separate="no""#)
                .expect("live-tokenized interior quote in tags value should parse");
        assert_eq!(arguments.all_tags, vec![Cow::Borrowed("scp")]);
        assert_eq!(arguments.default_tags, vec![Cow::Borrowed(r#"rating="<0"#)]);
        assert!(!arguments.separate);
        assert!(!arguments.unsupported_list_pages_filter);
    }
}
