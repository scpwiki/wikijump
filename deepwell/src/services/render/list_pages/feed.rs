/*
 * services/render/list_pages/feed.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::super::runtime::RenderRuntime;
use super::super::service::RenderService;
use super::super::{
    WikidotListPagesFeedInput, WikidotListPagesFeedItem, WikidotListPagesFeedOutput,
};
use super::presentation::{
    render_list_pages_snapshot_user, render_list_pages_wikidot_user,
};
use super::substitution::{parse_list_pages_score_selector, split_list_pages_values};
use crate::error::prelude::{Error, ErrorType, ExnError, Result, ResultExt};
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, DateSelector, FoundPageFields,
    IncludedCategories, OrderBySelector, PageParentSelector, PageQuery,
    PageQueryScoreFilterCache, PageTypeSelector, PaginationSelector, RangeSelector,
    TagCondition,
};
use crate::services::{PageRevisionService, ServiceContext, SiteService, TextService};
use crate::types::Reference;
use std::borrow::Cow;
use std::collections::BTreeMap;
use time::OffsetDateTime;

const WIKIDOT_RSS_ITEM_LIMIT: u64 = 20;
const MAX_WIKIDOT_RSS_SELECTOR_BYTES: usize = 8 * 1024;
const MAX_WIKIDOT_RSS_SELECTOR_VALUES: usize = 100;

impl RenderService {
    pub async fn render_wikidot_list_pages_feed(
        ctx: &ServiceContext<'_>,
        input: WikidotListPagesFeedInput,
    ) -> Result<WikidotListPagesFeedOutput> {
        let WikidotListPagesFeedInput {
            site_id,
            pagetype,
            category,
            tags,
            parent,
            created_by,
            rating,
            range,
        } = input;
        for selector in [
            pagetype.as_deref(),
            category.as_deref(),
            tags.as_deref(),
            parent.as_deref(),
            created_by.as_deref(),
            rating.as_deref(),
            range.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if selector.len() > MAX_WIKIDOT_RSS_SELECTOR_BYTES {
                return Err(Error::new(
                    "Wikidot ListPages feed selector is too long",
                    ErrorType::BadRequest,
                )
                .into());
            }
        }

        SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(|| {
                Error::new(
                    format!("failed to load site ID {site_id} for ListPages feed"),
                    ErrorType::Render,
                )
            })?;

        let page_type = match pagetype.as_deref().map(str::trim) {
            None | Some("") | Some("normal") => PageTypeSelector::Normal,
            Some("*") => PageTypeSelector::All,
            Some("hidden") => PageTypeSelector::Hidden,
            Some(_) => {
                return Err(wikidot_feed_argument_error("Invalid pagetype attribute."));
            }
        };

        let mut categories = Vec::new();
        let mut excluded_categories = Vec::new();
        let category_all = if let Some(category) = category.as_deref() {
            let values = bounded_feed_values(category)?;
            let mut saw_all = values.is_empty();
            for value in values {
                if value == "*" {
                    saw_all = true;
                    continue;
                }
                if let Some(value) = value.strip_prefix('-') {
                    excluded_categories.push(Cow::Owned(value.to_owned()));
                } else {
                    categories.push(Cow::Owned(
                        value.strip_prefix('+').unwrap_or(&value).to_owned(),
                    ));
                }
            }
            categories.is_empty() && saw_all
        } else {
            true
        };

        let mut any_tags = Vec::new();
        let mut all_tags = Vec::new();
        let mut no_tags = Vec::new();
        let mut untagged = false;
        let mut empty_result = match range.as_deref().map(str::trim) {
            None | Some("") => false,
            Some(".") => true,
            Some(_) => {
                return Err(wikidot_feed_argument_error("Invalid range argument."));
            }
        };
        if let Some(tags) = tags.as_deref() {
            for value in bounded_feed_values(tags)? {
                match value.as_str() {
                    "-" => untagged = true,
                    "=" | "==" => empty_result = true,
                    _ if value.starts_with('+') => {
                        all_tags.push(Cow::Owned(value[1..].to_owned()));
                    }
                    _ if value.starts_with('-') => {
                        no_tags.push(Cow::Owned(value[1..].to_owned()));
                    }
                    _ => any_tags.push(Cow::Owned(value)),
                }
            }
        }

        let parent_reference;
        let parent_references;
        let page_parent = match parent.as_deref().map(str::trim) {
            None | Some("") | Some("*") => PageParentSelector::All,
            Some("-") => PageParentSelector::NoParent,
            Some("." | "=" | "-=") => {
                empty_result = true;
                PageParentSelector::All
            }
            Some(parent) => {
                parent_reference = Reference::Slug(Cow::Owned(parent.to_owned()));
                parent_references = [parent_reference];
                PageParentSelector::HasParents(&parent_references)
            }
        };

        let created_by = created_by.as_deref().map(str::trim);
        if created_by == Some("=") {
            empty_result = true;
        }
        let authors = created_by
            .filter(|author| !author.is_empty())
            .filter(|author| *author != "=")
            .map(|author| vec![Cow::Owned(author.to_owned())])
            .unwrap_or_default();
        let resolved_authors = if authors.is_empty() {
            None
        } else {
            Some(
                Self::resolve_list_pages_authors(ctx, site_id, 0, &authors, true, false)
                    .await?,
            )
        };
        let score = match rating.as_deref().map(str::trim) {
            None | Some("") => Vec::new(),
            Some(rating) if wikidot_feed_rating_is_valid(rating) => {
                vec![parse_list_pages_score_selector(rating).ok_or_else(|| {
                    wikidot_feed_argument_error("Invalid rating argument.")
                })?]
            }
            Some(_) => {
                return Err(wikidot_feed_argument_error("Invalid rating argument."));
            }
        };

        let pages = if empty_result {
            Vec::new()
        } else {
            let query = PageQuery {
                current_page_id: 0,
                current_site_id: site_id,
                queried_site_id: None,
                page_type,
                categories: CategoriesSelector {
                    included_categories: if category_all {
                        IncludedCategories::All
                    } else {
                        IncludedCategories::List(&categories)
                    },
                    excluded_categories: &excluded_categories,
                },
                tags: TagCondition {
                    any_present: &any_tags,
                    all_present: &all_tags,
                    none_present: &no_tags,
                    untagged,
                },
                page_parent,
                contains_outgoing_links: &[],
                creation_date: DateSelector::FromPresent {
                    start: OffsetDateTime::UNIX_EPOCH,
                },
                update_date: DateSelector::FromPresent {
                    start: OffsetDateTime::UNIX_EPOCH,
                },
                author: resolved_authors
                    .as_ref()
                    .map_or(AuthorSelector::All, |authors| authors.as_selector()),
                score: &score,
                votes: &[],
                offset: 0,
                range: RangeSelector::Current,
                name: None,
                slug: None,
                slugs: &[],
                data_form_fields: &[],
                // The live feed endpoint ignores its serialized `order` path
                // and always uses ListPages' default newest-first order.
                order: Some(OrderBySelector::default()),
                candidate_limit: None,
                pagination: PaginationSelector {
                    limit: Some(WIKIDOT_RSS_ITEM_LIMIT),
                    per_page: WIKIDOT_RSS_ITEM_LIMIT as u8,
                    reversed: false,
                },
                variables: &[],
                fields: FoundPageFields {
                    title: true,
                    slug: true,
                    page_revision_id: true,
                    created_at: true,
                    created_by: true,
                    ..FoundPageFields::default()
                },
            };
            let mut permission_cache = BTreeMap::new();
            let mut score_filter_cache = PageQueryScoreFilterCache::default();
            RenderRuntime::new(ctx)
                .find_viewable_list_pages_rows(
                    query,
                    WIKIDOT_RSS_ITEM_LIMIT as usize,
                    &mut permission_cache,
                    Some(&mut score_filter_cache),
                )
                .await?
                .pages
                .pages
        };

        let user_displays = Self::load_wikidot_user_displays(ctx, &pages).await?;
        let snapshot_displays =
            Self::load_list_pages_snapshot_displays(ctx, &pages).await?;
        let mut items = Vec::with_capacity(pages.len());
        for page in pages {
            let revision = PageRevisionService::get_latest(ctx, site_id, page.page_id)
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to load a ListPages feed revision",
                        ErrorType::Render,
                    )
                })?;
            let body_html = TextService::get(ctx, &revision.compiled_body_html_hash)
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to load ListPages feed page HTML",
                        ErrorType::Render,
                    )
                })?;
            let snapshot = snapshot_displays.get(&page.page_id);
            let created_by_html = page
                .created_by
                .and_then(|user_id| {
                    user_displays.get(&user_id).map(|user| {
                        render_list_pages_wikidot_user(user_id, Some(user))
                            .replace(r#" data-wikijump-compat-listpages-user="1""#, "")
                    })
                })
                .or_else(|| {
                    snapshot
                        .and_then(|snapshot| snapshot.created_by_name.as_deref())
                        .map(render_list_pages_snapshot_user)
                })
                .unwrap_or_default();
            items.push(WikidotListPagesFeedItem {
                slug: page.slug.unwrap_or(revision.slug),
                title: page.title.unwrap_or(revision.title),
                created_at: snapshot
                    .map(|snapshot| snapshot.created_at)
                    .or(page.created_at)
                    .unwrap_or(revision.created_at),
                body_html,
                created_by_html,
            });
        }

        Ok(WikidotListPagesFeedOutput { items })
    }
}

fn bounded_feed_values(value: &str) -> Result<Vec<String>> {
    let values = split_list_pages_values(value);
    if values.len() > MAX_WIKIDOT_RSS_SELECTOR_VALUES {
        return Err(Error::new(
            "Wikidot ListPages feed selector has too many values",
            ErrorType::BadRequest,
        )
        .into());
    }
    Ok(values)
}

fn wikidot_feed_rating_is_valid(value: &str) -> bool {
    let value = [">=", "<=", "<>", ">", "<", "="]
        .into_iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .unwrap_or(value);
    let digits = value.strip_prefix('-').unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn wikidot_feed_argument_error(message: &'static str) -> ExnError {
    Error::new(message, ErrorType::BadRequest).into()
}
