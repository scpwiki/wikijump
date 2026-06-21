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

use super::prelude::*;
use crate::services::audit::{AuditEvent, AuditService, ObjectScope};
use regex::RegexSet;
use std::net::IpAddr;

/// Describes one filter which a `FilterMatcher` can verify against.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FilterSummary {
    pub filter_id: i64,
    pub regex: String,
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
            error!(
                "String failed filter ID {} (regex '{}'): {}",
                info.filter_id, info.regex, info.description,
            );

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
