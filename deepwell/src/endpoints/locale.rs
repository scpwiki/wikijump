/*
 * endpoints/locales.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
pub struct TranslateInput {
    locales: Vec<String>,
    messages: HashMap<String, MessageArguments<'static>>,
}

type TranslateOutput = HashMap<String, Option<String>>;

pub async fn locale_info(
    _ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<LocaleOutput> {
    let locale_str: String = params.one()?;
    info!("Getting locale information for {locale_str}");
    let locale = LanguageIdentifier::from_bytes(locale_str.as_bytes())?;
    Ok(LocaleOutput {
        language: str!(locale.language),
        script: locale.script.map(|s| str!(s)),
        region: locale.region.map(|s| str!(s)),
        variants: locale.variants().map(|v| str!(v)).collect(),
    })
}

pub async fn translate_strings(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<TranslateOutput> {
    let TranslateInput { locales, messages } = params.parse()?;

    if locales.is_empty() {
        error!("No locales specified in translate call");
        return Err(ServiceError::NoLocalesSpecified);
    }

    info!(
        "Translating {} message keys in locale {} (or {} fallbacks)",
        messages.len(),
        &locales[0],
        locales.len() - 1,
    );

    let mut output: TranslateOutput = HashMap::new();
    let locales = {
        let mut langids = Vec::new();
        for locale in locales {
            let langid = LanguageIdentifier::from_bytes(locale.as_bytes())?;
            langids.push(langid);
        }
        langids
    };

    for (message_key, arguments_raw) in messages {
        info!(
            "Formatting message key {message_key} ({} arguments)",
            arguments_raw.len(),
        );

        let arguments = arguments_raw.into_fluent_args();
        let translation =
            ctx.localization()
                .translate_option(&locales, &message_key, &arguments)?;

        output.insert(message_key, translation.map(|t| t.to_string()));
    }

    Ok(output)
}
