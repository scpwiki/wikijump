/*
 * routes/render.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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
use ftml::render::html::{HtmlMeta, HtmlOutput, HtmlRender};
use ftml::render::Render;
use ftml::tree::SyntaxTree;

#[derive(Serialize, Debug)]
struct RenderOutput<'a> {
    pages_included: Vec<PageRef<'a>>,
    text: &'a str,
    tokens: &'a [ExtractedToken<'a>],
    syntax_tree: SyntaxTree<'a>,
    warnings: Vec<ParseWarning>,
    html: &'a str,
    style: &'a str,
    meta: &'a [HtmlMeta],
}

pub fn route_render_html(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path!("render" / "html"))
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(move |input| {
            let (mut text, pages_included) =
                try_response!(run_include(&log, input)).into();

            ftml::preprocess(&log, &mut text);

            let tokenization = tokenize(&log, &text);
            let (syntax_tree, warnings) = ftml::parse(&log, &tokenization).into();
            let HtmlOutput { html, style, meta } = HtmlRender.render(&syntax_tree);

            let resp = Response::ok(RenderOutput {
                pages_included,
                text: &text,
                tokens: tokenization.tokens(),
                syntax_tree,
                warnings,
                html: &html,
                style: &style,
                meta: &meta,
            });

            warp::reply::json(&resp)
        })
}
