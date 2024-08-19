/*
 * services/score/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use super::impls::*;
use super::prelude::*;

#[derive(Debug)]
pub struct ScoreService;

impl ScoreService {
    pub async fn score(ctx: &ServiceContext, page_id: i64) -> Result<ScoreValue> {
        let txn = ctx.seaorm_transaction();
        let condition = Self::build_condition(page_id);
        let scorer = Self::get_scorer(ctx, page_id).await?;
        let score = scorer.score(txn, condition).await?;
        Ok(score)
    }

    /// Gets the correct `Scorer` implementation for this page.
    ///
    /// Currently stubbed, will be implemented when relevant settings are added.
    pub async fn get_scorer(
        _ctx: &ServiceContext,
        _page_id: i64,
    ) -> Result<&'static impl Scorer> {
        // TODO
        Ok(&NullScorer)
    }

    /// Helper method for retrieving a `VoteMap` for a page.
    /// Takes inputs as used in `Scorer.score()`.
    ///
    /// This can become a full service method, see above.
    pub(crate) async fn collect_votes(
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<VoteMap> {
        // Query for votes aggregated by value.
        //
        // As raw SQL:
        //
        // SELECT value, COUNT(value)
        // FROM page_vote
        // WHERE page_id = $1
        // AND deleted_at IS NULL
        // AND disabled_at IS NULL
        // GROUP BY value;

        #[derive(FromQueryResult, Debug)]
        struct VoteCountRow {
            value: VoteValue,
            count: u64,
        }

        let counts = PageVote::find()
            .column(page_vote::Column::Value)
            .column_as(page_vote::Column::Value.count(), "count")
            .filter(condition)
            .group_by(page_vote::Column::Value)
            .into_model::<VoteCountRow>()
            .all(txn)
            .await?;

        // Gather results into map
        let mut map = VoteMap::new();

        for VoteCountRow { value, count } in counts {
            map.insert(value, count);
        }

        Ok(map)
    }

    fn build_condition(page_id: i64) -> Condition {
        Condition::all()
            .add(page_vote::Column::PageId.eq(page_id))
            .add(page_vote::Column::DeletedAt.is_null())
            .add(page_vote::Column::DisabledAt.is_null())
    }
}
