/*
 * services/page_query/mod.rs
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

mod compat_select;
mod count_pages;
mod list_pages;
mod service;
mod structs;

pub use self::compat_select::{SelectPageTags, SelectPages};
pub use self::count_pages::{
    CountPagesExactCountEligibilityDiagnostics, CountPagesExactCountEligibilityInput,
    count_pages_exact_count_eligibility_diagnostics,
};
pub use self::list_pages::{
    ListPagesRenderDiagnosticsInput, list_pages_render_diagnostics,
};
pub use self::service::PageQueryService;
pub(crate) use self::service::{PageQueryScoreFilterCache, PageQueryScoreFilterSession};
pub(crate) use self::structs::MAX_PAGE_QUERY_SCORE_SELECTORS;
pub use self::structs::{
    AuthorSelector, CategoriesSelector, ComparisonOperation, DataFormSelector,
    DateSelector, DateTimeResolution, FoundPageFields, FoundPageRow, FoundPages,
    IncludedCategories, OrderBySelector, OrderProperty, PageParentSelector, PageQuery,
    PageQueryResultMetadata, PageTypeSelector, PaginationSelector, RangeSelector,
    ScoreSelector, TagCondition, normalize_wikidot_author_name,
    parse_static_wikidot_data_form_values, static_wikidot_data_form_matches,
};
