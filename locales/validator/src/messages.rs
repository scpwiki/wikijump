/*
 * messages.rs
 *
 * wikijump-locales-validator - Validate Wikijump's Fluent localization files
 * Copyright (C) 2021 Wikijump Team
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

use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

/// The "primary" locale, to compare other locales against.
///
/// This is defined as one which is always complete, containing
/// every message key used by the application.
///
/// Thus, we can compare all other locales to it, ensuring they
/// are equal or subsets, raising errors on any new message keys,
/// as they are either typos or removed keys.
const PRIMARY_LOCALE: LanguageIdentifier = langid!("en");

#[derive(Debug, Default, Clone)]
pub struct Catalog {
    primary: Messages,
    locales: HashMap<LanguageIdentifier, Messages>,
}

impl Catalog {}

#[derive(Debug, Default, Clone)]
pub struct Messages {
    messages: HashMap<String, MessageUsages>,
}

#[derive(Debug, Default, Clone)]
pub struct MessageUsages {
    functions: Vec<String>,
    messages: Vec<String>,
    terms: Vec<String>,
    variables: Vec<String>,
}
