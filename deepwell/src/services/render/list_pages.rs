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
use crate::services::page_query::{
    CategoriesSelector, DateSelector, FoundPageFields, IncludedCategories,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageTypeSelector,
    PaginationSelector, RangeSelector, TagCondition,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{PageQueryService, PageRevisionService};
use crate::types::{Action, PageId, Permission, Resource};
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
use time::OffsetDateTime;

// This intentionally recognizes only the SCP-8980 proof shape. Generic ListPages
// parsing and URL-derived offsets remain separate follow-up work.
const CONTENT_VARIABLE: &str = "%%content%%";
const FRAGMENT_CATEGORY: &str = "fragment";

static LIST_PAGES_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)\[\[module[ \t]+ListPages(?P<attributes>[^\]]*)\]\](?P<body>.*?)\[\[/module\]\]"#,
    )
    .expect("ListPages block regular expression should compile")
});

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
struct SupportedListPages {
    body_template: String,
    offset: u32,
}

pub(super) async fn expand_list_pages(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    page_id: &PageId,
) -> Result<String> {
    if !wikitext.contains("[[module ListPages") {
        return Ok(wikitext);
    }

    let occurrences = find_occurrences(&wikitext);
    if occurrences.is_empty() {
        warn!(
            "Page ID {} contains ListPages text, but no complete block was recognized",
            page_id.page_id,
        );
        return Ok(wikitext);
    }

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
            Some(specification) => {
                let selected =
                    select_fragment(ctx, page_id, specification.offset).await?;
                let content = selected.as_deref().unwrap_or("");
                expanded.push_str(
                    &specification
                        .body_template
                        .replace(CONTENT_VARIABLE, content),
                );
            }
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

fn find_occurrences(wikitext: &str) -> Vec<ListPagesOccurrence> {
    LIST_PAGES_BLOCK
        .captures_iter(wikitext)
        .map(|captures| {
            let full_match = captures
                .get(0)
                .expect("ListPages capture should contain the full match");
            let attributes = captures
                .name("attributes")
                .expect("ListPages capture should contain attributes")
                .as_str();
            let body = captures
                .name("body")
                .expect("ListPages capture should contain a body")
                .as_str();

            ListPagesOccurrence {
                start: full_match.start(),
                end: full_match.end(),
                specification: parse_supported_specification(attributes, body),
            }
        })
        .collect()
}

fn parse_supported_specification(
    attribute_source: &str,
    body: &str,
) -> Option<SupportedListPages> {
    let attributes = parse_attributes(attribute_source)?;

    if attributes.len() != 5
        || attributes.get("parent") != Some(&".")
        || attributes.get("category") != Some(&FRAGMENT_CATEGORY)
        || attributes.get("order") != Some(&"created_at")
        || attributes.get("limit") != Some(&"1")
        || attributes.get("offset") != Some(&"@URL|0")
        || body.trim() != CONTENT_VARIABLE
    {
        return None;
    }

    // A page render has no request URL query in this layer, so @URL|0 uses its
    // declared fallback. Parsing a supplied URL offset is intentionally unsupported.
    Some(SupportedListPages {
        body_template: body.to_owned(),
        offset: 0,
    })
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
