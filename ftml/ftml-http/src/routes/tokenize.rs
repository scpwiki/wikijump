/*
 * routes/tokenize.rs
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

#[derive(Serialize, Debug)]
struct TokenizeOutput<'a> {
    pages_included: Vec<PageRef<'a>>,
    text: &'a str,
    tokens: &'a [ExtractedToken<'a>],
}

pub fn route_tokenize(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("tokenize"))
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(move |input| {
            let (mut text, pages_included) =
                try_response!(run_include(&log, input)).into();

            ftml::preprocess(&log, &mut text);

            let tokenization = ftml::tokenize(&log, &text);
            let resp = Response::ok(TokenizeOutput {
                pages_included,
                text: &text,
                tokens: tokenization.tokens(),
            });

            warp::reply::json(&resp)
        })
}
