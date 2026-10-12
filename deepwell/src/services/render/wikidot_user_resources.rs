/*
 * services/render/wikidot_user_resources.rs
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

//! Wikidot's own user-resource URLs in rendered output.
//!
//! Wikidot links each user it renders to `www.wikidot.com/user:info/<slug>`
//! and draws the avatar and karma badge from its avatar base. Since
//! 2026-09-21 both are https. In the shared corpus every scp-wiki compile up
//! to 2026-09-21T15:57:59Z links `http://www.wikidot.com/user:info/` and
//! every compile from 2026-09-21T21:45:52Z on links `https://` (1,013 avatar
//! printusers, no exception); live thread views and forum pages after that
//! date do the same. The avatar base is a serve-time value (the
//! `(%%%URL_AVATARS%%%)` placeholder in Wikidot's compiled HTML) that Wikidot
//! serves over https.
//!
//! Wikijump renders a page or module when Wikidot would compile or serve it
//! now, so it uses these URLs. Content Wikidot compiled before the change,
//! such as an imported forum post body, keeps its observed links.

/// Prefix of a Wikidot user's profile link, followed by the user slug.
pub(crate) const WIKIDOT_USER_INFO_URL: &str = "https://www.wikidot.com/user:info/";

/// Base of Wikidot's `avatar.php` and `userkarma.php` images.
pub(crate) const WIKIDOT_AVATAR_BASE: &str = "https://www.wikidot.com";
