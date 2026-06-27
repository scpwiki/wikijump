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

use super::prelude::*;
use crate::models::page::{self, Entity as Page};
use crate::services::page_query::{
    CategoriesSelector, DateSelector, FoundPageFields, IncludedCategories,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageTypeSelector,
    PaginationSelector, RangeSelector, TagCondition,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::score::ScoreValue;
use crate::services::{PageQueryService, PageRevisionService, ScoreService};
use crate::types::{Action, PageId, Permission, Resource};
use regex::Regex;
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
const RATING_VARIABLE: &str = "rating";
const FRAGMENT_CATEGORY: &str = "fragment";
const MAX_LIST_PAGES_MODULES: usize = 256;
const MAX_LIST_PAGES_ATTRIBUTES_BYTES: usize = 8 * 1024;
const MAX_LIST_PAGES_BODY_BYTES: usize = 64 * 1024;
const MODULE_OPEN: &str = "[[module";
const MODULE_CLOSE: &str = "[[/module]]";

static MODULE_ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?P<name>[A-Za-z_][A-Za-z0-9_-]*)[ \t]*=[ \t]*\"(?P<value>[^\"]*)\""#)
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
    FragmentChildContent {
        body_template: String,
        offset: u32,
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
            Some(SupportedListPages::FragmentChildContent {
                body_template,
                offset,
            }) => {
                let selected = select_fragment(ctx, page_id, offset).await?;
                let content = selected.as_deref().unwrap_or("");
                expanded.push_str(&body_template.replace(CONTENT_VARIABLE, content));
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

    if attributes.len() == 5
        && attributes.get("parent") == Some(&".")
        && attributes.get("category") == Some(&FRAGMENT_CATEGORY)
        && attributes.get("order") == Some(&"created_at")
        && attributes.get("limit") == Some(&"1")
        && attributes.get("offset") == Some(&"@URL|0")
        && body.trim() == CONTENT_VARIABLE
    {
        // A page render has no request URL query in this layer, so @URL|0 uses
        // its declared fallback. Parsing a supplied URL offset is intentionally
        // unsupported.
        return Ok(Some(SupportedListPages::FragmentChildContent {
            body_template: body.to_owned(),
            offset: 0,
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

async fn select_fragment(
    ctx: &ServiceContext<'_>,
    page_id: &PageId,
    offset: u32,
) -> Result<Option<String>> {
    let included_categories = [Cow::Borrowed(FRAGMENT_CATEGORY)];
    let unbounded_date = DateSelector::FromPresent {
        start: OffsetDateTime::UNIX_EPOCH,
    };

    let found = PageQueryService::find(
        ctx,
        PageQuery {
            current_page_id: page_id.page_id,
            current_site_id: page_id.site_id,
            queried_site_id: None,
            page_type: PageTypeSelector::All,
            categories: CategoriesSelector {
                included_categories: IncludedCategories::List(&included_categories),
                excluded_categories: &[],
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &[],
                none_present: &[],
            },
            page_parent: PageParentSelector::ChildOf,
            contains_outgoing_links: &[],
            creation_date: unbounded_date,
            update_date: unbounded_date,
            author: &[],
            score: &[],
            votes: &[],
            offset,
            range: RangeSelector::Others,
            name: None,
            slug: None,
            data_form_fields: &[],
            order: Some(OrderBySelector {
                property: OrderProperty::CreatedAt,
                ascending: true,
            }),
            pagination: PaginationSelector {
                limit: Some(1),
                per_page: 1,
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

    let Some(selected) = found.pages.into_iter().next() else {
        debug!(
            "ListPages found no fragment child for page ID {}",
            page_id.page_id,
        );
        return Ok(None);
    };

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
        return Ok(None);
    }

    debug!(
        "ListPages selected child page ID {} for page ID {}",
        selected.page_id, page_id.page_id,
    );

    let wikitext = PageRevisionService::get_wikitext(
        ctx,
        selected.site_id,
        Reference::Id(selected.page_id),
    )
    .await?;

    Ok(Some(wikitext))
}

fn render_error(message: impl Into<String>) -> Error {
    Error::new(message.into(), ErrorType::Render)
}
