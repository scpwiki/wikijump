/*
 * types/enums/relation.rs
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
pub enum RelationType {
    SiteUser,
    #[strum(to_string = "ban", serialize = "site-ban")]
    SiteBan,
    #[allow(dead_code)] // TEMP
    #[strum(to_string = "application", serialize = "site-application")]
    SiteApplication,
    #[strum(to_string = "member", serialize = "site-member")]
    SiteMember,
    #[strum(to_string = "star", serialize = "page-star")]
    PageStar,
    #[strum(to_string = "watch", serialize = "page-watch")]
    PageWatch,
    PageAttribution,
    #[strum(to_string = "follow", serialize = "user-follow")]
    UserFollow,
    #[allow(dead_code)] // TEMP
    #[strum(to_string = "contact", serialize = "user-contact")]
    UserContact,
    #[allow(dead_code)] // TEMP
    #[strum(to_string = "contact-request", serialize = "user-contact-request")]
    UserContactRequest,
    #[strum(to_string = "block", serialize = "user-block")]
    UserBlock,
    #[strum(to_string = "bot-owner", serialize = "user-bot-owner")]
    UserBotOwner,
}

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
pub enum RelationObjectType {
    Site,
    User,
    Page,
    File,
}

#[cfg(test)]
mod tests {
    use super::RelationType;
    use std::str::FromStr;

    #[test]
    fn relation_type_display_keeps_legacy_database_values() {
        let cases = [
            (RelationType::SiteUser, "site-user"),
            (RelationType::SiteBan, "ban"),
            (RelationType::SiteApplication, "application"),
            (RelationType::SiteMember, "member"),
            (RelationType::PageStar, "star"),
            (RelationType::PageWatch, "watch"),
            (RelationType::PageAttribution, "page-attribution"),
            (RelationType::UserFollow, "follow"),
            (RelationType::UserContact, "contact"),
            (RelationType::UserContactRequest, "contact-request"),
            (RelationType::UserBlock, "block"),
            (RelationType::UserBotOwner, "bot-owner"),
        ];

        for (relation_type, database_value) in cases {
            assert_eq!(relation_type.to_string(), database_value);
        }
    }

    #[test]
    fn relation_type_parses_legacy_and_variant_database_values() {
        let cases = [
            (RelationType::SiteUser, "site-user", "site-user"),
            (RelationType::SiteBan, "ban", "site-ban"),
            (
                RelationType::SiteApplication,
                "application",
                "site-application",
            ),
            (RelationType::SiteMember, "member", "site-member"),
            (RelationType::PageStar, "star", "page-star"),
            (RelationType::PageWatch, "watch", "page-watch"),
            (
                RelationType::PageAttribution,
                "page-attribution",
                "page-attribution",
            ),
            (RelationType::UserFollow, "follow", "user-follow"),
            (RelationType::UserContact, "contact", "user-contact"),
            (
                RelationType::UserContactRequest,
                "contact-request",
                "user-contact-request",
            ),
            (RelationType::UserBlock, "block", "user-block"),
            (RelationType::UserBotOwner, "bot-owner", "user-bot-owner"),
        ];

        for (relation_type, legacy_value, variant_value) in cases {
            assert_eq!(RelationType::from_str(legacy_value).unwrap(), relation_type);
            assert_eq!(
                RelationType::from_str(variant_value).unwrap(),
                relation_type
            );
        }

        assert!(RelationType::from_str("user").is_err());
    }
}
