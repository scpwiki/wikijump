/*
 * services/email/structs.rs
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

use serde::{Deserialize, Serialize};

/// A deserialized response from the MailCheck API.
///
/// Describes all the fields received from the API, but not all fields are used.
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct MailCheckResponse {
    pub status: u16,
    pub email: String,
    pub domain: String,
    pub mx: bool,
    pub disposable: bool,
    pub alias: bool,
    pub did_you_mean: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EmailValidationOutput {
    pub valid: bool,
    pub classification: EmailClassification,
    pub did_you_mean: Option<String>,
}

impl Default for EmailValidationOutput {
    fn default() -> Self {
        EmailValidationOutput {
            valid: true,
            classification: EmailClassification::Normal,
            did_you_mean: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum EmailClassification {
    Normal,
    Disposable,
    Alias,
    Invalid,
}
