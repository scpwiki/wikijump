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

macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $name(pub i64);
    };
}

// Base
define_id_type!(UserId);
define_id_type!(SiteId);

// Pages
define_id_type!(PageId);
define_id_type!(PageCategoryId);
define_id_type!(PageRevisionId);
define_id_type!(PageLockId);
define_id_type!(PageVoteId);

// Files
define_id_type!(FileId);
define_id_type!(FileRevisionId);

// Forum
define_id_type!(ForumGroupId);
define_id_type!(ForumCategoryId);
define_id_type!(ForumThreadId);
define_id_type!(ForumPostId);
define_id_type!(ForumPostRevisionId);

// Miscellaneous
define_id_type!(FilterId);
define_id_type!(RelationId);
define_id_type!(AuditEventId);
