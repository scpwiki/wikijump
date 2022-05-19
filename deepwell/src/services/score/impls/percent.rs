/*
 * services/score/impls/percent.rs
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

    fn score(&self, votes: &VoteMap) -> f64 {
        votes.get(1) / votes.count()
    }
}
