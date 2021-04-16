/*
 * render/html/handle.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use super::HtmlContext;
use crate::data::PageInfo;
use crate::tree::Module;

#[derive(Debug)]
pub struct Handle;

impl Handle {
    pub fn render_module(
        &self,
        log: &slog::Logger,
        ctx: &mut HtmlContext,
        module: &Module,
    ) {
        debug!(log, "Rendering module"; "module" => module.name());

        // TODO
        ctx.push_raw_str("<p>TODO: module ");
        ctx.push_escaped(module.name());
        ctx.push_raw_str("</p>");
    }

    pub fn get_page_title(&self, log: &slog::Logger, slug: &str) -> String {
        debug!(log, "Fetching page title"; "slug" => slug);

        // TODO
        format!("TODO: actual title ({})", slug)
    }

    pub fn get_message(
        &self,
        log: &slog::Logger,
        locale: &str,
        message: &str,
    ) -> &'static str {
        debug!(
            log,
            "Fetching message";
            "locale" => locale,
            "message" => message,
        );

        // TODO
        match message {
            "collapsible-open" => "+ open block",
            "collapsible-close" => "- open block",
            _ => {
                error!(
                    log,
                    "Unknown message requested";
                    "message" => message,
                );

                ""
            }
        }
    }

    pub fn post_html(&self, log: &slog::Logger, _info: &PageInfo, _html: &str) -> String {
        debug!(log, "Submitting HTML to create iframe-able snippet");

        // TODO
        str!("https://example.com/")
    }
}
