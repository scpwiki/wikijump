/*
 * endpoints/locales.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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
use crate::utils::{parse_locales, strip_fluent_control_chars, validate_locale};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Debug, Clone)]
pub struct LocaleOutput {
    pub language: String,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TranslateInput {
    locales: Vec<String>,
    messages: HashMap<String, MessageArguments<'static>>,

    /// A list of message keys to run `strip_fluent_control_chars()` on.
    ///
    /// For each of the keys here, the translated message has its Fluent-added
    /// control characters stripped before it is returned in the response.
    ///
    /// By default this is empty, meaning to leave all messages unmodified.
    ///
    /// # Errors
    /// If there are any keys in this list which are not in `messages`, then
    /// an error will be returned.
    #[serde(default)]
    strip_message_keys: HashSet<String>,
}

type TranslateOutput = HashMap<String, Option<String>>;

pub async fn locale_info(
    _ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<LocaleOutput> {
    let locale_str: String = parse_one!(params, Localization);

    let locale = validate_locale(&locale_str).or_raise(|| {
        Error::new(
            "failed to parse locale string",
            ErrorType::LocaleInvalid {
                locale: str!(locale_str),
            },
        )
    })?;

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
        locales: locales_str,
        messages,
        strip_message_keys,
    } = parse!(params, Localization);

    // Check that locales are specified
    if locales_str.is_empty() {
        error!("No locales specified in translate call");
        bail!(Error::new(
            "failed to translate with no locales",
            ErrorType::NoLocalesSpecified,
        ));
    }

    // Check that all message keys to strip are being requested
    for message_key in &strip_message_keys {
        if !messages.contains_key(message_key.as_str()) {
            bail!(Error::new(
                format!(
                    "invalid argument: cannot strip control characters from message '{}' when it is not requested to be translated, for locales {:?}",
                    message_key, locales_str,
                ),
                ErrorType::BadRequest,
            ));
        }
    }

    info!(
        "Translating {} message keys in locale {} (or {} fallbacks)",
        messages.len(),
        locales_str[0],
        locales_str.len() - 1,
    );
    debug!("Message keys to translate: {messages:?}");

    let mut output: TranslateOutput = HashMap::new();
    let locales = parse_locales(&locales_str)?;

    for (message_key, arguments_raw) in messages {
        trace!(
            "Formatting message key {message_key} ({} arguments)",
            arguments_raw.len(),
        );

        let arguments = arguments_raw.into_fluent_args();
        let translation = ctx
            .localization()
            .translate_option(&locales, &message_key, &arguments)
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get translation for message '{}' with locales {:?}",
                        message_key, locales_str,
                    ),
                    ErrorType::Localization,
                )
            })?
            .map(|translation| {
                let mut translation = translation.to_string();

                if strip_message_keys.contains(&message_key) {
                    strip_fluent_control_chars(&mut translation);
                }

                translation
            });

        output.insert(message_key, translation);
    }

    Ok(output)
}
