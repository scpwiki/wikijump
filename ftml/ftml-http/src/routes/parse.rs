/*
 * routes/parse.rs
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
use ftml::tree::SyntaxTree;
use ftml::ParseWarning;

#[derive(Serialize, Debug)]
struct ParseOutput<'a> {
    text: &'a str,
    tokens: &'a [ExtractedToken<'a>],
    syntax_tree: SyntaxTree<'a>,
    warnings: Vec<ParseWarning>,
    pages_included: Vec<PageRef<'a>>,
}

pub fn route_parse(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("parse"))
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(move |input| {
            let (mut text, pages_included) =
                try_response!(run_include(&log, input)).into();

            ftml::preprocess(&log, &mut text);

            let tokenization = ftml::tokenize(&log, &text);
            let (syntax_tree, warnings) = ftml::parse(&log, &tokenization).into();

            let resp = Response::ok(ParseOutput {
                text: &text,
                tokens: tokenization.tokens(),
                syntax_tree,
                warnings,
                pages_included,
            });

            warp::reply::json(&resp)
        })
}
