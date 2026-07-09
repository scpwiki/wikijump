/*
 * services/page_query/list_pages.rs
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
pub struct ListPagesRenderDiagnosticsInput {
    pub metadata: PageQueryResultMetadata,
    pub view_permission_filtering_applied: bool,
    pub post_query_exclusion_applied: bool,
    pub post_query_offset_applied: bool,
    pub requested_limit: u64,
    pub query_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListPagesRenderDiagnostics {
    pub blocked: bool,
    pub blocker_code: Option<&'static str>,
    pub blocker_detail: Option<String>,
    pub requested_limit: u64,
    pub query_limit: u64,
    pub candidate_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ListPagesRenderBlocker {
    Unsupported { reason: String },
    CapExceeded,
    DeferredFiltering,
    DeferredOrdering,
    NotExactCountSafe,
    ViewPermissionFiltering,
    PostQueryExclusion,
    PostQueryOffset,
    UnsafeSqlWindow,
}

impl ListPagesRenderBlocker {
    fn diagnostic_code(&self) -> &'static str {
        match self {
            ListPagesRenderBlocker::Unsupported { .. } => "unsupported",
            ListPagesRenderBlocker::CapExceeded => "cap_exceeded",
            ListPagesRenderBlocker::DeferredFiltering => "deferred_filtering",
            ListPagesRenderBlocker::DeferredOrdering => "deferred_ordering",
            ListPagesRenderBlocker::NotExactCountSafe => "not_exact_count_safe",
            ListPagesRenderBlocker::ViewPermissionFiltering => {
                "view_permission_filtering"
            }
            ListPagesRenderBlocker::PostQueryExclusion => "post_query_exclusion",
            ListPagesRenderBlocker::PostQueryOffset => "post_query_offset",
            ListPagesRenderBlocker::UnsafeSqlWindow => "unsafe_sql_window",
        }
    }

    fn diagnostic_detail(&self) -> Option<String> {
        match self {
            ListPagesRenderBlocker::Unsupported { reason } => Some(reason.clone()),
            ListPagesRenderBlocker::CapExceeded
            | ListPagesRenderBlocker::DeferredFiltering
            | ListPagesRenderBlocker::DeferredOrdering
            | ListPagesRenderBlocker::NotExactCountSafe
            | ListPagesRenderBlocker::ViewPermissionFiltering
            | ListPagesRenderBlocker::PostQueryExclusion
            | ListPagesRenderBlocker::PostQueryOffset
            | ListPagesRenderBlocker::UnsafeSqlWindow => None,
        }
    }
}

pub fn list_pages_render_diagnostics(
    input: ListPagesRenderDiagnosticsInput,
) -> ListPagesRenderDiagnostics {
    let blocker = if let Some(reason) = input.metadata.unsupported_reason.as_ref() {
        Some(ListPagesRenderBlocker::Unsupported {
            reason: reason.clone(),
        })
    } else if input.metadata.cap_exceeded {
        Some(ListPagesRenderBlocker::CapExceeded)
    } else if input.metadata.filtering_deferred_to_rust {
        Some(ListPagesRenderBlocker::DeferredFiltering)
    } else if input.metadata.ordering_deferred_to_rust {
        Some(ListPagesRenderBlocker::DeferredOrdering)
    } else if !input.metadata.exact_count_safe {
        Some(ListPagesRenderBlocker::NotExactCountSafe)
    } else if input.view_permission_filtering_applied {
        Some(ListPagesRenderBlocker::ViewPermissionFiltering)
    } else if input.post_query_exclusion_applied {
        Some(ListPagesRenderBlocker::PostQueryExclusion)
    } else if input.post_query_offset_applied {
        Some(ListPagesRenderBlocker::PostQueryOffset)
    } else if input.metadata.sql_limit_offset_applied
        && input
            .metadata
            .candidate_count
            .is_some_and(|candidate_count| candidate_count > input.query_limit as usize)
    {
        Some(ListPagesRenderBlocker::UnsafeSqlWindow)
    } else {
        None
    };

    ListPagesRenderDiagnostics {
        blocked: blocker.is_some(),
        blocker_code: blocker
            .as_ref()
            .map(ListPagesRenderBlocker::diagnostic_code),
        blocker_detail: blocker
            .as_ref()
            .and_then(ListPagesRenderBlocker::diagnostic_detail),
        requested_limit: input.requested_limit,
        query_limit: input.query_limit,
        candidate_count: input.metadata.candidate_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::page_query::PageQueryResultMetadata;

    fn exact_metadata() -> PageQueryResultMetadata {
        PageQueryResultMetadata {
            candidate_count: Some(12),
            exact_count_safe: true,
            ..PageQueryResultMetadata::default()
        }
    }

    fn input(metadata: PageQueryResultMetadata) -> ListPagesRenderDiagnosticsInput {
        ListPagesRenderDiagnosticsInput {
            metadata,
            view_permission_filtering_applied: false,
            post_query_exclusion_applied: false,
            post_query_offset_applied: false,
            requested_limit: 10,
            query_limit: 12,
        }
    }

    fn diagnostics(input: ListPagesRenderDiagnosticsInput) -> ListPagesRenderDiagnostics {
        list_pages_render_diagnostics(input)
    }

    #[test]
    fn list_pages_render_diagnostics_allows_exact_metadata() {
        assert_eq!(
            diagnostics(input(exact_metadata())),
            ListPagesRenderDiagnostics {
                blocked: false,
                blocker_code: None,
                blocker_detail: None,
                requested_limit: 10,
                query_limit: 12,
                candidate_count: Some(12),
            },
        );
    }

    #[test]
    fn list_pages_render_diagnostics_preserves_unsupported_detail() {
        let mut metadata = exact_metadata();
        metadata.unsupported_reason = Some("data form ordering".to_owned());

        assert_eq!(
            diagnostics(input(metadata)),
            ListPagesRenderDiagnostics {
                blocked: true,
                blocker_code: Some("unsupported"),
                blocker_detail: Some("data form ordering".to_owned()),
                requested_limit: 10,
                query_limit: 12,
                candidate_count: Some(12),
            },
        );
    }

    #[test]
    fn list_pages_render_diagnostics_maps_every_blocker_to_stable_code() {
        let cases = [
            (
                "cap_exceeded",
                apply_cap_exceeded as fn(&mut ListPagesRenderDiagnosticsInput),
            ),
            ("deferred_filtering", apply_deferred_filtering),
            ("deferred_ordering", apply_deferred_ordering),
            ("not_exact_count_safe", apply_not_exact_count_safe),
            ("view_permission_filtering", apply_view_permission_filtering),
            ("post_query_exclusion", apply_post_query_exclusion),
            ("post_query_offset", apply_post_query_offset),
            ("unsafe_sql_window", apply_unsafe_sql_window),
        ];

        for (code, apply) in cases {
            let mut input = input(exact_metadata());
            apply(&mut input);

            assert_eq!(
                diagnostics(input).blocker_code,
                Some(code),
                "{code} should have a stable diagnostic code",
            );
        }
    }

    #[test]
    fn list_pages_render_diagnostics_keep_blocker_priority_order() {
        let priority = [
            apply_cap_exceeded as fn(&mut ListPagesRenderDiagnosticsInput),
            apply_deferred_filtering,
            apply_deferred_ordering,
            apply_not_exact_count_safe,
            apply_view_permission_filtering,
            apply_post_query_exclusion,
            apply_post_query_offset,
            apply_unsafe_sql_window,
        ];
        let expected = [
            "cap_exceeded",
            "deferred_filtering",
            "deferred_ordering",
            "not_exact_count_safe",
            "view_permission_filtering",
            "post_query_exclusion",
            "post_query_offset",
            "unsafe_sql_window",
        ];

        for (index, code) in expected.iter().enumerate() {
            let mut input = input(exact_metadata());
            for apply in &priority[index..] {
                apply(&mut input);
            }

            assert_eq!(diagnostics(input).blocker_code, Some(*code));
        }
    }

    fn apply_cap_exceeded(input: &mut ListPagesRenderDiagnosticsInput) {
        input.metadata.cap_exceeded = true;
    }

    fn apply_deferred_filtering(input: &mut ListPagesRenderDiagnosticsInput) {
        input.metadata.filtering_deferred_to_rust = true;
    }

    fn apply_deferred_ordering(input: &mut ListPagesRenderDiagnosticsInput) {
        input.metadata.ordering_deferred_to_rust = true;
    }

    fn apply_not_exact_count_safe(input: &mut ListPagesRenderDiagnosticsInput) {
        input.metadata.exact_count_safe = false;
    }

    fn apply_view_permission_filtering(input: &mut ListPagesRenderDiagnosticsInput) {
        input.view_permission_filtering_applied = true;
    }

    fn apply_post_query_exclusion(input: &mut ListPagesRenderDiagnosticsInput) {
        input.post_query_exclusion_applied = true;
    }

    fn apply_post_query_offset(input: &mut ListPagesRenderDiagnosticsInput) {
        input.post_query_offset_applied = true;
    }

    fn apply_unsafe_sql_window(input: &mut ListPagesRenderDiagnosticsInput) {
        input.metadata.sql_limit_offset_applied = true;
        input.metadata.candidate_count = Some(13);
        input.query_limit = 12;
    }
}
