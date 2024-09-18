/*
 * services/score/impls/test.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct TestScorer;

#[async_trait]
impl Scorer for TestScorer {
    #[inline]
    fn score_type(&self) -> ScoreType {
        ScoreType::Test
    }

    #[inline]
    fn accepts_vote_type(&self, _: VoteType) -> bool {
        true
    }

    #[inline]
    async fn score(&self, _: &DatabaseTransaction, _: Condition) -> Result<ScoreValue> {
        let mut rng = thread_rng();
        let value = rng.gen_range(-100..100);
        Ok(ScoreValue::Integer(value))
    }
}
