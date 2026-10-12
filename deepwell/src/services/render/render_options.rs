/*
 * services/render/render_options.rs
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

//! Which page and request a render run is happening for.
//!
//! These describe the surroundings of a render rather than the wikitext being
//! rendered: which site and page the run belongs to, how much include
//! expansion it may spend, whether a corpus trace is collecting timings, and
//! which Wikidot URL path arguments the request carried.

use super::diagnostics::{CorpusRenderScope, CorpusRenderTrace};
use super::list_pages::ListPagesPagerRoute;
use super::render_budget::SharedRenderCostBudget;
use super::url_arguments::UrlArguments;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RenderLifecycle {
    Standalone,
    SavedPage,
    AjaxModule,
    PagePreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RenderContext {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_category_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
    pub(super) text_block_page_id: Option<i64>,
    pub(super) lifecycle: RenderLifecycle,
    pub(super) suppress_nested_list_pages: bool,
}

#[derive(Clone, Debug)]
pub(super) struct RenderInnerOptions<'a> {
    pub(super) observed_user_info: Option<Vec<ftml::data::UserInfo<'static>>>,
    pub(super) render_context: RenderContext,
    pub(super) viewer_user_id: Option<i64>,
    pub(super) current_page_data_form_values: Option<BTreeMap<String, String>>,
    pub(super) max_include_expansions: usize,
    pub(super) render_cost_budget: SharedRenderCostBudget,
    pub(super) trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
    pub(super) persist_compiled_text: bool,

    /// The Wikidot URL path arguments this request carried. Empty for every
    /// render that is not serving a page view, including the render that
    /// produces a revision's stored HTML.
    pub(super) url: UrlArguments<'a>,
}

/// What distinguishes one page render from another beyond its wikitext.
#[derive(Clone, Debug)]
pub(super) struct RenderPageOptions<'a> {
    pub(super) max_include_expansions: usize,
    pub(super) viewer_user_id: Option<i64>,
    pub(super) current_page_data_form_values: Option<BTreeMap<String, String>>,

    /// Whether this render may update compiled text and hosted page blocks.
    pub(super) persist_page_artifacts: bool,

    /// The Wikidot URL path arguments this request carried.
    pub(super) url: UrlArguments<'a>,

    pub(super) trace: Option<&'a CorpusRenderTrace>,
}

#[derive(Clone, Debug)]
pub(super) struct RenderExpansionOptions<'a> {
    pub(super) current_site_id: Option<i64>,
    pub(super) current_category_id: Option<i64>,
    pub(super) current_page_id: Option<i64>,
    pub(super) viewer_user_id: Option<i64>,
    pub(super) current_page_data_form_values: Option<BTreeMap<String, String>>,
    pub(super) max_include_expansions: usize,
    pub(super) render_cost_budget: SharedRenderCostBudget,
    pub(super) trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
    pub(super) url: UrlArguments<'a>,
    pub(super) list_pages_pager_route: ListPagesPagerRoute,
    pub(super) page_preview: bool,
    pub(super) suppress_nested_list_pages: bool,
}

impl RenderContext {
    pub(super) fn none() -> Self {
        Self {
            current_site_id: None,
            current_category_id: None,
            current_page_id: None,
            text_block_page_id: None,
            lifecycle: RenderLifecycle::Standalone,
            suppress_nested_list_pages: false,
        }
    }

    pub(super) fn page(site_id: i64, category_id: i64, page_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_category_id: Some(category_id),
            current_page_id: Some(page_id),
            text_block_page_id: Some(page_id),
            lifecycle: RenderLifecycle::SavedPage,
            suppress_nested_list_pages: false,
        }
    }

    pub(super) fn page_view(site_id: i64, category_id: i64, page_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_category_id: Some(category_id),
            current_page_id: Some(page_id),
            text_block_page_id: None,
            lifecycle: RenderLifecycle::SavedPage,
            suppress_nested_list_pages: false,
        }
    }

    pub(super) fn page_nav(site_id: i64, category_id: i64, current_page_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_category_id: Some(category_id),
            current_page_id: Some(current_page_id),
            text_block_page_id: None,
            lifecycle: RenderLifecycle::SavedPage,
            suppress_nested_list_pages: false,
        }
    }

    pub(super) fn ajax_module(site_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_category_id: None,
            current_page_id: Some(0),
            text_block_page_id: None,
            lifecycle: RenderLifecycle::AjaxModule,
            suppress_nested_list_pages: false,
        }
    }

    pub(super) fn page_preview_with_page(
        site_id: i64,
        current_page_id: Option<i64>,
    ) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_category_id: None,
            current_page_id,
            text_block_page_id: None,
            lifecycle: RenderLifecycle::PagePreview,
            suppress_nested_list_pages: false,
        }
    }

    pub(super) fn list_pages_default_summary(
        site_id: i64,
        category_id: i64,
        page_id: i64,
        page_preview: bool,
    ) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_category_id: (!page_preview).then_some(category_id),
            current_page_id: (!page_preview).then_some(page_id),
            text_block_page_id: None,
            lifecycle: if page_preview {
                RenderLifecycle::PagePreview
            } else {
                RenderLifecycle::SavedPage
            },
            suppress_nested_list_pages: true,
        }
    }

    pub(super) fn list_pages_pager_route(self) -> ListPagesPagerRoute {
        match self.lifecycle {
            RenderLifecycle::Standalone | RenderLifecycle::SavedPage => {
                ListPagesPagerRoute::SavedPage
            }
            RenderLifecycle::AjaxModule | RenderLifecycle::PagePreview => {
                ListPagesPagerRoute::AjaxModuleConnector
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_preview_has_site_state_without_saved_page_identity() {
        assert_eq!(
            RenderContext::page_preview_with_page(7, None),
            RenderContext {
                current_site_id: Some(7),
                current_category_id: None,
                current_page_id: None,
                text_block_page_id: None,
                lifecycle: RenderLifecycle::PagePreview,
                suppress_nested_list_pages: false,
            },
        );
        assert_eq!(
            RenderContext::page_preview_with_page(7, None).list_pages_pager_route(),
            ListPagesPagerRoute::AjaxModuleConnector,
        );
    }
}
