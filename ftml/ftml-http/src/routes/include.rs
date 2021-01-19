/*
 * routes/include.rs
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
struct IncludeInput {
    text: String,
    callback_url: String,
    missing_include_template: String,
}

pub fn route_include(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("include"))
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(move |input| {
            let IncludeInput {
                text,
                callback_url,
                missing_include_template,
            } = input;

            let resp: Response<IncludeOutput> =
                run_include(&log, &text, &callback_url, &missing_include_template).into();

            warp::reply::json(&resp)
        })
}

pub fn run_include(
    log: &slog::Logger,
    text: &str,
    callback_url: &str,
    missing_include_template: &str,
) -> Result<IncludeOutput<'static>, Error> {
    let includer = HttpIncluder::new(&callback_url, &missing_include_template)?;

    match ftml::include(log, &text, includer) {
        Ok((output, pages)) => {
            info!(
                log,
                "Got successful return for page inclusions";
                "output" => &output,
                "pages" => pages.len(),
            );

            // Clone page references to avoid lifetime issues
            Ok(IncludeOutput {
                text: output,
                pages: pages.iter().map(PageRef::to_owned).collect(),
            })
        }
        Err(error) => {
            warn!(
                log,
                "Error fetching included pages or data";
                "error" => str!(error),
            );

            Err(error)
        }
    }
}
