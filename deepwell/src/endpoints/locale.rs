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
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

#[derive(Serialize, Debug, Clone)]
pub struct LocaleOutput {
    language: String,
    script: Option<String>,
    region: Option<String>,
    variants: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TranslateInput<'a> {
    locale: &'a str,
    messages: HashMap<String, MessageArguments<'a>>,
}

type TranslateOutput = HashMap<String, String>;

pub async fn locale_info(
    _ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<LocaleOutput> {
    let locale_str: String = params.one()?;
    tide::log::info!("Getting locale information for {locale_str}");
    let locale = LanguageIdentifier::from_bytes(locale_str.as_bytes())?;
    Ok(LocaleOutput {
        language: str!(locale.language),
        script: locale.script.map(|s| str!(s)),
        region: locale.region.map(|s| str!(s)),
        variants: locale.variants().map(|v| str!(v)).collect(),
    })
}

pub async fn translate_strings(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<TranslateOutput> {
    let TranslateInput {
        locale: locale_str,
        messages,
    } = params.parse()?;

    tide::log::info!(
        "Translating {} message keys in locale {locale_str}",
        messages.len(),
    );

    let locale = LanguageIdentifier::from_bytes(locale_str.as_bytes())?;
    let mut output: TranslateOutput = HashMap::new();

    for (message_key, arguments_raw) in messages {
        tide::log::info!(
            "Formatting message key {message_key} ({} arguments)",
            arguments_raw.len(),
        );

        let arguments = arguments_raw.into_fluent_args();
        let translation =
            ctx.localization()
                .translate(&locale, &message_key, &arguments)?;

        output.insert(message_key, translation.to_string());
    }

    Ok(output)
}
