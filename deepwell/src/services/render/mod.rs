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

#[allow(unused_imports)]
mod prelude {
    pub use super::super::prelude::*;
    pub use super::structs::*;
    pub use ftml::data::PageInfo;
    pub use ftml::parsing::ParseError;
    pub use ftml::render::Render;
    pub use ftml::render::html::{HtmlOutput, HtmlRender};
    pub use ftml::settings::WikitextSettings;
    pub use ftml::{self};
}

mod compat_fallback_code;
mod compat_html_fragments;
mod compat_text_fragments;
mod diagnostics;
mod footnote_dom;
mod generator;
mod html_text;
mod iftags;
mod include_attachment_owners;
mod include_comment_branches;
mod include_variable_iftags;
mod issued_markers;
#[allow(dead_code)]
mod list_pages;
mod list_pages_content_sections;
mod list_pages_parents;
mod list_pages_row_values;
mod list_pages_scanner;
mod list_pages_template;
mod literal_regions;
mod metacomponent;
mod native_list_context;
mod pages_by_tag;
mod percent_encoding;
mod render_dependency;
mod replay;
mod service;
mod structs;
mod wikidot_compat_restore;
mod wikidot_embed;
mod wikidot_inline_markers;
mod wikidot_link_protection;
mod wikidot_residual_markers;

pub(crate) use self::diagnostics::{
    CORPUS_RENDER_BUDGET_US, CORPUS_RENDER_DIMENSIONS, CorpusRenderDimension,
    CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace, CorpusRenderTraceSnapshot,
    StageGuard, is_corpus_render_timing,
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
    CorpusReplayPreparedWikitext, CorpusReplayStageTimings, CorpusReplaySyntaxFeatures,
};
pub use self::structs::*;
