/*
 * models/user.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use chrono::NaiveDateTime;

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub slug: String,
    pub username_changes: i16,
    pub email: String,
    pub email_verified_at: NaiveDateTime,
    pub remember_token: String,
    pub language: String,
    pub karma_points: i32,
    pub karma_level: i16,
    pub real_name: String,
    pub pronouns: String,
    pub dob: NaiveDateTime,
    pub bio: String,
    pub about_page: String,
    pub avatar_path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: NaiveDateTime,
}
