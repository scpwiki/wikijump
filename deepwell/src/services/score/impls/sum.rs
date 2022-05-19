/*
 * services/score/impls/sum.rs
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
pub struct SumScorer;

#[async_trait]
impl Scorer for SumScorer {
    #[inline]
    fn score_type(&self) -> ScoreType {
        ScoreType::Sum
    }

    #[inline]
    fn accepts_vote_type(&self, _: VoteType) -> bool {
        true
    }

    async fn score(
        &self,
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<f64> {
        #[derive(FromQueryResult, Debug)]
        struct SumRow {
            sum: u64,
        }

        // Query for sum of all votes.
        //
        // As raw SQL:
        //
        // SELECT SUM(value)
        // FROM page_vote
        // WHERE page_id = $1
        // AND deleted_at IS NULL
        // AND disabled_at IS NULL
        // GROUP BY value;

        let result = PageVote::find()
            .column_as(Expr::col(page_vote::Column::Value).sum(), "sum")
            .filter(condition)
            .into_model::<SumRow>()
            .one(txn)
            .await?
            .expect("No results in aggregate query");

        Ok(result.sum as f64)
    }
}
