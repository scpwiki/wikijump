/*
 * locales/error.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use std::fmt::Debug;
use std::io;
use thiserror::Error as ThisError;
use unic_langid::LanguageIdentifierError;

#[derive(ThisError, Debug)]
pub enum LocalizationLoadError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Language identifier error: {0}")]
    LangId(#[from] LanguageIdentifierError),

    #[error("Error loading fluent resources")]
    Fluent,
}

/// Creates a dummy Fluent error type from the input.
///
/// Because many of the `Err(_)` outputs for Fluent functions
/// are not `std::error::Error`, and this all happens at
/// load time where we bail if there's an issue anyways,
/// this simply logs whatever we get and then returns the
/// generic `LocalizationLoadError::Fluent` error variant.
pub fn fluent_load_err<T: Debug>(item: T) -> LocalizationLoadError {
    tide::log::error!("Fluent error: {:#?}", item);

    LocalizationLoadError::Fluent
}

#[derive(ThisError, Debug)]
pub enum LocalizationTranslateError {
    #[error("No messages are available for this locale")]
    NoLocale,

    #[error("Message key not found for this locale")]
    NoMessage,

    #[error("Message key was found, but has no value")]
    NoMessageValue,
}
