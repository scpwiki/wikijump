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
#[serde(rename_all = "camelCase")]
pub struct SendMessageDraft {
    pub message_draft_id: String,
}

pub type DeleteMessageDraft = SendMessageDraft;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DraftRecipients {
    #[serde(rename = "r")]
    pub regular: Vec<i64>,

    #[serde(rename = "cc")]
    pub carbon_copy: Vec<i64>,

    #[serde(rename = "bcc")]
    pub blind_carbon_copy: Vec<i64>,
}

impl DraftRecipients {
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        let i1 = self.regular.iter().copied();
        let i2 = self.carbon_copy.iter().copied();
        let i3 = self.blind_carbon_copy.iter().copied();

        i1.chain(i2).chain(i3)
    }

    pub fn len(&self) -> usize {
        self.regular.len() + self.carbon_copy.len() + self.blind_carbon_copy.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regular.is_empty()
            && self.carbon_copy.is_empty()
            && self.blind_carbon_copy.is_empty()
    }

    /// Determines if the recipient list is composed only of the given user ID.
    ///
    /// This accounts for duplicate entries, even across multiple types of recipients.
    /// Returns `true` for empty recipient lists.
    pub fn only_has(&self, user_id: i64) -> bool {
        self.iter().filter(|id| *id != user_id).count() == 0
    }
}

#[test]
fn recipients() {
    let recipients = DraftRecipients {
        regular: vec![10, 20, 30],
        carbon_copy: vec![20, 80],
        blind_carbon_copy: vec![70],
    };

    assert_eq!(
        recipients.iter().collect::<Vec<_>>(),
        vec![10, 20, 30, 20, 80, 70],
        "Recipient iterator does not match expected",
    );
    assert_eq!(
        recipients.len(),
        6,
        "Recipient length does not match expected",
    );
    assert!(!recipients.is_empty(), "Recipient is_empty reports true");
    assert!(!recipients.only_has(10), "Recipient only_has reports true");
}

#[test]
fn recipients_empty() {
    let recipients = DraftRecipients {
        regular: vec![],
        carbon_copy: vec![],
        blind_carbon_copy: vec![],
    };

    assert_eq!(
        recipients.iter().collect::<Vec<_>>(),
        Vec::<i64>::new(),
        "Recipient iterator does not match expected",
    );
    assert_eq!(
        recipients.len(),
        0,
        "Recipient length does not match expected",
    );
    assert!(recipients.is_empty(), "Recipient is_empty reports false");
}

#[test]
fn recipients_only() {
    let recipients = DraftRecipients {
        regular: vec![],
        carbon_copy: vec![1],
        blind_carbon_copy: vec![1],
    };

    assert_eq!(
        recipients.iter().collect::<Vec<_>>(),
        vec![1, 1],
        "Recipient iterator does not match expected",
    );
    assert_eq!(
        recipients.len(),
        2,
        "Recipient length does not match expected",
    );
    assert!(!recipients.is_empty(), "Recipient is_empty reports true");
    assert!(recipients.only_has(1), "Recipient only_has reports false");
}
