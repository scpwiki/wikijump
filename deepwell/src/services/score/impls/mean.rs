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

use super::prelude::*;
use crate::services::ScoreService;

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
        let votes = ScoreService::collect_votes(txn, condition)
            .await
            .or_raise(|| make_error("mean"))?;

        let count = votes.count();
        if count == 0 {
            return Ok(ScoreValue::Float(0.0));
        }

        let count = count as f64;
        let sum = votes.sum() as f64;
        Ok(ScoreValue::Float(sum / count))
    }
}
