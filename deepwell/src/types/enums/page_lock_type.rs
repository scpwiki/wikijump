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
    // DeriveValueType delegates database strings to Display and FromStr.
    // Accept known historical spellings while keeping writes canonical.
    #[strum(
        serialize = "author_only",
        serialize = "author-only",
        serialize = "author_or_permission_only",
        to_string = "author-or-permission-only"
    )]
    AuthorOrPermissionOnly,
}

#[cfg(test)]
mod tests {
    use super::PageLockType;
    use sea_orm::{Value, sea_query::ValueType};

    const AUTHOR_LOCK: PageLockType = PageLockType::AuthorOrPermissionOnly;

    #[test]
    fn every_page_lock_type_round_trips_through_json_and_database_values() {
        for (lock_type, canonical) in [
            (PageLockType::Wikidot, "wikidot"),
            (PageLockType::PermissionOnly, "permission-only"),
            (AUTHOR_LOCK, "author-or-permission-only"),
        ] {
            let json =
                serde_json::to_string(&lock_type).expect("serialize page lock JSON");
            assert_eq!(json, format!(r#""{canonical}""#));
            assert_eq!(
                serde_json::from_str::<PageLockType>(&json)
                    .expect("deserialize canonical page lock JSON"),
                lock_type,
            );

            let value: Value = lock_type.into();
            assert_eq!(value, Value::String(Some(canonical.to_owned())),);
            assert_eq!(
                <PageLockType as ValueType>::try_from(value)
                    .expect("round-trip canonical page lock database value"),
                lock_type,
            );
        }
    }

    #[test]
    fn author_only_database_value_compatibility() {
        let canonical_value: Value = AUTHOR_LOCK.into();
        assert_eq!(
            canonical_value,
            Value::String(Some("author-or-permission-only".to_owned())),
        );
        assert_eq!(
            <PageLockType as ValueType>::try_from(canonical_value)
                .expect("Unable to round-trip the canonical database value"),
            AUTHOR_LOCK,
        );

        for compatible_value in [
            "author_only",
            "author-only",
            "author_or_permission_only",
            "AUTHOR-ONLY",
        ] {
            assert_eq!(
                <PageLockType as ValueType>::try_from(Value::String(Some(
                    compatible_value.to_owned(),
                )))
                .expect("Unable to read a compatible database value"),
                AUTHOR_LOCK,
            );
        }

        for invalid_value in ["author", "permission_only"] {
            assert!(
                <PageLockType as ValueType>::try_from(Value::String(Some(
                    invalid_value.to_owned(),
                )))
                .is_err()
            );
        }
        assert!(<PageLockType as ValueType>::try_from(Value::String(None)).is_err());
        assert!(<PageLockType as ValueType>::try_from(Value::Int(Some(1))).is_err());
    }

    #[test]
    fn author_only_json_compatibility() {
        for value in [r#""author-only""#, r#""author-or-permission-only""#] {
            assert_eq!(
                serde_json::from_str::<PageLockType>(value)
                    .expect("Unable to deserialize compatible JSON"),
                AUTHOR_LOCK,
            );
        }
        assert_eq!(
            serde_json::to_string(&AUTHOR_LOCK)
                .expect("Unable to serialize canonical JSON"),
            r#""author-or-permission-only""#,
        );

        for value in [r#""author_only""#, r#""author_or_permission_only""#] {
            assert!(serde_json::from_str::<PageLockType>(value).is_err());
        }
    }
}
