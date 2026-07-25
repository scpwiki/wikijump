/*
 * services/score/impls/mean.rs
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

use super::super::scorer::Scorer;
use super::super::structs::{ScoreType, VoteType};
use super::make_error;
use crate::error::prelude::{Result, ResultExt};
use crate::models::page_vote::{self, Entity as PageVote};
use ftml::data::ScoreValue;
use sea_orm::{
    ColumnTrait, Condition, DatabaseTransaction, EntityTrait, FromQueryResult,
    QueryFilter, QuerySelect,
};

#[derive(Debug)]
pub struct MeanScorer;

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
    ) -> Result<ScoreValue> {
        #[derive(FromQueryResult, Debug)]
        struct MeanRow {
            sum: Option<i64>,
            count: i64,
        }

        // Query for sum of all votes.
        // Same as in sum.rs
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
            .select_only()
            .column_as(page_vote::Column::Value.sum(), "sum")
            .column_as(page_vote::Column::Value.count(), "count")
            .filter(condition)
            .into_model::<MeanRow>()
            .one(txn)
            .await
            .or_raise(|| make_error("mean"))?
            .expect("No results in aggregate query");

        if count == 0 {
            return Ok(ScoreValue::Float(0.0));
        }
        Ok(ScoreValue::Float(sum.unwrap_or(0) as f64 / count as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::score::Scorer;

    #[test]
    fn mean_scorer_metadata_accepts_all_vote_styles() {
        let scorer = MeanScorer;

        assert_eq!(Scorer::score_type(&scorer), ScoreType::Mean);
        assert!(Scorer::accepts_vote_type(&scorer, VoteType::UpsDowns));
        assert!(Scorer::accepts_vote_type(&scorer, VoteType::FiveStar));
    }
}
