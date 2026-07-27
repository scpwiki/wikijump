/*
 * services/render/list_pages/substitution.rs
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

use super::template::{LISTPAGES_VARIABLE_REGEX, ListPagesTemplatePlan};
use crate::services::page_query::{
    AuthorSelector, ComparisonOperation, CountPagesExactCountEligibilityDiagnostics,
    CountPagesExactCountEligibilityInput, DataFormSelector, DateSelector,
    DateTimeResolution, FoundPageFields, FoundPageRow, MAX_PAGE_QUERY_SCORE_SELECTORS,
    OrderBySelector, OrderProperty, PageParentSelector, PageQueryResultMetadata,
    PageTypeSelector, ScoreSelector, count_pages_exact_count_eligibility_diagnostics,
    normalize_wikidot_author_name,
};
use crate::services::render::UrlArguments;
use sea_orm::FromQueryResult;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
use wikidot_normalize::normalize;

use super::super::compat::CompatHtmlFragments;
use super::super::compat::preparation::neutralize_authored_markers;
use super::super::literal_regions::LiteralRegionCursor;
use super::super::percent_encoding::percent_encode_path_segment;
use super::super::runtime_page_queries::CountPagesRawScanCompletion;
use super::super::service::{
    CountPagesRequiredTagBatchResult, LISTPAGES_ARGUMENT_REGEX,
    MAX_LISTPAGES_RENDER_LIMIT, MAX_LISTPAGES_RENDER_OFFSET,
    MAX_LISTPAGES_RENDER_SCAN_ROWS, MAX_WIKIDOT_AJAX_MODULE_BODY_BYTES,
    MAX_WIKIDOT_AJAX_MODULE_PARAMETER_BYTES, MAX_WIKIDOT_AJAX_MODULE_PARAMETERS,
    RenderService, WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX,
    WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX, escape_list_pages_html_attr,
    escape_list_pages_html_text, format_list_pages_rating,
    format_wikidot_list_pages_date, native_numbered_list_content,
};
use super::content_sections::wikidot_content_section;
use super::scanner::{find_list_pages_module_matches, list_pages_runtime_head_is_safe};
use ftml::data::PageInfo;
use ftml::{self};

#[derive(Debug, Clone)]
pub(in crate::services::render) struct WikidotUserDisplay {
    pub(in crate::services::render) user_id: i64,
    pub(in crate::services::render) name: String,
    pub(in crate::services::render) slug: Option<String>,
    pub(in crate::services::render) wikidot_profile: bool,
}

#[derive(Debug, Clone)]
pub(in crate::services::render) struct ListPagesSnapshotDisplay {
    pub(in crate::services::render) created_at: time::OffsetDateTime,
    pub(in crate::services::render) updated_at: time::OffsetDateTime,
    pub(in crate::services::render) created_by_name: Option<String>,
    pub(in crate::services::render) updated_by_name: Option<String>,
    pub(in crate::services::render) comments: i32,
    pub(in crate::services::render) commented_at: Option<time::OffsetDateTime>,
    pub(in crate::services::render) commented_by_name: Option<String>,
    pub(in crate::services::render) rating_votes: Option<i64>,
    pub(in crate::services::render) parent_fullname: Option<String>,
    pub(in crate::services::render) source_revision_count: i32,
}

#[derive(Debug, FromQueryResult)]
pub(in crate::services::render) struct CurrentPageAuthorSource {
    pub(in crate::services::render) from_wikidot: bool,
    pub(in crate::services::render) snapshot_present: bool,
    pub(in crate::services::render) created_by_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::services::render) struct ListPagesAuthorCacheKey {
    pub(in crate::services::render) filter_present: bool,
    pub(in crate::services::render) negated: bool,
    pub(in crate::services::render) normalized_names: Vec<String>,
}

pub(in crate::services::render) fn list_pages_author_cache_key(
    author_names: &[Cow<'static, str>],
    author_filter_present: bool,
) -> ListPagesAuthorCacheKey {
    let normalized_names = author_names
        .iter()
        .filter_map(|author| {
            if author.as_ref() == "=" {
                Some("=".to_owned())
            } else {
                let normalized = normalize_wikidot_author_name(author);
                (!normalized.is_empty()).then_some(normalized)
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    ListPagesAuthorCacheKey {
        filter_present: author_filter_present,
        negated: false,
        normalized_names,
    }
}

#[derive(Debug, Clone)]
pub(in crate::services::render) enum ResolvedListPagesAuthors {
    All,
    Any {
        user_ids: Vec<i64>,
        wikidot_snapshot_names: Vec<Cow<'static, str>>,
    },
    NotAny {
        user_ids: Vec<i64>,
        wikidot_snapshot_names: Vec<Cow<'static, str>>,
    },
    None,
}

impl ResolvedListPagesAuthors {
    pub(in crate::services::render) fn as_selector(&self) -> AuthorSelector<'_> {
        match self {
            Self::All => AuthorSelector::All,
            Self::Any {
                user_ids,
                wikidot_snapshot_names,
            } => AuthorSelector::Any {
                user_ids,
                wikidot_snapshot_names,
            },
            Self::NotAny {
                user_ids,
                wikidot_snapshot_names,
            } => AuthorSelector::NotAny {
                user_ids,
                wikidot_snapshot_names,
            },
            Self::None => AuthorSelector::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::services::render) struct ListPagesArguments {
    pub(in crate::services::render) current_page_only: bool,
    pub(in crate::services::render) category_selector_present: bool,
    pub(in crate::services::render) category_all: bool,
    pub(in crate::services::render) include_current_category: bool,
    pub(in crate::services::render) categories: Vec<Cow<'static, str>>,
    pub(in crate::services::render) excluded_categories: Vec<Cow<'static, str>>,
    pub(in crate::services::render) any_tags: Vec<Cow<'static, str>>,
    pub(in crate::services::render) default_tags: Vec<Cow<'static, str>>,
    pub(in crate::services::render) all_tags: Vec<Cow<'static, str>>,
    pub(in crate::services::render) no_tags: Vec<Cow<'static, str>>,
    pub(in crate::services::render) untagged: bool,
    pub(in crate::services::render) authors: Vec<Cow<'static, str>>,
    pub(in crate::services::render) author_filter_present: bool,
    pub(in crate::services::render) order: Option<OrderBySelector>,
    pub(in crate::services::render) reverse: bool,
    pub(in crate::services::render) limit: Option<u64>,
    pub(in crate::services::render) count_pages_explicit_limit: Option<u64>,
    pub(in crate::services::render) count_pages_per_page: Option<u64>,
    pub(in crate::services::render) offset: u32,
    pub(in crate::services::render) offset_origin: ListPagesOffsetOrigin,
    pub(in crate::services::render) exclude_current_page: bool,
    pub(in crate::services::render) page_type: PageTypeSelector,
    pub(in crate::services::render) page_parent: PageParentSelector<'static>,
    pub(in crate::services::render) creation_date: DateSelector,
    pub(in crate::services::render) update_date: DateSelector,
    pub(in crate::services::render) score: Vec<ScoreSelector>,
    pub(in crate::services::render) slug: Option<Cow<'static, str>>,
    pub(in crate::services::render) name_pattern: Option<Cow<'static, str>>,
    pub(in crate::services::render) data_form_fields: Vec<DataFormSelector<'static>>,
    pub(in crate::services::render) prepend_line: Option<String>,
    pub(in crate::services::render) append_line: Option<String>,
    pub(in crate::services::render) separate: bool,
    pub(in crate::services::render) wrapper: bool,
    pub(in crate::services::render) exclude_current_page_author: bool,
    pub(in crate::services::render) unsupported_author_filter: bool,
    pub(in crate::services::render) unsupported_list_pages_filter: bool,
    pub(in crate::services::render) link_to: Vec<Cow<'static, str>>,
    pub(in crate::services::render) unsupported_score_filter: bool,
    pub(in crate::services::render) unsupported_count_pages_filter: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::services::render) enum ListPagesOffsetOrigin {
    Static,
    Url,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::services::render) struct ExactNameListPagesBatchKey {
    pub(in crate::services::render) category_all: bool,
    pub(in crate::services::render) categories: Vec<String>,
    pub(in crate::services::render) excluded_categories: Vec<String>,
}

#[derive(Debug, Default)]
pub(in crate::services::render) struct ListPagesBatchDisplays {
    pub(in crate::services::render) user_displays: BTreeMap<i64, WikidotUserDisplay>,
    pub(in crate::services::render) snapshot_displays:
        BTreeMap<i64, ListPagesSnapshotDisplay>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::services::render) struct ListPagesBatchDisplayRequirements {
    pub(in crate::services::render) users: bool,
    pub(in crate::services::render) snapshots: bool,
}

impl ListPagesBatchDisplayRequirements {
    pub(in crate::services::render) fn include(
        &mut self,
        template: &ListPagesTemplatePlan,
    ) {
        let users = template.uses_created_by() || template.uses_updated_by();
        self.users |= users;
        self.snapshots |= users
            || template.uses_created_at()
            || template.uses_updated_at()
            || template.uses_comments()
            || template.uses_commented_by()
            || template.uses_commented_at()
            || template.uses_rating_votes();
    }
}

pub(in crate::services::render) fn exact_name_list_pages_batch_key(
    head: &str,
    template: &ListPagesTemplatePlan,
    arguments: &ListPagesArguments,
    current_category: &str,
) -> Option<ExactNameListPagesBatchKey> {
    let unparsed = LISTPAGES_ARGUMENT_REGEX.replace_all(head, "");
    if !unparsed.trim().is_empty() || template.uses_content() || template.uses_data_form()
    {
        return None;
    }

    let mut name_arguments = 0;
    for captures in LISTPAGES_ARGUMENT_REGEX.captures_iter(head) {
        if captures.name("op").map_or("=", |matched| matched.as_str()) != "=" {
            return None;
        }
        match captures["key"].to_ascii_lowercase().as_str() {
            "name" | "fullname" | "full_slug" | "fullslug" => {
                name_arguments += 1;
            }
            "category" => {}
            _ => return None,
        }
    }
    if name_arguments != 1 || arguments.slug.is_none() {
        return None;
    }

    let mut categories = arguments
        .categories
        .iter()
        .map(|category| category.to_string())
        .collect::<Vec<_>>();
    let category_all = if arguments.category_selector_present {
        if arguments.include_current_category {
            categories.push(current_category.to_owned());
        }
        arguments.category_all
    } else {
        categories.push(current_category.to_owned());
        false
    };

    Some(ExactNameListPagesBatchKey {
        category_all,
        categories,
        excluded_categories: arguments
            .excluded_categories
            .iter()
            .map(|category| category.to_string())
            .collect(),
    })
}

pub(in crate::services::render) fn union_found_page_fields(
    left: &mut FoundPageFields,
    right: &FoundPageFields,
) {
    left.title |= right.title;
    left.alt_title |= right.alt_title;
    left.slug |= right.slug;
    left.page_category_id |= right.page_category_id;
    left.page_revision_id |= right.page_revision_id;
    left.tags |= right.tags;
    left.created_at |= right.created_at;
    left.created_by |= right.created_by;
    left.updated_at |= right.updated_at;
    left.updated_by |= right.updated_by;
    left.score |= right.score;
}

pub(in crate::services::render) fn parse_list_pages_arguments(
    head: &str,
) -> Option<ListPagesArguments> {
    parse_list_pages_arguments_with_url(head, UrlArguments::default())
}

/// Parse a ListPages head against the request that is asking for it.
///
/// `url` carries the Wikidot URL path arguments, which a selector can name as
/// `@URL`. Pass the default for any render that is not serving a page view,
/// including the render that produces a revision's stored HTML.
pub(in crate::services::render) fn parse_list_pages_arguments_with_url(
    head: &str,
    url: UrlArguments<'_>,
) -> Option<ListPagesArguments> {
    if !list_pages_runtime_head_is_safe(head) {
        return None;
    }
    let unparsed = LISTPAGES_ARGUMENT_REGEX.replace_all(head, "");
    if !unparsed.trim().is_empty() {
        return None;
    }

    let mut category_all = true;
    let mut category_selector_present = false;
    let mut category_argument_is_plural = None;
    let mut current_page_only = false;
    let mut include_current_category = false;
    let mut categories = Vec::new();
    let mut excluded_categories = Vec::new();
    let any_tags = Vec::new();
    let mut default_tags = Vec::new();
    let mut all_tags = Vec::new();
    let mut no_tags = Vec::new();
    let mut untagged = false;
    let mut authors = Vec::new();
    let mut author_filter_present = false;
    let mut order = None;
    let mut reverse = false;
    let mut limit = None;
    let mut count_pages_explicit_limit = None;
    let mut count_pages_per_page = None;
    let mut offset = 0;
    let mut offset_origin = ListPagesOffsetOrigin::Static;
    let mut exclude_current_page = false;
    let mut page_type = PageTypeSelector::Normal;
    let mut page_parent = PageParentSelector::All;
    let mut creation_date = DateSelector::FromPresent {
        start: time::OffsetDateTime::UNIX_EPOCH,
    };
    let mut update_date = DateSelector::FromPresent {
        start: time::OffsetDateTime::UNIX_EPOCH,
    };
    let mut score = Vec::new();
    let mut slug = None;
    let mut name_pattern = None;
    let mut data_form_fields = Vec::new();
    let mut prepend_line = None;
    let mut append_line = None;
    let mut separate = true;
    let mut wrapper = true;
    let mut unsupported_author_filter = false;
    let mut exclude_current_page_author = false;
    let mut unsupported_list_pages_filter = false;
    let mut link_to = Vec::new();
    let mut unsupported_score_filter = false;
    let mut unsupported_count_pages_filter = false;

    for captures in LISTPAGES_ARGUMENT_REGEX.captures_iter(head) {
        let raw_key = &captures["key"];
        let key = raw_key.to_ascii_lowercase();
        let value = captures
            .name("double")
            .or_else(|| captures.name("single"))
            .or_else(|| captures.name("bare"))
            .unwrap()
            .as_str()
            .trim();
        if captures.name("op").map_or("=", |matched| matched.as_str()) != "="
            && !key.starts_with('_')
        {
            return None;
        }

        match key.as_str() {
            "tags" => {
                let resolved_url_tag;
                let value = match resolve_url_selector(value, url.tag) {
                    UrlSelector::Static(value) => value,
                    UrlSelector::Resolved(tag) => {
                        // A resolved `@URL` still leaves CountPages literal:
                        // its own URL-argument behavior has not been captured.
                        unsupported_count_pages_filter = true;
                        resolved_url_tag = tag;
                        resolved_url_tag.as_str()
                    }
                    UrlSelector::Dropped => {
                        unsupported_count_pages_filter = true;
                        continue;
                    }
                };
                for tag in split_list_pages_values(value) {
                    if is_no_tags_selector(&tag) {
                        untagged = true;
                        unsupported_count_pages_filter = true;
                        continue;
                    }
                    if is_current_page_tag_selector(&tag) {
                        unsupported_count_pages_filter = true;
                        unsupported_list_pages_filter = true;
                        continue;
                    }
                    if let Some(tag) = tag.strip_prefix('-') {
                        no_tags.push(Cow::Owned(tag.to_owned()));
                    } else if let Some(tag) = tag.strip_prefix('+') {
                        all_tags.push(Cow::Owned(tag.to_owned()));
                    } else {
                        default_tags.push(Cow::Owned(tag));
                    }
                }
            }
            "tag" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                for tag in split_list_pages_values(value) {
                    if is_no_tags_selector(&tag) {
                        untagged = true;
                        unsupported_count_pages_filter = true;
                        continue;
                    }
                    if is_current_page_tag_selector(&tag) {
                        unsupported_count_pages_filter = true;
                        unsupported_list_pages_filter = true;
                        continue;
                    }
                    if let Some(tag) = tag.strip_prefix('-') {
                        no_tags.push(Cow::Owned(tag.to_owned()));
                    } else if let Some(tag) = tag.strip_prefix('+') {
                        all_tags.push(Cow::Owned(tag.to_owned()));
                    } else {
                        default_tags.push(Cow::Owned(tag));
                    }
                }
            }
            "category" | "categories" => {
                let is_plural = key == "categories";
                if category_argument_is_plural
                    .is_some_and(|previous| previous != is_plural)
                {
                    return None;
                }
                category_argument_is_plural.get_or_insert(is_plural);
                let mut saw_included_category = false;
                let resolved_url_category;
                let value = match resolve_url_selector(value, url.category) {
                    UrlSelector::Static(value) => value,
                    UrlSelector::Resolved(category) => {
                        // As with a resolved tag, CountPages stays literal:
                        // its own URL-argument behavior is uncaptured.
                        unsupported_count_pages_filter = true;
                        resolved_url_category = category;
                        resolved_url_category.as_str()
                    }
                    UrlSelector::Dropped => {
                        // A dropped selector leaves the module exactly as it
                        // would be with no `category` argument at all, which
                        // live answers with the current page's category. It
                        // must not be recorded as a selector, or the query
                        // widens to every category instead.
                        unsupported_count_pages_filter = true;
                        continue;
                    }
                };
                category_selector_present = true;
                for category in split_list_pages_values(value) {
                    if category == "*" {
                        category_all = true;
                    } else if category == "." {
                        include_current_category = true;
                        saw_included_category = true;
                    } else if let Some(category) = category.strip_prefix('+') {
                        categories.push(Cow::Owned(category.to_owned()));
                        saw_included_category = true;
                    } else if let Some(category) = category.strip_prefix('-') {
                        excluded_categories.push(Cow::Owned(category.to_owned()));
                    } else {
                        categories.push(Cow::Owned(category));
                        saw_included_category = true;
                    }
                }
                if saw_included_category {
                    category_all = false;
                }
            }
            "limit" => {
                let parsed = parse_list_pages_numeric_argument(value)?;
                limit = Some(parsed);
                count_pages_explicit_limit = Some(parsed);
            }
            "perpage" | "per_page" => {
                let parsed = parse_list_pages_numeric_argument(value)?;
                count_pages_per_page = Some(parsed);
            }
            "offset" => {
                let parsed = if is_dynamic_list_pages_value(value) {
                    match url
                        .offset
                        .filter(|offset| *offset <= MAX_LISTPAGES_RENDER_OFFSET)
                    {
                        Some(offset) => {
                            offset_origin = ListPagesOffsetOrigin::Url;
                            u64::from(offset)
                        }
                        None => {
                            offset_origin = ListPagesOffsetOrigin::Fallback;
                            list_pages_url_fallback(value).unwrap_or("0").parse().ok()?
                        }
                    }
                } else {
                    offset_origin = ListPagesOffsetOrigin::Static;
                    value.parse().ok()?
                };
                if parsed > u64::from(MAX_LISTPAGES_RENDER_OFFSET) {
                    return None;
                }
                offset = parsed as u32;
            }
            "pagetype" | "page_type" | "page-type" => {
                page_type = parse_list_pages_page_type(value)?;
            }
            "parent" => {
                let value = list_pages_url_fallback(value).unwrap_or(value);
                match value {
                    "." => page_parent = PageParentSelector::ChildOf,
                    "*" | "" => page_parent = PageParentSelector::All,
                    _ if is_dynamic_list_pages_value(value) => return None,
                    _ => return None,
                }
            }
            "prependline" | "prepend_line" => {
                prepend_line = Some(value.to_owned());
            }
            "appendline" => {
                append_line = Some(value.to_owned());
            }
            "order" => {
                if value.is_empty() {
                    continue;
                }
                let value = list_pages_url_fallback(value).unwrap_or(value);
                order = Some(parse_list_pages_order(value)?);
            }
            "reverse" => match value.to_ascii_lowercase().as_str() {
                "yes" => reverse = true,
                _ => return None,
            },
            "name" | "fullname" | "full_slug" | "fullslug" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                if value == "=" {
                    current_page_only = true;
                    limit = Some(1);
                } else if !is_dynamic_list_pages_value(value) {
                    let value = wikidot_list_pages_name_slug(value);
                    if value.contains(['*', '%']) {
                        name_pattern = Some(Cow::Owned(value));
                    } else {
                        slug = Some(Cow::Owned(value));
                    }
                }
            }
            // These inputs need additional data or Wikidot semantics that are not
            // implemented by PageQueryService yet. Leaving the module untouched is
            // safer than silently returning a wrong list.
            "separate" => {
                separate = parse_list_pages_boolean_argument(value)?;
            }
            "created_by" | "createdby" => {
                author_filter_present = true;
                if is_dynamic_list_pages_value(value)
                    && list_pages_url_fallback(value).is_none()
                {
                    unsupported_author_filter = true;
                }
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                let author = value
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim();
                // Wikidot's `=` resolves to the author of the page holding
                // the module, so `-=` excludes that author rather than the
                // viewer.
                if author == "-=" {
                    exclude_current_page_author = true;
                    unsupported_count_pages_filter = true;
                    continue;
                }
                if !author.is_empty() {
                    authors.push(Cow::Owned(author.to_owned()));
                }
            }
            "range" => match value {
                "." => {
                    current_page_only = true;
                    limit = Some(1);
                }
                "others" | "other" => {
                    exclude_current_page = true;
                }
                "before" | "after" => {
                    unsupported_count_pages_filter = true;
                    unsupported_list_pages_filter = true;
                }
                _ => {}
            },
            "wrapper" => {
                wrapper = parse_list_pages_boolean_argument(value)?;
            }
            "rating" | "score" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                if score.len() == MAX_PAGE_QUERY_SCORE_SELECTORS {
                    unsupported_score_filter = true;
                } else {
                    score.push(parse_list_pages_score_selector(value)?);
                }
            }
            "created_at" | "createdat" | "date" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                creation_date = parse_list_pages_date_selector(value)?;
            }
            "updated_at" | "updatedat" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                update_date = parse_list_pages_date_selector(value)?;
            }
            "link_to" | "linkto" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    unsupported_list_pages_filter = true;
                    continue;
                };
                let target = value.trim();
                if target.is_empty() || target.contains(',') {
                    unsupported_count_pages_filter = true;
                    unsupported_list_pages_filter = true;
                    continue;
                }
                unsupported_count_pages_filter = true;
                let mut target = target.to_owned();
                normalize(&mut target);
                link_to.push(Cow::Owned(target));
            }
            "votes" | "form" | "urlattrprefix" => {
                unsupported_count_pages_filter = true;
                unsupported_list_pages_filter = true;
            }
            // Wikidot accepts these arguments without applying them to the
            // ListPages query or wrapper. Do not forward author-controlled
            // values into generated markup.
            "class" | "custom" | "style" | "unknown" => {}
            _ if raw_key.starts_with('_') => {
                let value = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                )?;
                let field = raw_key
                    .strip_prefix('_')
                    .expect("data form selector should start with an underscore");
                if field.is_empty() || is_dynamic_list_pages_value(value) {
                    return None;
                }
                data_form_fields.push(DataFormSelector {
                    field: Cow::Owned(field.to_owned()),
                    value: Cow::Owned(value.to_owned()),
                    negated: &captures["op"] == "!=",
                });
            }
            _ => return None,
        }
    }

    Some(ListPagesArguments {
        current_page_only,
        category_selector_present,
        category_all,
        include_current_category,
        categories,
        excluded_categories,
        any_tags,
        default_tags,
        all_tags,
        no_tags,
        untagged,
        authors,
        author_filter_present,
        order,
        reverse,
        limit,
        count_pages_explicit_limit,
        count_pages_per_page,
        offset,
        offset_origin,
        exclude_current_page,
        page_type,
        page_parent,
        creation_date,
        update_date,
        score,
        slug,
        name_pattern,
        data_form_fields,
        prepend_line,
        append_line,
        separate,
        wrapper,
        exclude_current_page_author,
        unsupported_author_filter,
        unsupported_list_pages_filter,
        link_to,
        unsupported_score_filter,
        unsupported_count_pages_filter,
    })
}

pub(in crate::services::render) fn count_pages_should_remain_literal(
    arguments: &ListPagesArguments,
) -> bool {
    let count_pages_bound = arguments
        .count_pages_explicit_limit
        .or(arguments.count_pages_per_page);
    arguments.unsupported_author_filter
        || arguments.unsupported_score_filter
        || arguments.unsupported_count_pages_filter
        || (count_pages_bound.is_none()
            && !arguments.current_page_only
            && !count_pages_has_static_filter(arguments))
        || count_pages_bound.is_some_and(|limit| {
            limit
                .saturating_add(u64::from(arguments.offset))
                .saturating_add(u64::from(arguments.exclude_current_page))
                > u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
        })
        || (arguments.category_selector_present
            && arguments.category_all
            && arguments.count_pages_explicit_limit.is_none()
            && !count_pages_has_static_filter(arguments))
        || (arguments.current_page_only
            && (arguments.category_selector_present
                || arguments.page_type != PageTypeSelector::Normal
                || arguments.page_parent != PageParentSelector::All
                || !arguments.default_tags.is_empty()
                || !arguments.any_tags.is_empty()
                || !arguments.all_tags.is_empty()
                || !arguments.no_tags.is_empty()
                || arguments.author_filter_present
                || !arguments.excluded_categories.is_empty()
                || arguments.creation_date
                    != (DateSelector::FromPresent {
                        start: time::OffsetDateTime::UNIX_EPOCH,
                    })
                || arguments.update_date
                    != (DateSelector::FromPresent {
                        start: time::OffsetDateTime::UNIX_EPOCH,
                    })
                || !arguments.score.is_empty()
                || !arguments.data_form_fields.is_empty()
                || arguments.slug.is_some()
                || arguments.name_pattern.is_some()))
}

pub(in crate::services::render) fn count_pages_capture_is_literal(
    literal_regions: &mut LiteralRegionCursor<'_>,
    offset: usize,
) -> bool {
    literal_regions.containing_end(offset).is_some()
}

pub(in crate::services::render) fn count_pages_required_tag_batch_result(
    raw_total: i64,
    can_view: Option<bool>,
) -> CountPagesRequiredTagBatchResult {
    let Some(can_view) = can_view else {
        return CountPagesRequiredTagBatchResult::PreserveLiteral;
    };
    if !can_view {
        return CountPagesRequiredTagBatchResult::Exact(0);
    }
    let Ok(raw_total) = u64::try_from(raw_total) else {
        return CountPagesRequiredTagBatchResult::PreserveLiteral;
    };
    if raw_total >= u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS) {
        return CountPagesRequiredTagBatchResult::PreserveLiteral;
    }

    CountPagesRequiredTagBatchResult::Exact(raw_total as usize)
}

pub(in crate::services::render) fn count_pages_required_tag_batch_selector(
    arguments: &ListPagesArguments,
) -> Option<&str> {
    let required_tag = match (
        arguments.default_tags.as_slice(),
        arguments.all_tags.as_slice(),
    ) {
        ([tag], []) | ([], [tag]) => tag.as_ref(),
        _ => return None,
    };
    if arguments.current_page_only
        || arguments.category_selector_present
        || !arguments.any_tags.is_empty()
        || arguments.author_filter_present
        || arguments.order.is_some()
        || arguments.limit.is_some()
        || arguments.count_pages_explicit_limit.is_some()
        || arguments.count_pages_per_page.is_some()
        || arguments.offset != 0
        || arguments.exclude_current_page
        || arguments.page_type != PageTypeSelector::Normal
        || arguments.page_parent != PageParentSelector::All
        || arguments.creation_date
            != (DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            })
        || arguments.update_date
            != (DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            })
        || !arguments.score.is_empty()
        || arguments.slug.is_some()
        || arguments.name_pattern.is_some()
        || !arguments.data_form_fields.is_empty()
        || arguments.unsupported_author_filter
        || arguments.unsupported_count_pages_filter
    {
        return None;
    }

    Some(required_tag)
}

pub(in crate::services::render) fn count_pages_has_static_filter(
    arguments: &ListPagesArguments,
) -> bool {
    !arguments.categories.is_empty()
        || !arguments.default_tags.is_empty()
        || !arguments.any_tags.is_empty()
        || !arguments.all_tags.is_empty()
        || arguments.author_filter_present
        || arguments.page_type != PageTypeSelector::Normal
        || arguments.page_parent != PageParentSelector::All
        || arguments.creation_date
            != (DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            })
        || arguments.update_date
            != (DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            })
        || !arguments.score.is_empty()
        || arguments.slug.is_some()
        || arguments.name_pattern.is_some()
        || !arguments.data_form_fields.is_empty()
}

pub(in crate::services::render) fn count_pages_exact_count_render_diagnostics(
    metadata: PageQueryResultMetadata,
    view_permission_filtering_applied: bool,
    post_query_exclusion_applied: bool,
    post_query_offset_applied: bool,
    count_pages_explicit_limit: Option<u64>,
    count_pages_query_limit: u64,
) -> CountPagesExactCountEligibilityDiagnostics {
    let explicit_count_pages_bound_matches_sql_window =
        count_pages_explicit_limit.is_some_and(|limit| limit == count_pages_query_limit);

    count_pages_exact_count_eligibility_diagnostics(
        CountPagesExactCountEligibilityInput {
            metadata,
            view_permission_filtering_applied,
            post_query_filtering_applied: false,
            post_query_exclusion_applied,
            post_query_offset_applied,
            explicit_count_pages_bound_matches_sql_window,
        },
    )
}

pub(in crate::services::render) fn count_pages_unbounded_total(
    raw_scan_completion: CountPagesRawScanCompletion,
    scanned_total: usize,
) -> Option<usize> {
    match raw_scan_completion {
        CountPagesRawScanCompletion::Complete => Some(scanned_total),
        CountPagesRawScanCompletion::Capped => None,
    }
}

pub(in crate::services::render) fn page_query_cap_requires_original_module(
    metadata: &PageQueryResultMetadata,
) -> bool {
    metadata.cap_exceeded
}

pub(in crate::services::render) fn count_pages_scan_requires_preservation(
    raw_scan_completion: CountPagesRawScanCompletion,
    viewable_count: usize,
    target_count: usize,
) -> bool {
    // ListPages may render the viewable prefix from a capped permission scan.
    // CountPages must preserve its module unless that same scan filled its exact bound.
    raw_scan_completion == CountPagesRawScanCompletion::Capped
        && viewable_count < target_count
}

pub(in crate::services::render) fn list_pages_row_scan_target(
    requested_limit: u64,
    overall_limit: Option<u64>,
    per_page: Option<u64>,
    offset: u32,
    exclude_current_page: bool,
) -> u64 {
    let rows = if per_page.is_some() {
        overall_limit.unwrap_or(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
    } else {
        requested_limit
    };
    rows.saturating_add(u64::from(offset))
        .saturating_add(u64::from(exclude_current_page))
        .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
}

pub(in crate::services::render) fn list_pages_content_query_target(
    query_limit: u64,
    requested_limit: u64,
    remaining_content_rows: usize,
    offset: u32,
    exclude_current_page: bool,
    has_pager: bool,
) -> u64 {
    if has_pager {
        return query_limit;
    }
    // One row beyond the remaining allowance distinguishes a sparse broad query from a true overflow without scanning its full declared range.
    let selected_rows_needed = requested_limit.min(
        u64::try_from(remaining_content_rows)
            .unwrap_or(u64::MAX)
            .saturating_add(1),
    );
    query_limit.min(
        selected_rows_needed
            .saturating_add(u64::from(offset))
            .saturating_add(u64::from(exclude_current_page)),
    )
}

pub(in crate::services::render) fn should_render_current_page_list_pages_row(
    current_page_only: bool,
    limit: Option<u64>,
    offset: u32,
) -> bool {
    current_page_only && limit.unwrap_or(1) > 0 && offset == 0
}

pub(in crate::services::render) fn requested_page_info_score(
    fields: &FoundPageFields,
    page_info: &PageInfo<'_>,
) -> Option<f32> {
    fields.score.then(|| page_info.score.to_f64() as f32)
}

pub(in crate::services::render) fn current_page_info_list_pages_row(
    current_site_id: i64,
    current_page_id: i64,
    page_info: &PageInfo<'_>,
    fields: &FoundPageFields,
) -> Option<FoundPageRow> {
    if fields.page_category_id
        || fields.page_revision_id
        || fields.created_at
        || fields.created_by
        || fields.updated_at
        || fields.updated_by
    {
        return None;
    }

    Some(FoundPageRow {
        page_id: current_page_id,
        site_id: current_site_id,
        title: fields.title.then(|| page_info.title.to_string()),
        alt_title: fields
            .alt_title
            .then_some(page_info.alt_title.as_ref())
            .flatten()
            .map(ToString::to_string),
        slug: fields
            .slug
            .then(|| RenderService::page_info_full_slug(page_info)),
        page_category_id: None,
        page_revision_id: None,
        tags: fields
            .tags
            .then(|| page_info.tags.iter().map(ToString::to_string).collect()),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: requested_page_info_score(fields, page_info),
    })
}

pub(in crate::services::render) fn parse_list_pages_numeric_argument(
    value: &str,
) -> Option<u64> {
    if let Some(fallback) = list_pages_url_fallback(value) {
        return fallback.parse().ok();
    }

    value.parse().ok()
}

pub(in crate::services::render) fn parse_list_pages_boolean_argument(
    value: &str,
) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" => Some(true),
        "no" | "false" => Some(false),
        _ => None,
    }
}

pub(in crate::services::render) fn build_wikidot_list_pages_module_source(
    module_body: String,
    parameters: &BTreeMap<String, String>,
) -> Option<String> {
    if module_body.len() > MAX_WIKIDOT_AJAX_MODULE_BODY_BYTES
        || parameters.len() > MAX_WIKIDOT_AJAX_MODULE_PARAMETERS
    {
        return None;
    }

    let mut source = String::from("[[module ListPages");
    for (key, value) in parameters {
        let normalized_key = key.to_ascii_lowercase();
        if !matches!(
            normalized_key.as_str(),
            "pagetype"
                | "page_type"
                | "page-type"
                | "category"
                | "tags"
                | "tag"
                | "parent"
                | "created_at"
                | "createdat"
                | "updated_at"
                | "updatedat"
                | "created_by"
                | "createdby"
                | "rating"
                | "score"
                | "name"
                | "fullname"
                | "full_slug"
                | "fullslug"
                | "range"
                | "order"
                | "offset"
                | "limit"
                | "perpage"
                | "per_page"
                | "separate"
                | "wrapper"
        ) || value.len() > MAX_WIKIDOT_AJAX_MODULE_PARAMETER_BYTES
            || value.chars().any(|character| character.is_control())
            || value.contains("]]")
        {
            return None;
        }
        let current_page_dependent = (matches!(
            normalized_key.as_str(),
            "name" | "fullname" | "full_slug" | "fullslug"
        ) && value.trim() == "=")
            || (normalized_key == "range" && value.trim() == ".")
            || (normalized_key == "parent" && value.trim() == ".")
            || (normalized_key == "category"
                && split_list_pages_values(value)
                    .iter()
                    .any(|category| category == "."));
        if current_page_dependent {
            return None;
        }

        let (quote, quoted_value) = if !value.contains('"') {
            ('"', value.as_str())
        } else if !value.contains('\'') {
            ('\'', value.as_str())
        } else {
            return None;
        };
        source.push(' ');
        source.push_str(key);
        source.push('=');
        source.push(quote);
        source.push_str(quoted_value);
        source.push(quote);
    }
    source.push_str("]]\n");
    source.push_str(&module_body);
    source.push_str("\n[[/module]]");
    let modules = find_list_pages_module_matches(&source);
    (modules.len() == 1
        && modules[0].start == 0
        && modules[0].end == source.len()
        && modules[0].runtime_safe)
        .then_some(source)
}

pub(in crate::services::render) fn parse_list_pages_score_selector(
    value: &str,
) -> Option<ScoreSelector> {
    let (comparison, value) = parse_list_pages_comparison(value);
    let score = if let Ok(value) = value.parse::<i64>() {
        ftml::data::ScoreValue::Integer(value)
    } else {
        ftml::data::ScoreValue::Float(value.parse().ok()?)
    };
    Some(ScoreSelector { score, comparison })
}

pub(in crate::services::render) fn parse_list_pages_date_selector(
    value: &str,
) -> Option<DateSelector> {
    let value = value.trim();
    let words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() == 3
        && words[0].eq_ignore_ascii_case("older")
        && words[1].eq_ignore_ascii_case("than")
    {
        let amount = words[2].parse().ok()?;
        return Some(DateSelector::Span {
            timestamp: subtract_wikidot_relative_time(
                time::OffsetDateTime::now_utc(),
                amount,
                "day",
            )?,
            resolution: DateTimeResolution::Second,
            comparison: ComparisonOperation::LessThan,
        });
    }
    if words.len() == 4
        && words[0].eq_ignore_ascii_case("older")
        && words[1].eq_ignore_ascii_case("than")
    {
        let amount = words[2].parse().ok()?;
        return Some(DateSelector::Span {
            timestamp: subtract_wikidot_relative_time(
                time::OffsetDateTime::now_utc(),
                amount,
                words[3],
            )?,
            resolution: DateTimeResolution::Second,
            comparison: ComparisonOperation::LessThan,
        });
    }
    if words.len() == 4
        && words[0].eq_ignore_ascii_case("newer")
        && words[1].eq_ignore_ascii_case("than")
    {
        let amount = words[2].parse().ok()?;
        return Some(DateSelector::Span {
            timestamp: subtract_wikidot_relative_time(
                time::OffsetDateTime::now_utc(),
                amount,
                words[3],
            )?,
            resolution: DateTimeResolution::Second,
            comparison: ComparisonOperation::GreaterThan,
        });
    }
    if words.len() == 3 && words[0].eq_ignore_ascii_case("last") {
        let amount = words[1].parse().ok()?;
        return Some(DateSelector::FromPresent {
            start: subtract_wikidot_relative_time(
                time::OffsetDateTime::now_utc(),
                amount,
                words[2],
            )?,
        });
    }

    let (comparison, date) = parse_list_pages_comparison(value);
    let parts = date.split('.').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    let year = parts[0].trim().parse::<i32>().ok()?;
    let month_number = parts
        .get(1)
        .map_or(Some(1), |part| part.trim().parse::<u8>().ok())?;
    let day = parts
        .get(2)
        .map_or(Some(1), |part| part.trim().parse::<u8>().ok())?;
    let month = time::Month::try_from(month_number).ok()?;
    let date = time::Date::from_calendar_date(year, month, day).ok()?;
    let timestamp = date.with_time(time::Time::MIDNIGHT).assume_utc();
    let resolution = match parts.len() {
        1 => DateTimeResolution::Year,
        2 => DateTimeResolution::Month,
        3 => DateTimeResolution::Day,
        _ => unreachable!(),
    };
    Some(DateSelector::Span {
        timestamp,
        resolution,
        comparison,
    })
}

pub(in crate::services::render) fn parse_list_pages_comparison(
    value: &str,
) -> (ComparisonOperation, &str) {
    for (prefix, comparison) in [
        (">=", ComparisonOperation::GreaterOrEqualThan),
        ("<=", ComparisonOperation::LessOrEqualThan),
        ("!=", ComparisonOperation::NotEqual),
        (">", ComparisonOperation::GreaterThan),
        ("<", ComparisonOperation::LessThan),
        ("=", ComparisonOperation::Equal),
    ] {
        if let Some(value) = value.trim().strip_prefix(prefix) {
            return (comparison, value.trim());
        }
    }
    (ComparisonOperation::Equal, value.trim())
}

pub(in crate::services::render) fn subtract_wikidot_relative_time(
    timestamp: time::OffsetDateTime,
    amount: i64,
    unit: &str,
) -> Option<time::OffsetDateTime> {
    let unit = unit.trim_end_matches('s').to_ascii_lowercase();
    match unit.as_str() {
        "second" | "minute" | "hour" | "day" | "week" => {
            let seconds_per_unit = match unit.as_str() {
                "second" => 1,
                "minute" => 60,
                "hour" => 3_600,
                "day" => 86_400,
                "week" => 604_800,
                _ => unreachable!(),
            };
            let seconds = amount.checked_mul(seconds_per_unit)?;
            timestamp.checked_sub(time::Duration::seconds(seconds))
        }
        "month" | "year" => {
            let months = amount.checked_mul(if unit == "year" { 12 } else { 1 })?;
            let month_index = i64::from(timestamp.year())
                .checked_mul(12)?
                .checked_add(i64::from(u8::from(timestamp.month())) - 1)?
                .checked_sub(months)?;
            let year = i32::try_from(month_index.div_euclid(12)).ok()?;
            let month =
                time::Month::try_from((month_index.rem_euclid(12) + 1) as u8).ok()?;
            let day = timestamp.day().min(month.length(year));
            let date = time::Date::from_calendar_date(year, month, day).ok()?;
            Some(timestamp.replace_date(date))
        }
        _ => None,
    }
}

pub(in crate::services::render) fn is_dynamic_list_pages_value(value: &str) -> bool {
    value.eq_ignore_ascii_case("@url")
        || value
            .split_once('|')
            .is_some_and(|(selector, _)| selector.eq_ignore_ascii_case("@url"))
}

pub(in crate::services::render) fn list_pages_url_fallback(value: &str) -> Option<&str> {
    value.split_once('|').and_then(|(selector, fallback)| {
        selector.eq_ignore_ascii_case("@url").then_some(fallback)
    })
}

/// What an `@URL` selector resolves to once the request's URL is known.
pub(in crate::services::render) enum UrlSelector<'a> {
    /// The selector names no `@URL`, or names one whose fallback applies.
    Static(&'a str),

    /// The URL supplied a tag, which replaces the whole `@URL` selector.
    Resolved(String),

    /// `@URL` with nothing to resolve to and no fallback. Live drops the
    /// constraint rather than matching nothing, so the module falls back to
    /// whatever it would do without the selector. For `tags` that widens to
    /// the whole site; for `category` it means the default category, not
    /// every category. Dropping is not the same as matching everything.
    Dropped,
}

/// Resolve an `@URL` selector against the URL path argument of the same name.
///
/// A selector names the argument it reads: `tags="@URL"` reads `/tag/<value>`
/// and `category="@URL"` reads `/category/<value>`. An empty argument counts
/// as absent for both, which live confirms by rendering `/tag` and `/category`
/// identically to the bare page URL. PagesByTag draws that line differently,
/// which is why neither module reuses the other's rule.
pub(in crate::services::render) fn resolve_url_selector<'a>(
    value: &'a str,
    url_value: Option<&str>,
) -> UrlSelector<'a> {
    if !is_dynamic_list_pages_value(value) {
        return UrlSelector::Static(value);
    }
    match url_value {
        Some(resolved) if !resolved.is_empty() => {
            UrlSelector::Resolved(resolved.to_owned())
        }
        _ => match list_pages_url_fallback(value) {
            Some(fallback) => UrlSelector::Static(fallback),
            None => UrlSelector::Dropped,
        },
    }
}

pub(in crate::services::render) fn static_list_pages_selector<'a>(
    value: &'a str,
    unsupported_count_pages_filter: &mut bool,
) -> Option<&'a str> {
    if let Some(fallback) = list_pages_url_fallback(value) {
        Some(fallback)
    } else if is_dynamic_list_pages_value(value) {
        *unsupported_count_pages_filter = true;
        None
    } else {
        Some(value)
    }
}

pub(in crate::services::render) fn list_pages_has_unsupported_parent_selector(
    head: &str,
) -> bool {
    LISTPAGES_ARGUMENT_REGEX
        .captures_iter(head)
        .any(|captures| {
            if !captures["key"].eq_ignore_ascii_case("parent") {
                return false;
            }

            let value = captures
                .name("double")
                .or_else(|| captures.name("single"))
                .or_else(|| captures.name("bare"))
                .map(|matched| matched.as_str().trim())
                .unwrap_or_default();
            let value = list_pages_url_fallback(value).unwrap_or(value);
            !matches!(value, "." | "*" | "")
        })
}

pub(in crate::services::render) fn list_pages_has_unsupported_page_type_selector(
    head: &str,
) -> bool {
    LISTPAGES_ARGUMENT_REGEX
        .captures_iter(head)
        .any(|captures| {
            if !matches!(
                captures["key"].to_ascii_lowercase().as_str(),
                "pagetype" | "page_type" | "page-type"
            ) {
                return false;
            }

            let value = captures
                .name("double")
                .or_else(|| captures.name("single"))
                .or_else(|| captures.name("bare"))
                .map(|matched| matched.as_str().trim())
                .unwrap_or_default();
            let value = list_pages_url_fallback(value).unwrap_or(value);
            !matches!(
                value.to_ascii_lowercase().as_str(),
                "all" | "*" | "hidden" | "normal" | ""
            )
        })
}

pub(in crate::services::render) fn wikidot_list_pages_name_slug(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "-")
}

pub(in crate::services::render) fn split_list_pages_values(value: &str) -> Vec<String> {
    value
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(in crate::services::render) fn is_current_page_tag_selector(value: &str) -> bool {
    matches!(value.trim().trim_start_matches(['+', '-']), "=" | "==")
}

pub(in crate::services::render) fn is_no_tags_selector(value: &str) -> bool {
    value.trim() == "-"
}

pub(in crate::services::render) fn parse_list_pages_order(
    value: &str,
) -> Option<OrderBySelector> {
    let (value, ascending) = match value.split_once(char::is_whitespace) {
        Some((property, direction)) => {
            let ascending = match direction.trim().to_ascii_lowercase().as_str() {
                "asc" | "ascending" => true,
                "desc" | "descending" => false,
                _ => return None,
            };
            (property, ascending)
        }
        None => match value.strip_prefix('-') {
            Some(value) => (value, false),
            None => parse_wikidot_camel_case_order(value).unwrap_or((value, true)),
        },
    };

    let property = match value.to_ascii_lowercase().as_str() {
        "name" | "slug" => OrderProperty::PageSlug,
        "fullname" | "fullslug" | "full_slug" => OrderProperty::FullSlug,
        "title" => OrderProperty::Title,
        "alt_title" | "alttitle" => OrderProperty::AltTitle,
        "created_at" | "createdat" | "created" | "date" => OrderProperty::CreatedAt,
        "updated_at" | "updatedat" | "updated" => OrderProperty::UpdatedAt,
        "size" => OrderProperty::Size,
        "rating" | "score" => OrderProperty::Score,
        "random" => OrderProperty::Random,
        _ => return None,
    };

    Some(OrderBySelector {
        property,
        ascending,
    })
}

pub(in crate::services::render) fn parse_wikidot_camel_case_order(
    value: &str,
) -> Option<(&str, bool)> {
    let lower = value.to_ascii_lowercase();
    for (suffix, ascending) in [
        ("ascending", true),
        ("descending", false),
        ("asc", true),
        ("desc", false),
    ] {
        if lower.ends_with(suffix) && value.len() > suffix.len() {
            return Some((&value[..value.len() - suffix.len()], ascending));
        }
    }

    None
}

pub(in crate::services::render) fn parse_list_pages_page_type(
    value: &str,
) -> Option<PageTypeSelector> {
    match value.to_ascii_lowercase().as_str() {
        "all" | "*" => Some(PageTypeSelector::All),
        "hidden" => Some(PageTypeSelector::Hidden),
        "normal" | "" => Some(PageTypeSelector::Normal),
        _ => None,
    }
}

#[cfg(test)]
pub(in crate::services::render) fn list_pages_body_variables_supported(
    body: &str,
) -> bool {
    ListPagesTemplatePlan::compile(body).is_some()
}

pub(in crate::services::render) fn unsupported_list_pages_replacement(
    module_source: &str,
    body: &str,
) -> String {
    if list_pages_body_has_numbered_rows(body)
        || list_pages_body_is_no_visible_tracking_markup(body)
    {
        "[[div class=\"list-pages-box\"]][[/div]]".to_owned()
    } else {
        module_source.to_owned()
    }
}

pub(in crate::services::render) fn list_pages_body_has_numbered_rows(body: &str) -> bool {
    body.lines()
        .any(|line| native_numbered_list_content(line).is_some())
}

pub(in crate::services::render) fn list_pages_body_is_no_visible_tracking_markup(
    body: &str,
) -> bool {
    let mut saw_tracking_markup = false;

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let lower = line.to_ascii_lowercase();
        let allowed = lower.starts_with("[[image ")
            || lower.starts_with("[[embed]]")
            || lower.starts_with("[[/embed]]")
            || lower.starts_with("<iframe ") && lower.contains("display: none")
            || lower.starts_with("[[module listusers ")
            || lower.starts_with("[[/module]]")
            || lower.starts_with("[[%%content{0}%%module listusers ")
            || lower.starts_with("[[%%content{0}%%/module]]");
        if !allowed {
            return false;
        }
        saw_tracking_markup = true;
    }

    saw_tracking_markup
}

#[cfg(test)]
pub(in crate::services::render) fn list_pages_body_uses_content_variable(
    body: &str,
) -> bool {
    ListPagesTemplatePlan::compile(body).is_some_and(|plan| plan.uses_content())
}

pub(in crate::services::render) fn substitute_list_pages_rating_only(
    template: &str,
    page: &FoundPageRow,
) -> String {
    let rating = format_list_pages_rating(page.score);
    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            if captures["name"].eq_ignore_ascii_case("rating") {
                rating.clone()
            } else {
                captures[0].to_owned()
            }
        })
        .into_owned();
    RenderService::resolve_wikidot_parser_functions(&substituted)
}

pub(in crate::services::render) fn push_list_pages_pager(
    output: &mut String,
    page_info: &PageInfo<'_>,
    offset: u32,
    per_page: u64,
    total_selected: usize,
) {
    let per_page = per_page
        .min(MAX_LISTPAGES_RENDER_LIMIT)
        .min(usize::MAX as u64) as usize;
    if per_page == 0 || total_selected <= per_page {
        return;
    }

    let page_count = total_selected.div_ceil(per_page);
    let current_page = (offset as usize / per_page).saturating_add(1);
    if current_page > page_count {
        return;
    }

    output.push_str("[[div class=\"pager\"]]\n");
    output.push_str(&format!(
        r#"[[span class="pager-no"]]page {current_page} of {page_count}[[/span]]"#
    ));

    let mut pages = BTreeSet::from([1, current_page, page_count]);
    if current_page > 1 {
        pages.insert(current_page - 1);
    }
    if current_page < page_count {
        pages.insert(current_page + 1);
    }
    if current_page <= 2 && page_count >= 3 {
        pages.insert(3);
    }
    if current_page + 1 >= page_count && page_count > 2 {
        pages.insert(page_count - 2);
    }
    if page_count > 1 {
        pages.insert(page_count - 1);
    }

    let mut previous = 0;
    for page in pages {
        if previous != 0 && page > previous + 1 {
            output.push_str(r#"[[span class="dots"]]...[[/span]]"#);
        }
        if page == current_page {
            output.push_str(&format!(r#"[[span class="current"]]{page}[[/span]]"#));
        } else {
            push_list_pages_pager_target(output, page_info, page, &page.to_string());
        }
        previous = page;
    }

    if current_page < page_count {
        push_list_pages_pager_target(output, page_info, current_page + 1, "next »");
    }

    output.push_str("\n[[/div]]\n");
}

pub(in crate::services::render) fn push_list_pages_pager_target(
    output: &mut String,
    page_info: &PageInfo<'_>,
    target_page: usize,
    label: &str,
) {
    output.push_str(r#"[[span class="target"]][[[/"#);
    output.push_str(&percent_encode_path_segment(page_info.page.as_ref()));
    output.push_str("/p/");
    output.push_str(&target_page.to_string());
    output.push('|');
    output.push_str(label);
    output.push_str("]]][[/span]]");
}

pub(in crate::services::render) struct ListPagesSubstitutionContext<'a> {
    pub(in crate::services::render) rendered_limit: usize,
    pub(in crate::services::render) ajax_module_response: bool,
    pub(in crate::services::render) site: &'a str,
    pub(in crate::services::render) category: &'a str,
    pub(in crate::services::render) user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
    pub(in crate::services::render) snapshot_displays:
        &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    pub(in crate::services::render) page_wikitext: Option<&'a str>,
    pub(in crate::services::render) page_wikitext_scalar_count: Option<usize>,
    pub(in crate::services::render) page_parent_fullname: Option<&'a str>,
    pub(in crate::services::render) page_child_count: Option<u64>,
    pub(in crate::services::render) page_revision_count: Option<u64>,
    pub(in crate::services::render) expanded_content:
        Option<&'a BTreeMap<Option<usize>, String>>,
    pub(in crate::services::render) data_form_values: &'a BTreeMap<String, String>,
    pub(in crate::services::render) render_generated_html: bool,
}

pub(in crate::services::render) fn substitute_list_pages_variables_with_fragments(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    context: &ListPagesSubstitutionContext<'_>,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    let slug = page.slug.as_deref().unwrap_or("");
    // Page-query rows already retain Wikidot's normalized full slug, including
    // a non-default category prefix. Reconstructing it would duplicate that prefix.
    let full_slug = slug.to_owned();
    let link = format!(
        "http://{}.wikidot.com/{full_slug}/noredirect/true",
        context.site,
    );
    let title = page.title.as_deref().unwrap_or(slug);
    let generated_wikitext_title = preserve_list_pages_generated_text_typography(title);
    let title_linked = if slug.is_empty() {
        generated_wikitext_title.clone()
    } else {
        format!("[/{slug} {generated_wikitext_title}]")
    };
    let snapshot = context.snapshot_displays.get(&page.page_id);
    let created_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.created_by_name.as_deref());
    let updated_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.updated_by_name.as_deref());
    let commented_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.commented_by_name.as_deref());
    let created_by = created_by_snapshot
        .map(str::to_owned)
        .or_else(|| {
            page.created_by.map(|user_id| {
                context
                    .user_displays
                    .get(&user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| user_id.to_string())
            })
        })
        .unwrap_or_default();
    let created_by_unix = list_pages_created_by_unix(
        page,
        context.user_displays,
        context.snapshot_displays,
    );
    let created_by_linked = created_by_snapshot
        .map(render_list_pages_snapshot_user)
        .or_else(|| {
            page.created_by.map(|user_id| {
                render_list_pages_wikidot_user(
                    user_id,
                    context.user_displays.get(&user_id),
                )
            })
        })
        .unwrap_or_default();
    let updated_by = updated_by_snapshot
        .map(str::to_owned)
        .or_else(|| {
            page.updated_by.map(|user_id| {
                context
                    .user_displays
                    .get(&user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| user_id.to_string())
            })
        })
        .unwrap_or_default();
    let updated_by_linked = updated_by_snapshot
        .map(render_list_pages_snapshot_user)
        .or_else(|| {
            page.updated_by.map(|user_id| {
                render_list_pages_wikidot_user(
                    user_id,
                    context.user_displays.get(&user_id),
                )
            })
        })
        .unwrap_or_default();
    let commented_by = commented_by_snapshot.map(str::to_owned).unwrap_or_default();
    let created_at = snapshot
        .map(|snapshot| snapshot.created_at)
        .or(page.created_at);
    let updated_at = snapshot
        .map(|snapshot| snapshot.updated_at)
        .or(page.updated_at);
    let commented_at = snapshot.and_then(|snapshot| snapshot.commented_at);
    let comments = snapshot
        .map(|snapshot| snapshot.comments.to_string())
        .unwrap_or_else(|| {
            if context.ajax_module_response {
                "0".to_owned()
            } else {
                String::new()
            }
        });
    let tags = page.tags.as_deref().unwrap_or(&[]);
    let visible_tags = tags
        .iter()
        .filter(|tag| is_list_pages_visible_tag(tag))
        .cloned()
        .collect::<Vec<_>>();
    let hidden_tags = tags
        .iter()
        .filter(|tag| is_list_pages_hidden_tag(tag))
        .cloned()
        .collect::<Vec<_>>();
    let tags_text = visible_tags.join(" ");
    let rating = format_list_pages_rating(page.score);
    // The frozen corpus predates vote-count capture. Keep this value typed as
    // optional provenance and select the component's explicit zero-vote state
    // when it is absent; inventing a count from the net rating would create a
    // visibly plausible but false upvote/downvote ratio.
    let rating_votes = snapshot
        .and_then(|snapshot| snapshot.rating_votes)
        .unwrap_or(0)
        .to_string();
    let index = index.to_string();
    let total = total.to_string();
    let rendered_limit = context.rendered_limit.to_string();

    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "title_linked" => title_linked.clone(),
                "linked_title" => title_linked.clone(),
                "title" => generated_wikitext_title.clone(),
                "name" | "slug" | "page_unix_name" => slug.to_owned(),
                "fullname" | "full_slug"
                    if list_pages_variable_starts_triple_link_target(
                        template,
                        captures
                            .get(0)
                            .expect("ListPages variable capture exists")
                            .start(),
                    ) =>
                {
                    format!("/{full_slug}")
                }
                "fullname" | "full_slug" => full_slug.clone(),
                "link" if !slug.is_empty() && !context.site.is_empty() => link.clone(),
                "link" => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
                "created_by" | "createdby" => created_by.clone(),
                "created_by_linked" | "createdbylinked" | "author" => {
                    protect_list_pages_generated_html(
                        created_by_linked.clone(),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "created_by_unix" => created_by_unix
                    .clone()
                    .unwrap_or_else(|| captures[0].to_owned()),
                "created_at" | "createdat" | "date" => protect_list_pages_generated_html(
                    format_list_pages_created_at(
                        created_at,
                        captures.name("format").map(|matched| matched.as_str()),
                        context.render_generated_html,
                    ),
                    context.render_generated_html,
                    compat_html,
                ),
                "updated_by" | "updatedby" => updated_by.clone(),
                "updated_by_linked" | "updatedbylinked" => {
                    protect_list_pages_generated_html(
                        updated_by_linked.clone(),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "updated_at" | "updatedat" | "date_edited" => {
                    protect_list_pages_generated_html(
                        format_list_pages_created_at(
                            updated_at,
                            captures.name("format").map(|matched| matched.as_str()),
                            context.render_generated_html,
                        ),
                        context.render_generated_html,
                        compat_html,
                    )
                }
                "commented_by"
                | "commentedby"
                | "commented_by_linked"
                | "commentedbylinked" => commented_by.clone(),
                "commented_at" | "commentedat" => protect_list_pages_generated_html(
                    format_list_pages_created_at(
                        commented_at,
                        captures.name("format").map(|matched| matched.as_str()),
                        context.render_generated_html,
                    ),
                    context.render_generated_html,
                    compat_html,
                ),
                "rating" => rating.clone(),
                "rating_votes" | "ratingvotes" => rating_votes.clone(),
                "comments" => comments.clone(),
                "tags" => tags_text.clone(),
                "tags_linked" | "tagslinked" => render_list_pages_tags(
                    &visible_tags,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                    compat_html,
                ),
                "_tags_linked" => render_list_pages_tags(
                    &hidden_tags,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                    compat_html,
                ),
                "_tags" => tags.join(" "),
                "category" => context.category.to_owned(),
                "size" => context
                    .page_wikitext_scalar_count
                    .map(|scalar_count| scalar_count.to_string())
                    .unwrap_or_else(|| captures[0].to_owned()),
                "children" => context
                    .page_child_count
                    .map(|child_count| child_count.to_string())
                    .unwrap_or_else(|| captures[0].to_owned()),
                "revisions" => context
                    .page_revision_count
                    .map(|revision_count| revision_count.to_string())
                    .unwrap_or_else(|| captures[0].to_owned()),
                "site_domain" if !context.site.is_empty() => {
                    format!("{}.wikidot.com", context.site)
                }
                "site_domain" => captures[0].to_owned(),
                "parent_fullname" => {
                    context.page_parent_fullname.unwrap_or("").to_owned()
                }
                // Live Wikidot leaves this variable unsubstituted on a
                // plus/minus site, so the authored text survives rather than
                // collapsing to an empty cell.
                "rating_percent" => captures[0].to_owned(),
                "form_data" | "form_raw" => captures
                    .name("argument")
                    .and_then(|matched| context.data_form_values.get(matched.as_str()))
                    .cloned()
                    .unwrap_or_default(),
                "content" => {
                    let section = captures
                        .name("argument")
                        .and_then(|matched| matched.as_str().parse().ok());
                    context
                        .expanded_content
                        .and_then(|content| content.get(&section).cloned())
                        .or_else(|| {
                            context.page_wikitext.map(|wikitext| {
                                wikidot_content_section(wikitext, section)
                            })
                        })
                        .unwrap_or_default()
                }
                "index" => index.clone(),
                "total" => total.clone(),
                "limit" => rendered_limit.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
            }
        })
        .into_owned();

    RenderService::resolve_wikidot_parser_functions(&substituted)
}

fn list_pages_variable_starts_triple_link_target(template: &str, start: usize) -> bool {
    template[..start]
        .rfind("[[[")
        .is_some_and(|opening| template[opening + 3..start].trim().is_empty())
}

#[cfg(test)]
pub(in crate::services::render) fn substitute_list_pages_variables(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    context: &ListPagesSubstitutionContext<'_>,
) -> String {
    let mut compat_html = CompatHtmlFragments::new(template);
    let protected = substitute_list_pages_variables_with_fragments(
        template,
        page,
        index,
        total,
        context,
        &mut compat_html,
    );
    compat_html.restore(&protected)
}

pub(in crate::services::render) fn substitute_count_pages_variables(
    template: &str,
    total: usize,
) -> String {
    let total = total.to_string();
    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "total" | "count" => total.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
            }
        })
        .into_owned();
    let mut substituted = RenderService::resolve_wikidot_parser_functions(&substituted);
    neutralize_authored_markers(&mut substituted);
    substituted
}

pub(in crate::services::render) fn render_list_pages_tags(
    tags: &[String],
    path_prefix: Option<&str>,
    render_as_html: bool,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    let path_prefix = path_prefix
        .filter(|prefix| !prefix.trim().is_empty())
        .unwrap_or("/system:page-tags/tag/");
    tags.iter()
        .map(|tag| {
            let href = list_pages_tag_link_href(path_prefix, tag);
            let label = compat_html.push_plain(tag);
            if render_as_html {
                format!(
                    r#"<a href="{href}">{label}</a>"#,
                    href = escape_list_pages_html_attr(&href),
                )
            } else {
                format!("[{href} {label}]")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(in crate::services::render) fn list_pages_tag_link_href(
    path_prefix: &str,
    tag: &str,
) -> String {
    let path_prefix = percent_encode_list_pages_href_prefix(path_prefix.trim());
    let tag = percent_encode_list_pages_path_segment(tag.trim());
    if path_prefix.starts_with("http://")
        || path_prefix.starts_with("https://")
        || path_prefix.starts_with('/')
    {
        format!("{path_prefix}{tag}")
    } else {
        format!("/{path_prefix}{tag}")
    }
}

pub(in crate::services::render) fn percent_encode_list_pages_href_prefix(
    value: &str,
) -> String {
    percent_encode_list_pages_href_bytes(value, |byte| {
        matches!(
            byte,
            b':' | b'/' | b'?' | b'&' | b'=' | b',' | b'@' | b'%' | b'+' | b';'
        )
    })
}

pub(in crate::services::render) fn percent_encode_list_pages_path_segment(
    value: &str,
) -> String {
    percent_encode_list_pages_href_bytes(value, |_| false)
}

pub(in crate::services::render) fn percent_encode_list_pages_href_bytes(
    value: &str,
    preserve_reserved: impl Fn(u8) -> bool,
) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ if preserve_reserved(byte) => encoded.push(byte as char),
            _ => {
                use std::fmt::Write as _;
                write!(&mut encoded, "%{byte:02X}")
                    .expect("writing to a String cannot fail");
            }
        }
    }
    encoded
}

pub(in crate::services::render) fn render_tag_cloud_box(
    tags: &[(String, usize)],
) -> String {
    let max_count = tags.iter().map(|(_, count)| *count).max().unwrap_or(1);
    let mut output = String::from("[[div class=\"pages-tag-cloud-box\"]]\n");

    for (tag, count) in tags {
        let weight = if max_count <= 1 {
            1.0
        } else {
            0.5 + ((*count as f32 / max_count as f32) * 2.5)
        };
        let tag_path =
            format!("/system:page-tags/tag/{}", escape_list_pages_html_attr(tag));
        output.push_str(&format!(
            r#"[[span class="tag" style="font-size: {weight:.2}em;"]][{tag_path} {tag_text}][[/span]] "#,
            tag_text = escape_list_pages_html_text(tag),
        ));
    }

    output.push_str("\n[[/div]]");
    output
}

pub(in crate::services::render) fn is_tag_cloud_visible_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty()
        && !tag.starts_with('_')
        && !tag.starts_with("codex-")
        && !tag.starts_with("branch-")
        && !tag.starts_with("feature-")
        && !matches!(
            tag,
            "declared-universe"
                | "declared-universe-include-support"
                | "verification"
                | "preview"
                | "ui-authoring"
                | "edited"
                | "fragment"
        )
}

pub(in crate::services::render) fn is_list_pages_visible_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty() && !tag.starts_with('_')
}

pub(in crate::services::render) fn is_list_pages_hidden_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty() && tag.starts_with('_')
}

pub(in crate::services::render) fn render_list_pages_wikidot_user(
    user_id: i64,
    user: Option<&WikidotUserDisplay>,
) -> String {
    let Some(user) = user else {
        return user_id.to_string();
    };
    if !user.wikidot_profile {
        return escape_list_pages_html_text(&user.name);
    }
    let slug = user.slug.as_deref().unwrap_or(&user.name);
    format!(
        concat!(
            r#"<span class="printuser avatarhover" data-wikijump-compat-listpages-user="1">"#,
            r#"<a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
            r#"<img alt="{name}" class="small" src="http://www.wikidot.com/avatar.php?userid={user_id}&amp;size=small"/>"#,
            r#"</a><a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">{name}</a>"#,
            r#"</span>"#
        ),
        slug = escape_list_pages_html_attr(slug),
        user_id = user.user_id,
        name = escape_list_pages_html_text(&user.name),
    )
}

pub(in crate::services::render) fn render_list_pages_snapshot_user(name: &str) -> String {
    escape_list_pages_html_text(name)
}

pub(in crate::services::render) fn list_pages_revision_count(
    page: &FoundPageRow,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
    revision_counts: &BTreeMap<i64, u64>,
) -> Option<u64> {
    match snapshot_displays.get(&page.page_id) {
        Some(snapshot) => u64::try_from(snapshot.source_revision_count).ok(),
        None => revision_counts.get(&page.page_id).copied(),
    }
}

pub(in crate::services::render) fn list_pages_parent_fullname<'a>(
    page: &FoundPageRow,
    snapshot_displays: &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    relational_parent_fullnames: &'a BTreeMap<i64, String>,
) -> Option<&'a str> {
    let parent_fullname = match snapshot_displays.get(&page.page_id) {
        Some(snapshot) => snapshot.parent_fullname.as_deref()?,
        None => relational_parent_fullnames.get(&page.page_id)?.as_str(),
    };
    (!parent_fullname.is_empty()).then_some(parent_fullname)
}

pub(in crate::services::render) fn list_pages_created_by_unix(
    page: &FoundPageRow,
    user_displays: &BTreeMap<i64, WikidotUserDisplay>,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
) -> Option<String> {
    if snapshot_displays
        .get(&page.page_id)
        .and_then(|snapshot| snapshot.created_by_name.as_deref())
        .is_some_and(|created_by_name| !created_by_name.is_empty())
    {
        return None;
    }
    let user = user_displays.get(&page.created_by?)?;
    let slug = user.slug.as_deref()?;
    if slug.is_empty() {
        return None;
    }
    Some(slug.to_owned())
}

pub(in crate::services::render) fn preserve_list_pages_generated_text_typography(
    value: &str,
) -> String {
    if !value.contains("...") {
        return value.to_owned();
    }
    let marker = list_pages_literal_ellipsis_marker();
    value.replace("...", &marker)
}

pub(in crate::services::render) fn list_pages_literal_ellipsis_marker() -> String {
    format!(
        "{WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

pub(in crate::services::render) fn restore_list_pages_literal_ellipsis_markers(
    html: &str,
) -> String {
    WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX
        .replace_all(html, "...")
        .into_owned()
}

pub(in crate::services::render) fn format_list_pages_created_at(
    created_at: Option<time::OffsetDateTime>,
    format: Option<&str>,
    render_as_html: bool,
) -> String {
    let Some(created_at) = created_at else {
        return String::new();
    };
    let created_at = created_at
        .to_offset(time::UtcOffset::from_hms(9, 0, 0).expect("valid JST offset"));
    let format = format.unwrap_or("%e %b %Y, %H:%M");
    let display_format = format.split('|').next().unwrap_or(format);
    let text = format_wikidot_list_pages_date(created_at, display_format);
    let encoded_format = percent_encode_path_segment(format);
    if render_as_html {
        format!(
            r#"<span class="odate time_{} format_{}" style="cursor: help; display: inline;">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    } else {
        format!(
            r#"<span class="odate time_{} format_{}" data-wikijump-compat-date="1" style="cursor: help; display: inline;">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    }
}

fn protect_list_pages_generated_html(
    html: String,
    rendered_inside_generated_html: bool,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    if html.is_empty() || rendered_inside_generated_html {
        html
    } else {
        compat_html.push_html(html)
    }
}
