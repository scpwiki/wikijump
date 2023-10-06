/*
 * services/message/structs.rs
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageDraft {
    pub user_id: i64,
    pub recipients: Vec<i64>,
    pub carbon_copy: Vec<i64>,
    pub blind_carbon_copy: Vec<i64>,
    pub subject: String,
    pub wikitext: String,
    pub reply_to: Option<String>,
    pub forwarded_from: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageDraft {
    pub message_draft_id: String,
    pub recipients: Vec<i64>,
    pub carbon_copy: Vec<i64>,
    pub blind_carbon_copy: Vec<i64>,
    pub subject: String,
    pub wikitext: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DraftRecipients {
    #[serde(rename = "r")]
    pub recipients: Vec<i64>,

    #[serde(rename = "cc")]
    pub carbon_copy: Vec<i64>,

    #[serde(rename = "bcc")]
    pub blind_carbon_copy: Vec<i64>,
}

impl DraftRecipients {
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        let i1 = self.recipients.iter().copied();
        let i2 = self.carbon_copy.iter().copied();
        let i3 = self.blind_carbon_copy.iter().copied();

        i1.chain(i2).chain(i3)
    }

    pub fn len(&self) -> usize {
        self.recipients.len() + self.carbon_copy.len() + self.blind_carbon_copy.len()
    }
}
