/*
 * services/score/impls/mean.rs
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
pub struct MeanScorer;

#[async_trait]
impl Scorer for MeanScorer {
    #[inline]
    fn score_type(&self) -> ScoreType {
        ScoreType::Mean
    }

    fn accepts_vote_type(&self, vote_type: VoteType) -> bool {
        match vote_type {
            VoteType::UpsDowns | VoteType::FiveStar => true,
        }
    }

    async fn score(
        &self,
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<f64> {
        #[derive(FromQueryResult, Debug)]
        struct MeanRow {
            sum: u64,
            count: u64,
        }

        // Query for sum of all votes.
        //
        // As raw SQL:
        //
        // SELECT SUM(value), COUNT(value)
        // FROM page_vote
        // WHERE page_id = $1
        // AND deleted_at IS NULL
        // AND disabled_at IS NULL
        // GROUP BY value;

        let MeanRow { sum, count } = PageVote::find()
            .column_as(Expr::col(page_vote::Column::Value).sum(), "sum")
            .column_as(Expr::col(page_vote::Column::Value).count(), "count")
            .filter(condition)
            .into_model::<MeanRow>()
            .one(txn)
            .await?
            .expect("No results in aggregate query");

        if count == 0 {
            Ok(0.0)
        } else {
            Ok((sum / count) as f64)
        }
    }
}
