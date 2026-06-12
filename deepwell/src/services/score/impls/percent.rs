/*
 * services/score/impls/percent.rs
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
pub struct PercentScorer;

impl Scorer for PercentScorer {
    #[inline]
    fn score_type(&self) -> ScoreType {
        ScoreType::Percent
    }

    fn accepts_vote_type(&self, vote_type: VoteType) -> bool {
        match vote_type {
            VoteType::UpsDowns => true,
            VoteType::FiveStar => false,
        }
    }

    async fn score(
        &self,
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<ScoreValue> {
        // We need to do a GROUP BY either way here,
        // may as well use the helper method.
        let votes = ScoreService::collect_votes(txn, condition)
            .await
            .or_raise(|| make_error("percent"))?;

        let total = votes.count();
        if total == 0 {
            return Ok(ScoreValue::Float(0.0));
        }

        let upvotes = votes.get(1) as f64;
        let total = total as f64;
        let percent = upvotes / total * 100.0;
        Ok(ScoreValue::Float(percent))
    }
}
