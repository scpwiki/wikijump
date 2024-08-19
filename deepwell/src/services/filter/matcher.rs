/*
 * services/filter/matcher.rs
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

use super::prelude::*;
use regex::RegexSet;

/// Describes one filter which a `FilterMatcher` can verify against.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FilterSummary {
    pub filter_id: i64,
    pub description: String,
}

/// Wrapper structure which determines which filter(s) a string violates.
///
/// Internally uses `RegexSet` for performance, and has fragments describing
/// each filter flagged by the given string.
#[derive(Debug)]
pub struct FilterMatcher {
    regex_set: RegexSet,
    filter_data: Vec<FilterSummary>,
}

impl FilterMatcher {
    #[inline]
    pub fn new(regex_set: RegexSet, filter_data: Vec<FilterSummary>) -> Self {
        FilterMatcher {
            regex_set,
            filter_data,
        }
    }

    /// Verifies that the given string does not trip any filters of this type.
    ///
    /// For any filter violations, they are logged and an error is returned.
    pub async fn verify(&self, ctx: &ServiceContext, text: &str) -> Result<()> {
        let matches = self.regex_set.matches(text);
        if !matches.matched_any() {
            info!("String passed all filters, is clear");
            return Ok(());
        }

        for index in matches {
            let description = &self.filter_data[index];
            error!(
                "String failed filter ID {}: {}",
                description.filter_id, description.description,
            );

            // TODO audit log, with contextual data (what it's checking)
            //      (will need to add extra args)
            let _ = ctx;
        }

        Err(Error::FilterViolation)
    }
}
