/*
 * services/render/next_previous_page.rs
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

//! Wikidot `NextPage` and `PreviousPage` legacy module rendering.

use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::LazyLock;

use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;
use regex::Regex;

use super::compat::CompatHtmlFragments;
use super::compat::text_fragments::CompatTextFragments;
use super::list_pages::template::ListPagesTemplatePlan;
use super::list_pages::{
    ListPagesAuthorCacheKey, ListPagesBatchDisplayRequirements,
    ListPagesBlockRenderResult, ListPagesContentCache, ListPagesExpansionBudget,
    ListPagesPageContext, ResolvedListPagesAuthors, is_list_pages_visible_tag,
    parse_list_pages_arguments, register_generated_list_pages_html,
};
use super::literal_regions::LiteralRegionIndex;
use super::runtime::IncludeSourceCache;
use super::service::{
    IncludeExpansion, IncludeExpansionBudget, MAX_LISTPAGES_RENDER_SCAN_ROWS,
    RenderService,
};
use super::url_arguments::UrlArguments;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, DateSelector, FoundPageFields, FoundPageRow,
    FoundPages, IncludedCategories, OrderBySelector, OrderProperty, PageParentSelector,
    PageQuery, PageQueryScoreFilterCache, PageTypeSelector, PaginationSelector,
    RangeSelector, TagCondition,
};
use crate::services::{PageService, ServiceContext};

pub(super) static NEXT_PREVIOUS_PAGE_MODULE_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r"(?is)\[\[module\s+(?:NextPage|PreviousPage)(?:\s+[^\]]*)?\]\]")
            .expect("NextPreviousPage module-opening regular expression should compile")
    });

static NEXT_PREVIOUS_PAGE_OPEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?is)\[\[module\s+(?P<name>NextPage|PreviousPage)(?P<head>(?:\s+[^\]]*)?)\]\]",
    )
    .expect("NextPreviousPage module regular expression should compile")
});

static NEXT_PREVIOUS_ARGUMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)(?:^|\s)(?P<key>[A-Za-z_][A-Za-z0-9_-]*)\s*=\s*(?P<value>"[^"]*"|'[^']*'|[^\s\]]+)"#,
    )
    .expect("NextPreviousPage argument regular expression should compile")
});

#[derive(Debug)]
pub(super) struct NextPreviousPageExpansion {
    pub(super) wikitext: String,
    pub(super) included_pages: Vec<ftml::data::PageRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextPreviousModule {
    Next,
    Previous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextPreviousOrder {
    Date,
    Title,
}

#[derive(Debug)]
struct NextPreviousPageArguments {
    order: NextPreviousOrder,
    category_all: bool,
    categories: Vec<Cow<'static, str>>,
    any_tags: Vec<Cow<'static, str>>,
    all_tags: Vec<Cow<'static, str>>,
    no_tags: Vec<Cow<'static, str>>,
    current_visible_tags_any: bool,
    no_candidate_can_match: bool,
}

struct NextPreviousOccurrence<'a> {
    start: usize,
    end: usize,
    original: &'a str,
    name: NextPreviousModule,
    head: &'a str,
    body: &'a str,
}

#[derive(Debug)]
struct CurrentPageSortKey {
    page_id: i64,
    title: String,
    slug: String,
    created_at: time::OffsetDateTime,
}

impl RenderService {
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn expand_next_previous_page_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        include_budget: &mut IncludeExpansionBudget,
        url: UrlArguments<'_>,
        compat_html: &mut CompatHtmlFragments,
        include_source_cache: &mut IncludeSourceCache,
        compat_text: &mut CompatTextFragments,
    ) -> Result<NextPreviousPageExpansion> {
        if !settings.enable_page_syntax
            || !NEXT_PREVIOUS_PAGE_MODULE_OPEN_REGEX.is_match(&wikitext)
        {
            return Ok(NextPreviousPageExpansion {
                wikitext,
                included_pages: Vec::new(),
            });
        }
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(NextPreviousPageExpansion {
                wikitext,
                included_pages: Vec::new(),
            });
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let occurrences = find_next_previous_page_occurrences(&wikitext);
        if occurrences.is_empty() {
            return Ok(NextPreviousPageExpansion {
                wikitext,
                included_pages: Vec::new(),
            });
        }

        let current_key =
            load_current_next_previous_sort_key(ctx, current_page_id, page_info).await?;
        let mut expanded = String::with_capacity(wikitext.len());
        let mut included_pages = Vec::new();
        let mut cursor = 0;
        let mut permission_cache = BTreeMap::new();
        let mut content_cache = ListPagesContentCache::default();
        let mut expansion_budget = ListPagesExpansionBudget::new();
        let mut score_filter_cache = PageQueryScoreFilterCache::default();
        let mut author_resolution_cache =
            BTreeMap::<ListPagesAuthorCacheKey, ResolvedListPagesAuthors>::new();

        for occurrence in occurrences {
            if literal_regions.contains(occurrence.start) {
                continue;
            }

            expanded.push_str(&wikitext[cursor..occurrence.start]);
            cursor = occurrence.end;

            let arguments =
                parse_next_previous_page_arguments(occurrence.head, page_info, url);
            let Some(template) = ListPagesTemplatePlan::compile(occurrence.body) else {
                expanded
                    .push_str(&compat_text.push_escaped_html_text(occurrence.original));
                continue;
            };
            let pages = select_next_previous_page(
                ctx,
                current_site_id,
                current_page_id,
                &current_key,
                occurrence.name,
                &arguments,
                template.fields(),
                &mut permission_cache,
                &mut score_filter_cache,
            )
            .await?;
            let mut display_requirements = ListPagesBatchDisplayRequirements::default();
            display_requirements.include(&template);
            let prefetched_displays = if pages.pages.is_empty() {
                None
            } else {
                Some(
                    Self::load_list_pages_batch_displays(
                        ctx,
                        &pages.pages,
                        display_requirements,
                    )
                    .await?,
                )
            };
            let Some(list_pages_arguments) =
                parse_list_pages_arguments(r#" limit="1" perPage="1""#)
            else {
                expanded
                    .push_str(&compat_text.push_escaped_html_text(occurrence.original));
                continue;
            };

            let rendered = Box::pin(Self::render_list_pages_block(
                ctx,
                ListPagesPageContext {
                    site_id: current_site_id,
                    page_id: Some(current_page_id),
                    url,
                },
                page_info,
                settings,
                list_pages_arguments,
                &template,
                *include_budget,
                Some(pages),
                prefetched_displays.as_ref(),
                include_source_cache,
                &mut content_cache,
                &mut expansion_budget,
                &mut permission_cache,
                &mut score_filter_cache,
                &mut author_resolution_cache,
                compat_text,
            ))
            .await?;

            match rendered {
                ListPagesBlockRenderResult::Expanded(IncludeExpansion {
                    wikitext: replacement,
                    included_pages: replacement_included_pages,
                    expanded_include_count,
                }) => {
                    include_budget.consume(expanded_include_count);
                    expanded.push_str(&register_generated_list_pages_html(
                        replacement,
                        compat_html,
                    ));
                    included_pages.extend(replacement_included_pages);
                }
                ListPagesBlockRenderResult::PreserveOriginal => {
                    expanded.push_str(
                        &compat_text.push_escaped_html_text(occurrence.original),
                    );
                }
            }
        }

        if cursor == 0 {
            return Ok(NextPreviousPageExpansion {
                wikitext,
                included_pages: Vec::new(),
            });
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(NextPreviousPageExpansion {
            wikitext: expanded,
            included_pages,
        })
    }
}

fn find_next_previous_page_occurrences(source: &str) -> Vec<NextPreviousOccurrence<'_>> {
    let mut occurrences = Vec::new();
    let mut search_start = 0;
    while let Some(captures) =
        NEXT_PREVIOUS_PAGE_OPEN_REGEX.captures(&source[search_start..])
    {
        let matched = captures
            .get(0)
            .expect("a NextPreviousPage match always has a complete opening");
        let start = search_start + matched.start();
        let open_end = search_start + matched.end();
        let name = if captures["name"].eq_ignore_ascii_case("nextpage") {
            NextPreviousModule::Next
        } else {
            NextPreviousModule::Previous
        };
        let head = captures.name("head").map_or("", |head| head.as_str());
        let (end, body) = if source[open_end..]
            .chars()
            .next()
            .is_some_and(|character| matches!(character, '\n' | '\r'))
        {
            match source[open_end..]
                .to_ascii_lowercase()
                .find("[[/module]]")
                .map(|offset| open_end + offset)
            {
                Some(close_start) => {
                    let close_end = close_start + "[[/module]]".len();
                    (close_end, &source[open_end..close_start])
                }
                None => (open_end, ""),
            }
        } else {
            (open_end, "")
        };
        occurrences.push(NextPreviousOccurrence {
            start,
            end,
            original: &source[start..end],
            name,
            head,
            body,
        });
        search_start = end;
    }
    occurrences
}

fn parse_next_previous_page_arguments(
    head: &str,
    page_info: &PageInfo<'_>,
    url: UrlArguments<'_>,
) -> NextPreviousPageArguments {
    let mut order = NextPreviousOrder::Date;
    let mut categories = Vec::new();
    let mut category_all = false;
    let mut category_seen = false;
    let mut any_tags = Vec::new();
    let mut all_tags = Vec::new();
    let mut no_tags = Vec::new();
    let mut current_visible_tags_any = false;
    let mut no_candidate_can_match = false;

    for captures in NEXT_PREVIOUS_ARGUMENT_REGEX.captures_iter(head) {
        let key = captures["key"].to_ascii_lowercase();
        let value = unquote_next_previous_argument(captures["value"].trim()).trim();
        match key.as_str() {
            "by" => {
                order = match value {
                    "title" => NextPreviousOrder::Title,
                    "date" | "" => NextPreviousOrder::Date,
                    _ => order,
                };
            }
            "category" => {
                category_seen = true;
                categories.clear();
                category_all = false;
                for category in split_next_previous_values(value) {
                    if category == "*" {
                        category_all = true;
                    } else if !category.is_empty() {
                        categories.push(Cow::Owned(category));
                    }
                }
            }
            "tags" | "tag" => {
                any_tags.clear();
                all_tags.clear();
                no_tags.clear();
                current_visible_tags_any = false;
                no_candidate_can_match = false;
                let resolved_url_tag;
                let value = if value.eq_ignore_ascii_case("@URL") {
                    if let Some(tag) = url.tag.filter(|tag| !tag.is_empty()) {
                        resolved_url_tag = tag;
                        resolved_url_tag
                    } else {
                        no_candidate_can_match = true;
                        ""
                    }
                } else {
                    value
                };
                for tag in split_next_previous_values(value) {
                    if tag == "=" {
                        current_visible_tags_any = true;
                    } else if let Some(tag) = tag.strip_prefix('+') {
                        if !tag.is_empty() {
                            all_tags.push(Cow::Owned(tag.to_owned()));
                        }
                    } else if let Some(tag) = tag.strip_prefix('-') {
                        if !tag.is_empty() {
                            no_tags.push(Cow::Owned(tag.to_owned()));
                        }
                    } else if !tag.is_empty() {
                        any_tags.push(Cow::Owned(tag));
                    }
                }
            }
            _ => {
                // Live Wikidot ignores unknown NextPage / PreviousPage
                // attributes while applying the recognized ones in the same
                // invocation.
            }
        }
    }

    if !category_seen || (!category_all && categories.is_empty()) {
        categories = vec![Cow::Owned(
            RenderService::page_info_category_slug(page_info).into_owned(),
        )];
        category_all = false;
    }

    if current_visible_tags_any {
        let current_tags = page_info
            .tags
            .iter()
            .filter(|tag| is_list_pages_visible_tag(tag))
            .map(|tag| Cow::Owned(tag.to_string()))
            .collect::<Vec<_>>();
        if current_tags.is_empty() && any_tags.is_empty() {
            no_candidate_can_match = true;
        }
        any_tags.extend(current_tags);
    }

    NextPreviousPageArguments {
        order,
        category_all,
        categories,
        any_tags,
        all_tags,
        no_tags,
        current_visible_tags_any,
        no_candidate_can_match,
    }
}

fn unquote_next_previous_argument(value: &str) -> &str {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn split_next_previous_values(value: &str) -> impl Iterator<Item = String> + '_ {
    value
        .split(|character: char| character == ',' || character.is_whitespace())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

async fn load_current_next_previous_sort_key(
    ctx: &ServiceContext<'_>,
    current_page_id: i64,
    page_info: &PageInfo<'_>,
) -> Result<CurrentPageSortKey> {
    let page = PageService::get_direct(ctx, current_page_id, false)
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to load current page ID {current_page_id} for NextPreviousPage render",
                ),
                ErrorType::Render,
            )
        })?;
    Ok(CurrentPageSortKey {
        page_id: current_page_id,
        title: page_info.title.to_string(),
        slug: RenderService::page_info_full_slug(page_info),
        created_at: page.created_at,
    })
}

#[allow(clippy::too_many_arguments)]
async fn select_next_previous_page(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    current_page_id: i64,
    current_key: &CurrentPageSortKey,
    module: NextPreviousModule,
    arguments: &NextPreviousPageArguments,
    mut fields: FoundPageFields,
    permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    score_filter_cache: &mut PageQueryScoreFilterCache,
) -> Result<FoundPages> {
    if arguments.no_candidate_can_match {
        return Ok(FoundPages { pages: Vec::new() });
    }

    fields.title = true;
    fields.slug = true;
    fields.page_category_id = true;
    fields.created_at = true;
    fields.tags |= arguments.current_visible_tags_any;

    let included_categories = if arguments.category_all {
        IncludedCategories::All
    } else {
        IncludedCategories::List(&arguments.categories)
    };
    let order = match arguments.order {
        NextPreviousOrder::Date => OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: true,
        },
        NextPreviousOrder::Title => OrderBySelector {
            property: OrderProperty::Title,
            ascending: true,
        },
    };
    let query = PageQuery {
        current_page_id,
        current_site_id,
        queried_site_id: None,
        page_type: PageTypeSelector::Normal,
        categories: CategoriesSelector {
            included_categories,
            excluded_categories: &[],
        },
        tags: TagCondition {
            any_present: &arguments.any_tags,
            all_present: &arguments.all_tags,
            none_present: &arguments.no_tags,
            untagged: false,
        },
        page_parent: PageParentSelector::All,
        contains_outgoing_links: &[],
        creation_date: DateSelector::FromPresent {
            start: time::OffsetDateTime::UNIX_EPOCH,
        },
        update_date: DateSelector::FromPresent {
            start: time::OffsetDateTime::UNIX_EPOCH,
        },
        author: AuthorSelector::All,
        score: &[],
        votes: &[],
        offset: 0,
        range: RangeSelector::Current,
        name: None,
        slug: None,
        slugs: &[],
        data_form_fields: &[],
        order: Some(order),
        candidate_limit: Some(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)),
        pagination: PaginationSelector {
            limit: Some(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)),
            per_page: PaginationSelector::default().per_page,
            reversed: false,
        },
        variables: &[],
        fields,
    };

    let mut rows = super::runtime::RenderRuntime::new(ctx)
        .find_viewable_list_pages_rows(
            query,
            MAX_LISTPAGES_RENDER_SCAN_ROWS as usize,
            permission_cache,
            Some(score_filter_cache),
        )
        .await?
        .pages
        .pages;

    sort_next_previous_rows(&mut rows, arguments.order);
    let selected = match arguments.order {
        NextPreviousOrder::Date => select_date_neighbor(rows, module, current_key),
        NextPreviousOrder::Title => select_title_neighbor(rows, module, current_key),
    };

    Ok(FoundPages {
        pages: selected.into_iter().collect(),
    })
}

fn sort_next_previous_rows(rows: &mut [FoundPageRow], order: NextPreviousOrder) {
    match order {
        NextPreviousOrder::Date => rows.sort_by(compare_next_previous_date_rows),
        NextPreviousOrder::Title => rows.sort_by(compare_next_previous_title_rows),
    }
}

fn select_date_neighbor(
    rows: Vec<FoundPageRow>,
    module: NextPreviousModule,
    current_key: &CurrentPageSortKey,
) -> Option<FoundPageRow> {
    match module {
        NextPreviousModule::Previous => rows
            .into_iter()
            .rev()
            .find(|row| compare_row_to_current_date(row, current_key) == Ordering::Less),
        NextPreviousModule::Next => rows.into_iter().find(|row| {
            compare_row_to_current_date(row, current_key) == Ordering::Greater
        }),
    }
}

fn select_title_neighbor(
    rows: Vec<FoundPageRow>,
    module: NextPreviousModule,
    current_key: &CurrentPageSortKey,
) -> Option<FoundPageRow> {
    match module {
        // Live Wikidot's title-mode PreviousPage is inclusive and therefore
        // returns the current page itself when the current page matches the
        // candidate selector.
        NextPreviousModule::Previous => rows.into_iter().find(|row| {
            matches!(
                compare_row_to_current_title(row, current_key),
                Ordering::Equal | Ordering::Greater
            )
        }),
        NextPreviousModule::Next => rows.into_iter().find(|row| {
            compare_row_to_current_title(row, current_key) == Ordering::Greater
        }),
    }
}

fn compare_next_previous_date_rows(
    left: &FoundPageRow,
    right: &FoundPageRow,
) -> Ordering {
    next_previous_row_created_at(left)
        .cmp(&next_previous_row_created_at(right))
        .then_with(|| left.page_id.cmp(&right.page_id))
        .then_with(|| compare_next_previous_title_rows(left, right))
}

fn compare_row_to_current_date(
    row: &FoundPageRow,
    current: &CurrentPageSortKey,
) -> Ordering {
    next_previous_row_created_at(row)
        .cmp(&current.created_at)
        .then_with(|| row.page_id.cmp(&current.page_id))
        .then_with(|| compare_row_to_current_title(row, current))
}

fn next_previous_row_created_at(row: &FoundPageRow) -> time::OffsetDateTime {
    row.created_at.unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
}

fn compare_next_previous_title_rows(
    left: &FoundPageRow,
    right: &FoundPageRow,
) -> Ordering {
    let left_title = next_previous_row_title(left);
    let right_title = next_previous_row_title(right);
    left_title
        .to_ascii_lowercase()
        .cmp(&right_title.to_ascii_lowercase())
        .then_with(|| left_title.cmp(right_title))
        .then_with(|| {
            left.slug
                .as_deref()
                .unwrap_or("")
                .cmp(right.slug.as_deref().unwrap_or(""))
        })
        .then_with(|| left.page_id.cmp(&right.page_id))
}

fn compare_row_to_current_title(
    row: &FoundPageRow,
    current: &CurrentPageSortKey,
) -> Ordering {
    let title = next_previous_row_title(row);
    title
        .to_ascii_lowercase()
        .cmp(&current.title.to_ascii_lowercase())
        .then_with(|| title.cmp(current.title.as_str()))
        .then_with(|| row.slug.as_deref().unwrap_or("").cmp(current.slug.as_str()))
        .then_with(|| row.page_id.cmp(&current.page_id))
}

fn next_previous_row_title(row: &FoundPageRow) -> &str {
    row.title
        .as_deref()
        .or(row.slug.as_deref())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{NextPreviousOrder, parse_next_previous_page_arguments};
    use crate::services::render::UrlArguments;
    use ftml::data::PageInfo;
    use ftml::prelude::ScoreValue;
    use std::borrow::Cow;

    fn page_info<'a>(category: &'a str, tags: &[&'a str]) -> PageInfo<'a> {
        PageInfo {
            page: Cow::Borrowed("current"),
            category: Some(Cow::Borrowed(category)),
            site: Cow::Borrowed("test"),
            title: Cow::Borrowed("Current"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: tags.iter().copied().map(Cow::Borrowed).collect(),
            language: Cow::Borrowed("en"),
        }
    }

    #[test]
    fn argument_parser_uses_live_tolerant_defaults_and_ignores_malformed_tokens() {
        let info = page_info("current-category", &["shared", "_hidden"]);
        let parsed = parse_next_previous_page_arguments(
            r#" category="docs" by"title" foo="bar" tags="+required,= -blocked""#,
            &info,
            UrlArguments::default(),
        );

        assert_eq!(parsed.order, NextPreviousOrder::Date);
        assert!(!parsed.category_all);
        assert_eq!(parsed.categories[0].as_ref(), "docs");
        assert_eq!(parsed.all_tags[0].as_ref(), "required");
        assert_eq!(parsed.any_tags[0].as_ref(), "shared");
        assert_eq!(parsed.no_tags[0].as_ref(), "blocked");
    }
}
