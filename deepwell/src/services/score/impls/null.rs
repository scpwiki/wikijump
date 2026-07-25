/*
 * services/score/impls/null.rs
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
use crate::error::prelude::Result;
use ftml::data::ScoreValue;
use sea_orm::{Condition, DatabaseTransaction};

#[derive(Debug)]
pub struct NullScorer;

impl Scorer for NullScorer {
    #[inline]
    fn score_type(&self) -> ScoreType {
        ScoreType::Null
    }

    #[inline]
    fn accepts_vote_type(&self, _: VoteType) -> bool {
        true
    }

    #[inline]
    async fn score(&self, _: &DatabaseTransaction, _: Condition) -> Result<ScoreValue> {
        Ok(ScoreValue::Integer(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::score::Scorer;

    #[test]
    fn null_scorer_metadata_accepts_all_vote_styles() {
        let scorer = NullScorer;

        assert_eq!(Scorer::score_type(&scorer), ScoreType::Null);
        assert!(Scorer::accepts_vote_type(&scorer, VoteType::UpsDowns));
        assert!(Scorer::accepts_vote_type(&scorer, VoteType::FiveStar));
    }
}
