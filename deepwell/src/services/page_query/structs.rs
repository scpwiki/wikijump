/*
 * services/page_query/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use super::prelude::*;
use crate::models::{
    page::Model as PageModel, page_parent::Model as PageParentModel,
    page_revision::Model as PageRevisionModel,
};
use crate::services::score::ScoreValue;
use std::borrow::Cow;
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
}

/// The relationship of the pages being queried to their parent/child pages.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PageParentSelector<'a> {
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    DataFormFieldName,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub author: &'a [Cow<'a, str>],
    pub score: &'a [ScoreSelector], // 5-star rating selector
    pub votes: &'a [ScoreSelector], // upvote/downvote rating selector
    pub offset: u32,
    pub range: RangeSelector,
    pub name: Option<Cow<'a, str>>,
    pub slug: Option<Cow<'a, str>>,
    pub data_form_fields: &'a [DataFormSelector<'a>],
    pub order: Option<OrderBySelector>,
    pub pagination: PaginationSelector,
    pub variables: &'a [PageQueryVariables<'a>],
}

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct PageQueryOutput<'a>(&'a [PageResult]);

#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct PageResult {
    metadata: PageModel,
    last_revision: PageRevisionModel,
    // last_comment: TODO,
    page_parents: Vec<PageParentModel>,
    wikitext: String,
    score: f32,
}
