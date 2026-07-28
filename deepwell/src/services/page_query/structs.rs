/*
 * services/page_query/structs.rs
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

// TODO: add serde, include time fmt conversions
#![allow(dead_code)] // TEMP

use crate::services::score::ScoreValue;
use crate::types::Reference;
use sea_orm::prelude::TimeDateTimeWithTimeZone;
use std::borrow::Cow;

pub(crate) const MAX_PAGE_QUERY_SCORE_SELECTORS: usize = 64;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// What kinds of pages (hidden or not) to select from.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PageTypeSelector {
    All,
    Normal,
    Hidden,
}

pub type CategoryList<'a> = &'a [Cow<'a, str>];
pub type TagList<'a> = &'a [Cow<'a, str>];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IncludedCategories<'a> {
    All,
    List(CategoryList<'a>),
}

/// Which categories to select from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoriesSelector<'a> {
    pub included_categories: IncludedCategories<'a>,
    pub excluded_categories: CategoryList<'a>,
}

/// What tag conditions to maintain during the search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCondition<'a> {
    /// Represents an OR operator for the tags; page may contain any of these tags.
    pub any_present: TagList<'a>,

    /// Represents the AND operator for the tags; page must contain all of these tags.
    pub all_present: TagList<'a>,

    /// Represents the NOT operator for the tags; page must *not* contain any of these tags.
    pub none_present: TagList<'a>,

    /// Wikidot's `tags="-"` selector; page must carry no tags at all.
    pub untagged: bool,
}

/// Selects pages by their creation author without overloading an empty list.
///
/// Local Wikijump pages have a stable user ID on their earliest revision. Imported Wikidot snapshots can instead have only the source author's display name, so a query may match either representation. `Any` combines the two representations with OR semantics. An empty `Any` is treated like `None`, never like `All`.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum AuthorSelector<'a> {
    #[default]
    All,
    Any {
        user_ids: &'a [i64],
        wikidot_snapshot_names: &'a [Cow<'a, str>],
    },
    /// Wikidot's `created_by="-="`; pages this author did not create.
    ///
    /// Both representations are excluded together, so a page matching either
    /// the local creator ID or the imported snapshot name is left out.
    NotAny {
        user_ids: &'a [i64],
        wikidot_snapshot_names: &'a [Cow<'a, str>],
    },
    None,
}

pub fn normalize_wikidot_author_name(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

/// The relationship of the pages being queried to their parent/child pages.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PageParentSelector<'a> {
    /// Pages without parent-based filtering.
    All,

    /// Pages which have no parent page.
    NoParent,

    /// Pages which share any parent page(s) with the page making the query.
    SameParents,

    /// Pages which do *not* share any parent page(s) with the page making the query.
    DifferentParents,

    /// Pages which are children of the page making the query.
    ChildOf,

    /// Pages which have specified parent pages.
    HasParents(&'a [Reference<'a>]),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ComparisonOperation {
    GreaterThan,
    LessThan,
    GreaterOrEqualThan,
    LessOrEqualThan,
    Equal,
    NotEqual,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DateTimeResolution {
    Second,
    Minute,
    Hour,
    Day,
    Month,
    Year,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DateSelector {
    /// A time span represented by a timestamp, the "resolution" of the time, and a comparison operator.
    Span {
        timestamp: OffsetDateTime,
        resolution: DateTimeResolution,
        comparison: ComparisonOperation,
    },

    /// A time span represented by a timestamp, from present to the time specified.
    FromPresent { start: OffsetDateTime },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScoreSelector {
    pub score: ScoreValue,
    pub comparison: ComparisonOperation,
}

/// Range of pages to display, relative to the current page.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RangeSelector {
    /// Display only the current page.
    Current,

    /// Display pages before the current page in queried results.
    Before,

    /// Display pages after the current page in queried results.
    After,

    /// Display all pages besides the current page.
    Others,
}

/// Selects all pages that have a data form with matching field-value pairs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFormSelector<'a> {
    pub field: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub negated: bool,
}

pub fn parse_static_wikidot_data_form_values(wikitext: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let mut started = false;

    for line in wikitext.lines() {
        let Some((field, value)) = line.split_once(':') else {
            if started || !line.trim().is_empty() {
                break;
            }
            continue;
        };
        let field = field.trim();
        if field.is_empty()
            || !field.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
            })
        {
            if started || !line.trim().is_empty() {
                break;
            }
            continue;
        }

        started = true;
        let value = unquote_static_wikidot_data_form_value(value.trim());
        values.insert(field.to_owned(), value.to_owned());
    }

    values
}

pub fn static_wikidot_data_form_matches(
    values: &BTreeMap<String, String>,
    selectors: &[DataFormSelector<'_>],
) -> bool {
    selectors.iter().all(|selector| {
        let Some(actual) = values.get(selector.field.as_ref()).map(String::as_str) else {
            return false;
        };
        let matches = actual == selector.value.as_ref();
        matches != selector.negated
    })
}

fn unquote_static_wikidot_data_form_value(value: &str) -> &str {
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if matches!((first, last), (b'\'', b'\'') | (b'"', b'"')) {
            return &value[1..value.len() - 1];
        }
    }

    value
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrderProperty {
    PageSlug,
    FullSlug,
    Title,
    AltTitle,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    Size,
    Score,
    Votes,
    Revisions,
    Comments,
    Random,
    DataFormFieldName {
        field: Cow<'static, str>,
        numeric: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBySelector {
    pub property: OrderProperty,
    pub ascending: bool,
}

impl Default for OrderBySelector {
    fn default() -> Self {
        OrderBySelector {
            property: OrderProperty::CreatedAt,
            ascending: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PaginationSelector {
    pub limit: Option<u64>,
    pub per_page: u8,
    pub reversed: bool,
}

impl Default for PaginationSelector {
    fn default() -> PaginationSelector {
        PaginationSelector {
            limit: None,
            per_page: 20,
            reversed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageQueryVariables<'a> {
    CreatedAt,
    CreatedBy,
    CreatedBySlug,
    CreatedById,
    CreatedByLinked,
    UpdatedAt,
    UpdatedBy,
    UpdatedBySlug,
    UpdatedById,
    UpdatedByLinked,
    CommentedAt,
    CommentedBy,
    CommentedBySlug,
    CommentedById,
    CommentedByLinked,
    PageSlug,
    Category,
    FullSlug,
    Title,
    TitleLinked,
    ParentNamed,
    ParentCategory,
    ParentSlug,
    ParentTitle,
    ParentTitleLinked,
    Link,
    Content,
    ContentN(u64),
    Preview,
    PreviewN(u64),
    Summary,
    FirstParagraph,
    Tags,
    TagsLinked,
    TagsLinkedURL(Cow<'a, str>),
    HiddenTags,
    HiddenTagsLinked,
    HiddenTagsLinkedURL(Cow<'a, str>),
    FormData(Cow<'a, str>),
    FormRaw(Cow<'a, str>),
    FormLabel(Cow<'a, str>),
    FormHint(Cow<'a, str>),
    Children,
    Comments,
    Size,
    Score,
    ScoreVotes,
    ScorePercent,
    Revisions,
    Index,
    Total,
    Limit,
    TotalOrLimit,
    SiteTitle,
    SiteName,
    SiteDomain,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuery<'a> {
    pub current_page_id: i64,
    pub current_site_id: i64,
    pub queried_site_id: Option<i64>,
    pub page_type: PageTypeSelector,
    pub categories: CategoriesSelector<'a>,
    pub tags: TagCondition<'a>,
    pub page_parent: PageParentSelector<'a>,
    pub contains_outgoing_links: &'a [Reference<'a>],
    pub creation_date: DateSelector,
    pub update_date: DateSelector,
    pub author: AuthorSelector<'a>,
    pub score: &'a [ScoreSelector], // 5-star rating selector
    pub votes: &'a [ScoreSelector], // upvote/downvote rating selector
    pub offset: u32,
    pub range: RangeSelector,
    pub name: Option<Cow<'a, str>>,
    pub slug: Option<Cow<'a, str>>,
    pub slugs: &'a [Cow<'a, str>],
    pub data_form_fields: &'a [DataFormSelector<'a>],
    pub order: Option<OrderBySelector>,
    pub candidate_limit: Option<u64>,
    pub pagination: PaginationSelector,
    pub variables: &'a [PageQueryVariables<'a>],
    pub fields: FoundPageFields,
}

/// Specifies which optional fields to include in the query results.
///
/// Fields required for filtering or ordering are always fetched
/// internally, but only appear in the output if requested here.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct FoundPageFields {
    pub title: bool,
    pub alt_title: bool,
    pub slug: bool,
    pub page_category_id: bool,
    pub page_revision_id: bool,
    pub tags: bool,
    pub created_at: bool,
    pub created_by: bool,
    pub updated_at: bool,
    pub updated_by: bool,
    pub score: bool,
}

/// A single page row in the query results.
///
/// Fields are optional because callers specify which fields
/// they need via `FoundPageFields`. Fields not requested will
/// be `None` to avoid unnecessary data transfer.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FoundPageRow {
    pub page_id: i64,
    pub site_id: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_category_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_revision_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub created_at: Option<TimeDateTimeWithTimeZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub updated_at: Option<TimeDateTimeWithTimeZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

/// The result of `PageQueryService::find()`.
///
/// Contains an ordered list of pages matching the query,
/// with only the requested fields populated.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FoundPages {
    pub pages: Vec<FoundPageRow>,
}

impl FoundPages {
    #[inline]
    pub fn total(&self) -> usize {
        self.pages.len()
    }
}

/// Metadata describing how a PageQuery result was produced.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct PageQueryResultMetadata {
    pub candidate_count: Option<usize>,
    pub cap_exceeded: bool,
    pub sql_limit_offset_applied: bool,
    pub filtering_deferred_to_rust: bool,
    pub ordering_deferred_to_rust: bool,
    pub exact_count_safe: bool,
    pub unsupported_reason: Option<String>,
}

/// PageQuery result plus metadata for later ListPages and CountPages decisions.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PageQueryResultEnvelope {
    pub pages: FoundPages,
    pub metadata: PageQueryResultMetadata,
}

impl PageQueryResultEnvelope {
    pub fn sql_limited(pages: FoundPages, candidate_count: usize) -> Self {
        Self {
            pages,
            metadata: PageQueryResultMetadata {
                candidate_count: Some(candidate_count),
                sql_limit_offset_applied: true,
                exact_count_safe: true,
                ..PageQueryResultMetadata::default()
            },
        }
    }

    pub fn deferred_filter(pages: FoundPages, candidate_count: Option<usize>) -> Self {
        Self {
            pages,
            metadata: PageQueryResultMetadata {
                candidate_count,
                filtering_deferred_to_rust: true,
                exact_count_safe: false,
                ..PageQueryResultMetadata::default()
            },
        }
    }

    pub fn deferred(
        pages: FoundPages,
        candidate_count: Option<usize>,
        filtering_deferred_to_rust: bool,
        ordering_deferred_to_rust: bool,
        cap_exceeded: bool,
    ) -> Self {
        Self {
            pages,
            metadata: PageQueryResultMetadata {
                candidate_count,
                filtering_deferred_to_rust,
                ordering_deferred_to_rust,
                cap_exceeded,
                exact_count_safe: false,
                ..PageQueryResultMetadata::default()
            },
        }
    }

    pub fn cap_exceeded(pages: FoundPages, candidate_count: usize) -> Self {
        Self {
            pages,
            metadata: PageQueryResultMetadata {
                candidate_count: Some(candidate_count),
                cap_exceeded: true,
                exact_count_safe: false,
                ..PageQueryResultMetadata::default()
            },
        }
    }

    pub fn unsupported(pages: FoundPages, reason: impl Into<String>) -> Self {
        Self {
            pages,
            metadata: PageQueryResultMetadata {
                exact_count_safe: false,
                unsupported_reason: Some(reason.into()),
                ..PageQueryResultMetadata::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_form_parser_ignores_leading_blanks_and_stops_after_body_text() {
        let values = parse_static_wikidot_data_form_values(
            "\n field-one : 'alpha'\nfield-two: \"beta\"\n\nbody text\nfield-three: gamma",
        );

        assert_eq!(values.get("field-one").map(String::as_str), Some("alpha"));
        assert_eq!(values.get("field-two").map(String::as_str), Some("beta"));
        assert!(!values.contains_key("field-three"));
    }

    #[test]
    fn static_data_form_matching_honors_negated_selectors() {
        let values = parse_static_wikidot_data_form_values("status: open\nkind: tale");

        assert!(static_wikidot_data_form_matches(
            &values,
            &[
                DataFormSelector {
                    field: Cow::Borrowed("status"),
                    value: Cow::Borrowed("open"),
                    negated: false,
                },
                DataFormSelector {
                    field: Cow::Borrowed("kind"),
                    value: Cow::Borrowed("scp"),
                    negated: true,
                },
            ],
        ));

        assert!(!static_wikidot_data_form_matches(
            &values,
            &[DataFormSelector {
                field: Cow::Borrowed("missing"),
                value: Cow::Borrowed("open"),
                negated: true,
            }],
        ));
    }

    #[test]
    fn default_order_and_pagination_match_list_pages_defaults() {
        assert_eq!(
            OrderBySelector::default(),
            OrderBySelector {
                property: OrderProperty::CreatedAt,
                ascending: false,
            },
        );
        assert_eq!(
            PaginationSelector::default(),
            PaginationSelector {
                limit: None,
                per_page: 20,
                reversed: false,
            },
        );
    }

    #[test]
    fn found_pages_total_is_page_count() {
        assert_eq!(FoundPages { pages: Vec::new() }.total(), 0);
    }

    #[test]
    fn page_query_result_envelope_describes_plain_sql_limited_results() {
        let result =
            PageQueryResultEnvelope::sql_limited(FoundPages { pages: Vec::new() }, 25);

        assert_eq!(result.pages.total(), 0);
        assert_eq!(result.metadata.candidate_count, Some(25));
        assert!(result.metadata.sql_limit_offset_applied);
        assert!(!result.metadata.cap_exceeded);
        assert!(!result.metadata.filtering_deferred_to_rust);
        assert!(!result.metadata.ordering_deferred_to_rust);
        assert!(result.metadata.exact_count_safe);
        assert_eq!(result.metadata.unsupported_reason, None);
    }

    #[test]
    fn page_query_result_envelope_describes_deferred_filter_results() {
        let result = PageQueryResultEnvelope::deferred_filter(
            FoundPages { pages: Vec::new() },
            Some(100),
        );

        assert_eq!(result.metadata.candidate_count, Some(100));
        assert!(!result.metadata.sql_limit_offset_applied);
        assert!(result.metadata.filtering_deferred_to_rust);
        assert!(!result.metadata.ordering_deferred_to_rust);
        assert!(!result.metadata.exact_count_safe);
        assert_eq!(result.metadata.unsupported_reason, None);
    }

    #[test]
    fn page_query_result_envelope_describes_combined_deferred_results() {
        let result = PageQueryResultEnvelope::deferred(
            FoundPages { pages: Vec::new() },
            Some(100),
            true,
            true,
            true,
        );

        assert_eq!(result.metadata.candidate_count, Some(100));
        assert!(result.metadata.filtering_deferred_to_rust);
        assert!(result.metadata.ordering_deferred_to_rust);
        assert!(result.metadata.cap_exceeded);
        assert!(!result.metadata.sql_limit_offset_applied);
        assert!(!result.metadata.exact_count_safe);
        assert_eq!(result.metadata.unsupported_reason, None);
    }

    #[test]
    fn page_query_result_envelope_describes_cap_exceeded_results() {
        let result =
            PageQueryResultEnvelope::cap_exceeded(FoundPages { pages: Vec::new() }, 501);

        assert_eq!(result.metadata.candidate_count, Some(501));
        assert!(result.metadata.cap_exceeded);
        assert!(!result.metadata.exact_count_safe);
        assert_eq!(result.metadata.unsupported_reason, None);
    }

    #[test]
    fn page_query_result_envelope_describes_unsupported_results() {
        let result = PageQueryResultEnvelope::unsupported(
            FoundPages { pages: Vec::new() },
            "data form ordering",
        );

        assert_eq!(result.metadata.candidate_count, None);
        assert!(!result.metadata.cap_exceeded);
        assert!(!result.metadata.sql_limit_offset_applied);
        assert!(!result.metadata.exact_count_safe);
        assert_eq!(
            result.metadata.unsupported_reason.as_deref(),
            Some("data form ordering"),
        );
    }
}
