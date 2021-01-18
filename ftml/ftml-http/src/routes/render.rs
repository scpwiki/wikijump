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
use ftml::ParseOutcome;

pub fn route_render_html(
    log: &slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    use ftml::Render;

    let factory = |preprocess| {
        let log = log.clone();

        move |input| {
            let TextInput { mut text } = input;

            if preprocess {
                ftml::preprocess(&log, &mut text);
            }

            let tokens = ftml::tokenize(&log, &text);
            let parsed = ftml::parse(&log, &tokens);
            let (tree, errors) = parsed.into();
            let output = ftml::HtmlRender.render(&tree);
            let result = ParseOutcome::new(output, errors);

            warp::reply::json(&result)
        }
    };

    let regular = warp::path!("render" / "html")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(true));

    let only = warp::path!("render" / "html" / "only")
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(false));

    regular.or(only)
}
