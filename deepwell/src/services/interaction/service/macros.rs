/*
 * services/interaction/service/macros.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

macro_rules! site {
    ($id:expr $(,)?) => {
        InteractionObject::Site($id)
    };
}

macro_rules! user {
    ($id:expr $(,)?) => {
        InteractionObject::User($id)
    };
}

macro_rules! page {
    ($id:expr $(,)?) => {
        InteractionObject::Page($id)
    };
}

// Adding an "i" (for "interaction") because file!() itself conflicts with logging.
macro_rules! ifile {
    ($id:expr $(,)?) => {
        InteractionObject::File($id)
    };
}
