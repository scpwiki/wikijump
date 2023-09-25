/*
 * endpoints/locales.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::locales::MessageArguments;
use ref_map::*;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[derive(Serialize, Debug)]
struct LocaleOutput<'a> {
    language: &'a str,
    script: Option<&'a str>,
    region: Option<&'a str>,
    variants: Vec<String>,
}

type TranslateInput<'a> = HashMap<String, MessageArguments<'a>>;
type TranslateOutput = HashMap<String, String>;

pub async fn locale_get(req: ApiRequest) -> ApiResponse {
    let locale_str = req.param("locale")?;
    tide::log::info!("Getting locale information for {locale_str}");

    let locale = LanguageIdentifier::from_bytes(locale_str.as_bytes())?;
    let output = LocaleOutput {
        language: locale.language.as_str(),
        script: locale.script.ref_map(|s| s.as_str()),
        region: locale.region.ref_map(|s| s.as_str()),
        variants: locale.variants().map(|v| v.as_str().into()).collect(),
    };

    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn translate_put(mut req: ApiRequest) -> ApiResponse {
    let input: TranslateInput = req.body_json().await?;
    let locale_str = req.param("locale")?;
    let localizations = &req.state().localizations;
    tide::log::info!(
        "Translating {} message keys in locale {locale_str}",
        input.len(),
    );

    let locale = LanguageIdentifier::from_bytes(locale_str.as_bytes())?;
    let mut output: TranslateOutput = HashMap::new();

    for (message_key, arguments_raw) in input {
        tide::log::info!(
            "Formatting message key {message_key} ({} arguments)",
            arguments_raw.len(),
        );

        let arguments = arguments_raw.into_fluent_args();
        match localizations.translate(&locale, &message_key, &arguments) {
            Ok(translation) => output.insert(message_key, translation.to_string()),
            Err(error) => return Err(ServiceError::from(error).into_tide_error()),
        };
    }

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    Ok(response)
}
