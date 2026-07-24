/*
 * types/enums/page_revision.rs
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
pub enum PageRevisionType {
    Regular,
    Rollback,
    Undo,
    Create,
    Delete,
    Undelete,
    Move,
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
pub enum PageRevisionChange {
    Wikitext,
    Title,
    AltTitle,
    Slug,
    Tags,
}

impl PageRevisionChange {
    pub const fn database_value(self) -> &'static str {
        match self {
            Self::Wikitext => "wikitext",
            Self::Title => "title",
            Self::AltTitle => "alt_title",
            Self::Slug => "slug",
            Self::Tags => "tags",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_revision_change_database_values_match_the_schema() {
        assert_eq!(PageRevisionChange::Wikitext.database_value(), "wikitext");
        assert_eq!(PageRevisionChange::Title.database_value(), "title");
        assert_eq!(PageRevisionChange::AltTitle.database_value(), "alt_title");
        assert_eq!(PageRevisionChange::Slug.database_value(), "slug");
        assert_eq!(PageRevisionChange::Tags.database_value(), "tags");
    }

    #[test]
    fn page_revision_change_json_keeps_the_public_kebab_case_contract() {
        assert_eq!(
            serde_json::to_string(&PageRevisionChange::AltTitle).unwrap(),
            r#""alt-title""#,
        );
    }
}
