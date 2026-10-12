/*
 * services/import/structs.rs
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

use crate::types::PageRevisionType;
use ftml::data::KarmaLevel;
use std::net::IpAddr;
use time::{Date, OffsetDateTime};

#[derive(Deserialize, Debug)]
pub struct ImportUser {
    // Required fields

    /// The user ID of the Wikidot user.
    pub user_id: i32,

    /// The datetime when the Wikidot user was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// The datetime when this Wikidot user was fetched.
    ///
    /// The database asserts that this date is strictly later than `created_at`,
    /// since it's not possible to fetch data about a user that has not yet been
    /// created. (And especially for these old users, it's been a decade plus
    /// since the user was created.)
    #[serde(with = "time::serde::rfc3339")]
    pub fetched_at: OffsetDateTime,

    /// The kind of Wikidot user.
    ///
    /// This is how we represent an extant user (which has a name and slug),
    /// or a deleted user (which doesn't usually have a known username).
    #[serde(flatten)]
    pub wikidot_user_type: ImportedUserType,

    /// The blob ID of the Wikidot user's avatar.
    ///
    /// This is created as normal with `blob_upload`.
    pub avatar_uploaded_blob_id: Option<String>,

    // Biographical fields
    pub real_name: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<Date>,
    pub location: Option<String>,
    pub biography: Option<String>,
    pub website: Option<String>,
    pub karma: KarmaLevel,
    pub is_pro: bool,

    // Request metadata

    /// If a `wikidot_user` row with this ID exists, then update its fields.
    #[serde(default)]
    pub upsert: bool,

    /// Mark an old `wikidot_user` row as `deleted` if it has a name collision.
    ///
    /// In the case where attempting to import this user fails due to a non-unique
    /// name or slug, but we find that the colliding user has an older `fetched_at`
    /// value, this indicates that this Wikidot user has since been deleted.
    ///
    /// In such a case, if this option is set, then the user is marked as `deleted`
    /// and we proceed with the new user insert. Otherwise, we raise an error.
    #[serde(default)]
    pub delete_old_if_same_name: bool,

    /// The user ID to associate with this request in the audit log.
    ///
    /// Note that this must be the same user who uploaded the avatar blob.
    pub importing_user_id: i64,

    /// The IP address to associate with this request in the audit log.
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "user_type", rename_all = "kebab-case")]
pub enum ImportedUserType {
    Extant { name: String, slug: String },
    Deleted,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct ImportUserOutput {
    pub user_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct ImportSite {
    pub site_id: i64,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub name: String,
    pub slug: String,
    pub locale: String,
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct ImportSiteOutput {
    pub site_id: i64,
}

#[derive(Deserialize, Debug)]
pub struct ImportPage {
    pub page_id: i64,
    pub site_id: i64,

    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub slug: String,
    pub locked: bool,
    pub discussion_thread_id: Option<i64>,
    pub ip_address: IpAddr,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct ImportPageOutput {
    pub site_id: i64,
    pub page_id: i64,
}

#[derive(Deserialize, Debug)]
pub struct ImportPageRevision {
    pub revision_id: i64,
    pub revision_type: PageRevisionType,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
    pub revision_number: i32,
    pub page_id: i64,
    pub site_id: i64,
    pub user_id: i64,
    pub wikitext: String,
    pub comments: String,
    pub title: String,
    pub slug: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct ImportPageRevisionOutput {
    pub site_id: i64,
    pub page_id: i64,
    pub page_revision_id: i64,
    pub page_revision_number: i32,
}
