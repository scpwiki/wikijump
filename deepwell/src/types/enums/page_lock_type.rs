/*
 * types/enums/page_lock_type.rs
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

use sea_orm::DeriveValueType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(
    EnumIter,
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    Hash,
    PartialEq,
    Eq,
    DeriveValueType,
    EnumString,
    Display,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum PageLockType {
    // Only mods+ can edit, legacy lock type
    Wikidot,
    // Only users with Page:BypassLock permission can edit
    PermissionOnly,
    // Authors and users with Page:BypassLock can edit
    #[serde(alias = "author-only")]
    #[strum(serialize = "author-only", serialize = "author-or-permission-only")]
    AuthorOrPermissionOnly,
}

/// Ensure the renamed author-or-permission lock type remains compatible with
/// rows and clients that still use the previous `author-only` value.
#[test]
fn author_only_compatibility() {
    assert_eq!(
        serde_json::from_str::<PageLockType>(r#""author-only""#)
            .expect("Unable to deserialize legacy author-only JSON"),
        PageLockType::AuthorOrPermissionOnly,
    );
    assert_eq!(
        "author-only"
            .parse::<PageLockType>()
            .expect("Unable to parse legacy author-only database value"),
        PageLockType::AuthorOrPermissionOnly,
    );
    assert_eq!(
        PageLockType::AuthorOrPermissionOnly.to_string(),
        "author-or-permission-only",
    );
}
