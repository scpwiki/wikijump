/*
 * routes/preproc.rs
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct PreprocessInput {
    text: String,
    callback_url: String,
    missing_include_template: String,
}

#[derive(Serialize, Debug)]
struct PreprocessOutput<'a> {
    text: String,
    pages: Vec<PageRef<'a>>,
}

pub fn route_preproc(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("preprocess"))
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(move |input| {
            let resp: Response<PreprocessOutput> = run_preproc(&log, input).into();

            warp::reply::json(&resp)
        })
}

fn run_preproc(
    log: &slog::Logger,
    PreprocessInput {
        text,
        callback_url,
        missing_include_template,
    }: PreprocessInput,
) -> Result<PreprocessOutput, Error> {
    let (mut text, pages) =
        run_include(log, &text, &callback_url, &missing_include_template)?.into();

    ftml::preprocess(log, &mut text);

    Ok(PreprocessOutput { text, pages })
}
