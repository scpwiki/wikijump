/*
 * types/id.rs
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

use crate::models::audit_log::Model as AuditEventModel;
use crate::models::file::Model as FileModel;
use crate::models::file_revision::Model as FileRevisionModel;
use crate::models::filter::Model as FilterModel;
use crate::models::page::Model as PageModel;
use crate::models::page_category::Model as PageCategoryModel;
use crate::models::page_lock::Model as PageLockModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::page_vote::Model as PageVoteModel;
use crate::models::relation::Model as RelationModel;
use crate::models::site::Model as SiteModel;
use crate::models::user::Model as UserModel;
use std::fmt::{self, Display};

macro_rules! define_id_type {
    // TEMP: Before forum schema is added
    ($name:ident) => {
        #[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $name(pub i64);

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };

    ($name:ident, $model:ident, $key:ident) => {
        define_id_type!($name)

        impl From<$name> for sea_orm::Value {
            #[inline]
            fn from($name(value): $name) -> sea_orm::Value {
                sea_orm::Value::BigInt(Some(value))
            }
        }

        impl From<&$model> for $name {
            #[inline]
            fn from(model: &$model) -> $name {
                model.$key
            }
        }
    };
}

// Base
define_id_type!(UserId, UserModel, user_id);
define_id_type!(SiteId, SiteModel, site_id);

// Pages
define_id_type!(PageId, PageModel, page_id);
define_id_type!(PageCategoryId, PageCategoryModel, category_id);
define_id_type!(PageRevisionId, PageRevisionModel, revision_id);
define_id_type!(PageLockId, PageLockModel, lock_id);
define_id_type!(PageVoteId, PageVoteModel, vote_id);

// Files
define_id_type!(FileId, FileModel, file_id);
define_id_type!(FileRevisionId, FileRevisionModel, revision_id);

// Forum
define_id_type!(ForumGroupId);
define_id_type!(ForumCategoryId);
define_id_type!(ForumThreadId);
define_id_type!(ForumPostId);
define_id_type!(ForumPostRevisionId);

// Miscellaneous
define_id_type!(FilterId, FilterModel, filter_id);
define_id_type!(RelationId, RelationModel, relation_id);
define_id_type!(AuditEventId, AuditEventModel, entry_id);
