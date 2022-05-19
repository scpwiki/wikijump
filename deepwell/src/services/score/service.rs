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

#[derive(Debug)]
pub struct ScoreService;

impl ScoreService {
    pub async fn score(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        scorer: &impl Scorer,
    ) -> Result<f64> {
        let txn = ctx.transaction();
        let condition = Self::build_condition(page_id);
        let score = scorer.score(txn, condition).await?;
        Ok(score)
    }

    #[inline]
    pub async fn collect_votes(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<VoteMap> {
        Self::collect_votes_inner(ctx.transaction(), Self::build_condition(page_id)).await
    }

    pub(crate) async fn collect_votes_inner(
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
            .column_as(Expr::col(page_vote::Column::Value).count(), "count")
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
