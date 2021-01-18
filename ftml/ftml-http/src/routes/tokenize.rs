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

pub fn route_tokenize(
    log: &slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let factory = |preprocess| {
        let log = log.clone();

        move |input| {
            let TextInput { mut text } = input;

            if preprocess {
                ftml::preprocess(&log, &mut text);
            }

            let result = ftml::tokenize(&log, &text);
            let tokens = result.tokens();
            warp::reply::json(&tokens)
        }
    };

    let regular = warp::path("tokenize")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(true));

    let only = warp::path!("tokenize" / "only")
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(false));

    regular.or(only)
}
