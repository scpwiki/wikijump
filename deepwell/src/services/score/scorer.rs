/*
 * services/score/scorer.rs
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

pub trait Scorer {
    /// What kind of score this scorer evaluates.
    ///
    /// There should be a 1-to-1 mapping between `Scorer`
    /// implementations and values for the `ScoreType` enum.
    fn score_type(&self) -> ScoreType;

    /// Whether this scorer accepts vote maps of this type.
    fn accepts_vote_type(&self, vtype: VoteType) -> bool;

    /// Takes the given `VoteMap` and produces a float score.
    /// This method is the core of the trait.
    fn score(&self, votes: &VoteMap) -> f64;
}
