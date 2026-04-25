/*
 * services/score/mod.rs
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

#[allow(unused_imports)]
mod prelude {
    pub use super::{super::prelude::*, Scorer, structs::*};
    pub use crate::models::page_vote::{self, Entity as PageVote};
    pub use ftml::data::ScoreValue;
    pub use sea_orm::{DatabaseTransaction, FromQueryResult};
    pub use std::future::Future;
}

mod impls;
mod scorer;
mod service;
mod structs;

pub use self::{impls::*, scorer::Scorer, service::ScoreService};
pub use ftml::data::ScoreValue;
