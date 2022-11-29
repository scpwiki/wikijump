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

#[async_trait]
pub trait Scorer {
    /// What kind of score this scorer evaluates.
    ///
    /// There should be a 1-to-1 mapping between `Scorer`
    /// implementations and values for the `ScoreType` enum.
    fn score_type(&self) -> ScoreType;

    /// Whether this scorer accepts vote maps of this type.
    fn accepts_vote_type(&self, vtype: VoteType) -> bool;

    /// Calculates the score associated with the given page ID.
    ///
    /// This is the primary method for calculating the score for a page.
    /// The method is given access to the database, as opposed to reading from
    /// a premade `VoteMap` as a performance consideration.
    ///
    /// The process for collecting votes by group and then later iterating
    /// to sum is less efficient than having the database doing the summing,
    /// for instance. Similarly, the `NullScorer` does not need to go to the
    /// database at all.
    ///
    /// In order to ensure the query is formed correctly, the `Condition` for
    /// querying active votes for a page is passed rather than the page ID.
    /// For reference: `page_id = $1 AND disabled_at IS NULL AND deleted_at IS NULL`.
    async fn score(
        &self,
        txn: &DatabaseTransaction,
        condition: Condition,
    ) -> Result<ScoreValue>;
}
