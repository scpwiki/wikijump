/*
 * services/filter/matcher.rs
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService, ObjectScope};
use regex::RegexSet;
use std::fmt;
use std::net::IpAddr;

/// Describes one filter which a `FilterMatcher` can verify against.
#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct FilterSummary {
    pub filter_id: i64,
    pub regex: String,
    pub description: String,
}

impl fmt::Debug for FilterSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterSummary")
            .field("filter_id", &self.filter_id)
            .field("regex", &"<redacted>")
            .field("description", &"<redacted>")
            .finish()
    }
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
    pub async fn verify(
        &self,
        ctx: &ServiceContext<'_>,
        field: &'static str,
        value: &str,
        object: ObjectScope,
        ip_address: IpAddr,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to verify filter (field '{}', value '{}')",
                    field, value,
                ),
                ErrorType::Filter,
            )
        };

        let matches = self.regex_set.matches(value);
        if !matches.matched_any() {
            info!("String passed all filters, is clear");
            return Ok(());
        }

        let mut failed = Vec::new();
        for index in matches {
            let info = &self.filter_data[index];
            error!("String failed filter: {info:?}");

            AuditService::log(
                ctx,
                ip_address,
                AuditEvent::FilterViolation {
                    object,
                    info,
                    field,
                    value,
                },
            )
            .await
            .or_raise(make_error)?;

            failed.push(info.clone());
        }

        bail!(Error::new(
            format!("filter failure for field '{field}'"),
            ErrorType::FilterViolation {
                field: str!(field),
                value: str!(value),
                failed,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcher_keeps_regex_set_and_filter_metadata_aligned() {
        let filters = vec![FilterSummary {
            filter_id: 1,
            regex: str!("forbidden"),
            description: str!("test filter"),
        }];
        let matcher = FilterMatcher::new(
            RegexSet::new(filters.iter().map(|filter| filter.regex.as_str())).unwrap(),
            filters,
        );

        assert!(matcher.regex_set.is_match("forbidden word"));
        assert_eq!(matcher.filter_data[0].filter_id, 1);
    }

    #[test]
    fn filter_summary_debug_redacts_private_filter_details() {
        let summary = FilterSummary {
            filter_id: 7,
            regex: str!("SECRET_ADMIN_REGEX"),
            description: str!("PRIVATE admin-only filter description"),
        };

        let debug = format!("{summary:?}");

        assert!(debug.contains("filter_id: 7"));
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("SECRET_ADMIN_REGEX"));
        assert!(!debug.contains("PRIVATE admin-only filter description"));
    }
}
