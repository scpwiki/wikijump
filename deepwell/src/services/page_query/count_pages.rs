/*
 * services/page_query/count_pages.rs
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

use super::PageQueryResultMetadata;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountPagesExactCountEligibilityInput {
    pub metadata: PageQueryResultMetadata,
    pub view_permission_filtering_applied: bool,
    pub post_query_filtering_applied: bool,
    pub post_query_exclusion_applied: bool,
    pub post_query_offset_applied: bool,
    pub explicit_count_pages_bound_matches_sql_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountPagesExactCountEligibilityDecision {
    pub allowed: bool,
    pub denied_reason: Option<CountPagesExactCountDenialReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CountPagesExactCountEligibilityDiagnostics {
    pub allowed: bool,
    pub denied_reason_code: Option<&'static str>,
    pub denied_reason_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountPagesExactCountDenialReason {
    Unsupported { reason: String },
    CapExceeded,
    FilteringDeferredToRust,
    OrderingDeferredToRust,
    NotExactCountSafe,
    UnsafeSqlWindow,
    ViewPermissionFiltering,
    PostQueryFiltering,
    PostQueryExclusion,
    PostQueryOffset,
}

impl CountPagesExactCountDenialReason {
    pub fn diagnostic_code(&self) -> &'static str {
        match self {
            CountPagesExactCountDenialReason::Unsupported { .. } => "unsupported",
            CountPagesExactCountDenialReason::CapExceeded => "cap_exceeded",
            CountPagesExactCountDenialReason::FilteringDeferredToRust => {
                "filtering_deferred_to_rust"
            }
            CountPagesExactCountDenialReason::OrderingDeferredToRust => {
                "ordering_deferred_to_rust"
            }
            CountPagesExactCountDenialReason::NotExactCountSafe => "not_exact_count_safe",
            CountPagesExactCountDenialReason::UnsafeSqlWindow => "unsafe_sql_window",
            CountPagesExactCountDenialReason::ViewPermissionFiltering => {
                "view_permission_filtering"
            }
            CountPagesExactCountDenialReason::PostQueryFiltering => {
                "post_query_filtering"
            }
            CountPagesExactCountDenialReason::PostQueryExclusion => {
                "post_query_exclusion"
            }
            CountPagesExactCountDenialReason::PostQueryOffset => "post_query_offset",
        }
    }

    pub fn diagnostic_detail(&self) -> Option<String> {
        match self {
            CountPagesExactCountDenialReason::Unsupported { reason } => {
                Some(reason.clone())
            }
            CountPagesExactCountDenialReason::CapExceeded
            | CountPagesExactCountDenialReason::FilteringDeferredToRust
            | CountPagesExactCountDenialReason::OrderingDeferredToRust
            | CountPagesExactCountDenialReason::NotExactCountSafe
            | CountPagesExactCountDenialReason::UnsafeSqlWindow
            | CountPagesExactCountDenialReason::ViewPermissionFiltering
            | CountPagesExactCountDenialReason::PostQueryFiltering
            | CountPagesExactCountDenialReason::PostQueryExclusion
            | CountPagesExactCountDenialReason::PostQueryOffset => None,
        }
    }
}

pub fn count_pages_exact_count_eligibility(
    input: CountPagesExactCountEligibilityInput,
) -> CountPagesExactCountEligibilityDecision {
    let denied_reason = if let Some(reason) = input.metadata.unsupported_reason {
        Some(CountPagesExactCountDenialReason::Unsupported { reason })
    } else if input.metadata.cap_exceeded {
        Some(CountPagesExactCountDenialReason::CapExceeded)
    } else if input.metadata.filtering_deferred_to_rust {
        Some(CountPagesExactCountDenialReason::FilteringDeferredToRust)
    } else if input.metadata.ordering_deferred_to_rust {
        Some(CountPagesExactCountDenialReason::OrderingDeferredToRust)
    } else if !input.metadata.exact_count_safe {
        Some(CountPagesExactCountDenialReason::NotExactCountSafe)
    } else if input.view_permission_filtering_applied {
        Some(CountPagesExactCountDenialReason::ViewPermissionFiltering)
    } else if input.post_query_filtering_applied {
        Some(CountPagesExactCountDenialReason::PostQueryFiltering)
    } else if input.post_query_exclusion_applied {
        Some(CountPagesExactCountDenialReason::PostQueryExclusion)
    } else if input.post_query_offset_applied {
        Some(CountPagesExactCountDenialReason::PostQueryOffset)
    } else if input.metadata.sql_limit_offset_applied
        && !input.explicit_count_pages_bound_matches_sql_window
    {
        Some(CountPagesExactCountDenialReason::UnsafeSqlWindow)
    } else {
        None
    };

    CountPagesExactCountEligibilityDecision {
        allowed: denied_reason.is_none(),
        denied_reason,
    }
}

pub fn count_pages_exact_count_eligibility_diagnostics(
    input: CountPagesExactCountEligibilityInput,
) -> CountPagesExactCountEligibilityDiagnostics {
    let decision = count_pages_exact_count_eligibility(input);

    CountPagesExactCountEligibilityDiagnostics {
        allowed: decision.allowed,
        denied_reason_code: decision
            .denied_reason
            .as_ref()
            .map(CountPagesExactCountDenialReason::diagnostic_code),
        denied_reason_detail: decision
            .denied_reason
            .as_ref()
            .and_then(CountPagesExactCountDenialReason::diagnostic_detail),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact_metadata() -> PageQueryResultMetadata {
        PageQueryResultMetadata {
            candidate_count: Some(12),
            exact_count_safe: true,
            ..PageQueryResultMetadata::default()
        }
    }

    fn input(metadata: PageQueryResultMetadata) -> CountPagesExactCountEligibilityInput {
        CountPagesExactCountEligibilityInput {
            metadata,
            view_permission_filtering_applied: false,
            post_query_filtering_applied: false,
            post_query_exclusion_applied: false,
            post_query_offset_applied: false,
            explicit_count_pages_bound_matches_sql_window: false,
        }
    }

    fn reason(
        input: CountPagesExactCountEligibilityInput,
    ) -> Option<CountPagesExactCountDenialReason> {
        count_pages_exact_count_eligibility(input).denied_reason
    }

    fn diagnostics(
        input: CountPagesExactCountEligibilityInput,
    ) -> CountPagesExactCountEligibilityDiagnostics {
        count_pages_exact_count_eligibility_diagnostics(input)
    }

    fn apply_denial(
        input: &mut CountPagesExactCountEligibilityInput,
        reason: &CountPagesExactCountDenialReason,
    ) {
        match reason {
            CountPagesExactCountDenialReason::Unsupported { reason } => {
                input.metadata.unsupported_reason = Some(reason.clone());
            }
            CountPagesExactCountDenialReason::CapExceeded => {
                input.metadata.cap_exceeded = true;
            }
            CountPagesExactCountDenialReason::FilteringDeferredToRust => {
                input.metadata.filtering_deferred_to_rust = true;
            }
            CountPagesExactCountDenialReason::OrderingDeferredToRust => {
                input.metadata.ordering_deferred_to_rust = true;
            }
            CountPagesExactCountDenialReason::NotExactCountSafe => {
                input.metadata.exact_count_safe = false;
            }
            CountPagesExactCountDenialReason::UnsafeSqlWindow => {
                input.metadata.sql_limit_offset_applied = true;
                input.explicit_count_pages_bound_matches_sql_window = false;
            }
            CountPagesExactCountDenialReason::ViewPermissionFiltering => {
                input.view_permission_filtering_applied = true;
            }
            CountPagesExactCountDenialReason::PostQueryFiltering => {
                input.post_query_filtering_applied = true;
            }
            CountPagesExactCountDenialReason::PostQueryExclusion => {
                input.post_query_exclusion_applied = true;
            }
            CountPagesExactCountDenialReason::PostQueryOffset => {
                input.post_query_offset_applied = true;
            }
        }
    }

    #[test]
    fn count_pages_exact_count_allows_plain_exact_metadata() {
        let decision = count_pages_exact_count_eligibility(input(exact_metadata()));

        assert!(decision.allowed);
        assert_eq!(decision.denied_reason, None);
    }

    #[test]
    fn count_pages_exact_count_denies_score_deferred_ordering() {
        let mut metadata = exact_metadata();
        metadata.ordering_deferred_to_rust = true;
        metadata.exact_count_safe = false;

        assert_eq!(
            reason(input(metadata)),
            Some(CountPagesExactCountDenialReason::OrderingDeferredToRust),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_data_form_deferred_filtering() {
        let mut metadata = exact_metadata();
        metadata.filtering_deferred_to_rust = true;
        metadata.exact_count_safe = false;

        assert_eq!(
            reason(input(metadata)),
            Some(CountPagesExactCountDenialReason::FilteringDeferredToRust),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_cap_exceeded() {
        let mut metadata = exact_metadata();
        metadata.cap_exceeded = true;
        metadata.exact_count_safe = false;

        assert_eq!(
            reason(input(metadata)),
            Some(CountPagesExactCountDenialReason::CapExceeded),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_unsupported_query() {
        let mut metadata = exact_metadata();
        metadata.unsupported_reason = Some("data form ordering".to_owned());
        metadata.exact_count_safe = false;

        assert_eq!(
            reason(input(metadata)),
            Some(CountPagesExactCountDenialReason::Unsupported {
                reason: "data form ordering".to_owned(),
            }),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_unsafe_sql_window() {
        let mut metadata = exact_metadata();
        metadata.sql_limit_offset_applied = true;

        assert_eq!(
            reason(input(metadata)),
            Some(CountPagesExactCountDenialReason::UnsafeSqlWindow),
        );
    }

    #[test]
    fn count_pages_exact_count_allows_sql_window_with_explicit_matching_bound() {
        let mut metadata = exact_metadata();
        metadata.sql_limit_offset_applied = true;
        let mut input = input(metadata);
        input.explicit_count_pages_bound_matches_sql_window = true;

        let decision = count_pages_exact_count_eligibility(input);

        assert!(decision.allowed);
        assert_eq!(decision.denied_reason, None);
    }

    #[test]
    fn count_pages_exact_count_denies_view_permission_filtering() {
        let mut input = input(exact_metadata());
        input.view_permission_filtering_applied = true;

        assert_eq!(
            reason(input),
            Some(CountPagesExactCountDenialReason::ViewPermissionFiltering),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_post_query_exclusion() {
        let mut input = input(exact_metadata());
        input.post_query_exclusion_applied = true;

        assert_eq!(
            reason(input),
            Some(CountPagesExactCountDenialReason::PostQueryExclusion),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_post_query_offset() {
        let mut input = input(exact_metadata());
        input.post_query_offset_applied = true;

        assert_eq!(
            reason(input),
            Some(CountPagesExactCountDenialReason::PostQueryOffset),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_post_query_filtering() {
        let mut input = input(exact_metadata());
        input.post_query_filtering_applied = true;

        assert_eq!(
            reason(input),
            Some(CountPagesExactCountDenialReason::PostQueryFiltering),
        );
    }

    #[test]
    fn count_pages_exact_count_denies_metadata_not_marked_exact_count_safe() {
        let mut metadata = exact_metadata();
        metadata.exact_count_safe = false;

        assert_eq!(
            reason(input(metadata)),
            Some(CountPagesExactCountDenialReason::NotExactCountSafe),
        );
    }

    #[test]
    fn count_pages_exact_count_diagnostics_allow_without_denial_fields() {
        assert_eq!(
            diagnostics(input(exact_metadata())),
            CountPagesExactCountEligibilityDiagnostics {
                allowed: true,
                denied_reason_code: None,
                denied_reason_detail: None,
            },
        );
    }

    #[test]
    fn count_pages_exact_count_diagnostics_preserve_unsupported_detail() {
        let mut metadata = exact_metadata();
        metadata.unsupported_reason = Some("data form ordering".to_owned());

        assert_eq!(
            diagnostics(input(metadata)),
            CountPagesExactCountEligibilityDiagnostics {
                allowed: false,
                denied_reason_code: Some("unsupported"),
                denied_reason_detail: Some("data form ordering".to_owned()),
            },
        );
    }

    #[test]
    fn count_pages_exact_count_diagnostics_map_every_denial_to_stable_code() {
        let cases = [
            (
                CountPagesExactCountDenialReason::Unsupported {
                    reason: "unsupported selector".to_owned(),
                },
                "unsupported",
                Some("unsupported selector"),
            ),
            (
                CountPagesExactCountDenialReason::CapExceeded,
                "cap_exceeded",
                None,
            ),
            (
                CountPagesExactCountDenialReason::FilteringDeferredToRust,
                "filtering_deferred_to_rust",
                None,
            ),
            (
                CountPagesExactCountDenialReason::OrderingDeferredToRust,
                "ordering_deferred_to_rust",
                None,
            ),
            (
                CountPagesExactCountDenialReason::NotExactCountSafe,
                "not_exact_count_safe",
                None,
            ),
            (
                CountPagesExactCountDenialReason::UnsafeSqlWindow,
                "unsafe_sql_window",
                None,
            ),
            (
                CountPagesExactCountDenialReason::ViewPermissionFiltering,
                "view_permission_filtering",
                None,
            ),
            (
                CountPagesExactCountDenialReason::PostQueryFiltering,
                "post_query_filtering",
                None,
            ),
            (
                CountPagesExactCountDenialReason::PostQueryExclusion,
                "post_query_exclusion",
                None,
            ),
            (
                CountPagesExactCountDenialReason::PostQueryOffset,
                "post_query_offset",
                None,
            ),
        ];

        for (reason, code, detail) in cases {
            assert_eq!(reason.diagnostic_code(), code);
            assert_eq!(reason.diagnostic_detail().as_deref(), detail);
        }
    }

    #[test]
    fn count_pages_exact_count_diagnostics_keep_denial_priority_order() {
        let mut metadata = exact_metadata();
        metadata.unsupported_reason = Some("unsupported selector".to_owned());
        metadata.cap_exceeded = true;
        metadata.filtering_deferred_to_rust = true;
        metadata.ordering_deferred_to_rust = true;
        metadata.exact_count_safe = false;
        metadata.sql_limit_offset_applied = true;
        let mut input = input(metadata);
        input.view_permission_filtering_applied = true;
        input.post_query_filtering_applied = true;
        input.post_query_exclusion_applied = true;
        input.post_query_offset_applied = true;

        assert_eq!(
            diagnostics(input),
            CountPagesExactCountEligibilityDiagnostics {
                allowed: false,
                denied_reason_code: Some("unsupported"),
                denied_reason_detail: Some("unsupported selector".to_owned()),
            },
        );
    }

    #[test]
    fn count_pages_exact_count_diagnostics_keep_priority_order_below_unsupported() {
        let priority = [
            CountPagesExactCountDenialReason::CapExceeded,
            CountPagesExactCountDenialReason::FilteringDeferredToRust,
            CountPagesExactCountDenialReason::OrderingDeferredToRust,
            CountPagesExactCountDenialReason::NotExactCountSafe,
            CountPagesExactCountDenialReason::ViewPermissionFiltering,
            CountPagesExactCountDenialReason::PostQueryFiltering,
            CountPagesExactCountDenialReason::PostQueryExclusion,
            CountPagesExactCountDenialReason::PostQueryOffset,
            CountPagesExactCountDenialReason::UnsafeSqlWindow,
        ];

        for (index, expected) in priority.iter().enumerate() {
            let mut input = input(exact_metadata());
            for reason in &priority[index..] {
                apply_denial(&mut input, reason);
            }

            assert_eq!(
                diagnostics(input).denied_reason_code,
                Some(expected.diagnostic_code()),
                "{expected:?} should win over every lower-priority exact-count denial",
            );
        }
    }
}
