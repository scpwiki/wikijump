/*
 * services/score/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

use super::prelude::*;
use crate::models::page_vote::{self, Entity as PageVote, Model as PageVoteModel};

#[derive(Debug)]
pub struct ScoreService;

impl ScoreService {
    // TODO
    pub async fn collect_votes(ctx: &ServiceContext<'_>, page_id: i64) -> Result<Vec<PageVoteModel>> {
        let txn = ctx.transaction();

        let votes = PageVote::find()
            .filter(
                Condition::all()
                    .add(page_vote::Column::PageId.eq(page_id))
                    .add(page_vote::Column::DeletedAt.is_null())
                    .add(page_vote::Column::DisabledAt.is_null())
            )
            .all(txn)
            .await?;

        Ok(votes)
    }
}
