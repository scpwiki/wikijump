/*
 * services/render/mod.rs
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

mod backlinks;
mod categories;
mod compat;
mod corpus;
mod diagnostics;
mod ftml_page_existence;
mod ftml_user_info;
mod generator;
mod html_text;
mod iftags;
mod include_attachment_owners;
mod include_comment_branches;
mod include_missing;
mod include_variable_iftags;
mod include_variables;
#[allow(dead_code)]
mod list_pages;
mod literal_regions;
mod metacomponent;
mod module_arguments;
mod native_list_context;
mod page_preview;
mod page_tree;
mod pages;
mod pages_by_tag;
mod percent_encoding;
mod render_dependency;
mod render_options;
mod replay;
mod runtime;
mod runtime_modules;
mod runtime_page_queries;
mod service;
mod structs;
mod url_arguments;
mod wikidot_hosts;

pub(crate) use self::corpus::{
    CorpusRenderFinalizerService, CorpusRenderInventoryService, RenderFinalizerSettings,
    RenderInventorySettings,
};
pub(crate) use self::diagnostics::{
    CORPUS_RENDER_BUDGET_US, CORPUS_RENDER_DIMENSIONS, CorpusRenderScope,
    CorpusRenderStage, CorpusRenderTrace, CorpusRenderTraceSnapshot, StageGuard,
    is_corpus_render_timing,
};
pub(crate) use self::literal_regions::LiteralRegionIndex;
pub use self::render_dependency::{
    RenderDependencyClass, RenderDependencyClasses, classify_render_dependencies,
};
pub(crate) use self::replay::{
    RenderReplayService, RenderReplaySettings, run_worker_action,
};
pub use self::service::RenderService;
pub(crate) use self::service::{
    CorpusReplayExpandedWikitext, CorpusReplayPreparationStage,
};
pub use self::structs::{
    RenderOutput, RenderPageOutput, WikidotListPagesFeedInput, WikidotListPagesFeedItem,
    WikidotListPagesFeedOutput,
};
pub use self::url_arguments::{
    UrlArgumentPair, UrlArguments, wikitext_reads_url_arguments,
    wikitext_requires_runtime_render,
};
