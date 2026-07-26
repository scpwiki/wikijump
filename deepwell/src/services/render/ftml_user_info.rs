/*
 * services/render/ftml_user_info.rs
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::ServiceContext;
use crate::services::page_query::normalize_wikidot_author_name;
use ftml::data::{KarmaLevel, UserInfo};
use ftml::render::UserInfoResolver;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub(super) struct UserInfoSnapshot {
    users: BTreeMap<String, UserInfo<'static>>,
}

impl UserInfoSnapshot {
    pub(super) async fn load(ctx: &ServiceContext<'_>, names: &[String]) -> Result<Self> {
        let slugs = names
            .iter()
            .map(|name| normalize_wikidot_author_name(name))
            .filter(|slug| !slug.is_empty())
            .collect::<BTreeSet<_>>();
        if slugs.is_empty() {
            return Ok(Self::default());
        }

        let users = WikidotUser::find()
            .filter(wikidot_user::Column::Slug.is_in(slugs))
            .filter(wikidot_user::Column::IsDeleted.eq(false))
            .all(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "failed to resolve Wikidot users for FTML render",
                    ErrorType::Render,
                )
            })?;

        Ok(Self {
            users: users
                .into_iter()
                .filter_map(|user| {
                    let slug = user.slug?;
                    let name = user.name.unwrap_or_else(|| slug.clone());
                    let user_id = i64::from(user.user_id);
                    let karma = u8::try_from(user.karma)
                        .ok()
                        .and_then(KarmaLevel::new)
                        .unwrap_or(KarmaLevel::Zero);
                    let info = UserInfo {
                        user_id,
                        user_slug: Cow::Owned(slug.clone()),
                        user_name: Cow::Owned(name),
                        user_karma: karma,
                        user_avatar_data: Cow::Owned(format!(
                            "http://www.wikidot.com/avatar.php?userid={user_id}&amp;size=small"
                        )),
                        user_profile_url: Cow::Owned(format!(
                            "http://www.wikidot.com/user:info/{slug}"
                        )),
                    };
                    Some((slug, info))
                })
                .collect(),
        })
    }
}

impl UserInfoResolver for UserInfoSnapshot {
    fn user_info(&self, name: &str) -> Option<UserInfo<'static>> {
        self.users
            .get(&normalize_wikidot_author_name(name))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_resolves_case_and_spacing_to_canonical_user() {
        let canonical = UserInfo {
            user_id: 122357,
            user_slug: Cow::Borrowed("system"),
            user_name: Cow::Borrowed("system"),
            user_karma: KarmaLevel::Five,
            user_avatar_data: Cow::Borrowed(
                "http://www.wikidot.com/avatar.php?userid=122357&amp;size=small",
            ),
            user_profile_url: Cow::Borrowed("http://www.wikidot.com/user:info/system"),
        };
        let snapshot = UserInfoSnapshot {
            users: BTreeMap::from([("system".to_owned(), canonical.clone())]),
        };

        assert_eq!(snapshot.user_info(" SYSTEM "), Some(canonical));
        assert!(snapshot.user_info("unknown").is_none());
    }
}
