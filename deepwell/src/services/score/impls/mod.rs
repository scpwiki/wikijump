/*
 * services/score/impls/mod.rs
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

// TEMP
#![allow(dead_code)]

use crate::error::prelude::{Error, ErrorType};

fn make_error(method: &str) -> Error {
    Error::new(
        format!("failed to calculate {} score for page", method),
        ErrorType::PageVote,
    )
}

mod mean;
mod null;
mod percent;
mod sum;
mod test;

pub use self::mean::MeanScorer;
pub use self::null::NullScorer;
pub use self::percent::PercentScorer;
pub use self::sum::SumScorer;
pub use self::test::TestScorer;

#[cfg(test)]
mod tests {
    use super::super::Scorer;
    use super::super::structs::{ScoreType, VoteType};
    use super::*;

    #[test]
    fn scorer_metadata_matches_supported_vote_types() {
        let mean = MeanScorer;
        assert_eq!(mean.score_type(), ScoreType::Mean);
        assert!(mean.accepts_vote_type(VoteType::UpsDowns));
        assert!(mean.accepts_vote_type(VoteType::FiveStar));

        let null = NullScorer;
        assert_eq!(null.score_type(), ScoreType::Null);
        assert!(null.accepts_vote_type(VoteType::UpsDowns));
        assert!(null.accepts_vote_type(VoteType::FiveStar));

        let percent = PercentScorer;
        assert_eq!(percent.score_type(), ScoreType::Percent);
        assert!(percent.accepts_vote_type(VoteType::UpsDowns));
        assert!(!percent.accepts_vote_type(VoteType::FiveStar));

        let sum = SumScorer;
        assert_eq!(sum.score_type(), ScoreType::Sum);
        assert!(sum.accepts_vote_type(VoteType::UpsDowns));
        assert!(sum.accepts_vote_type(VoteType::FiveStar));

        let test = TestScorer;
        assert_eq!(test.score_type(), ScoreType::Test);
        assert!(test.accepts_vote_type(VoteType::UpsDowns));
        assert!(test.accepts_vote_type(VoteType::FiveStar));
    }

    #[test]
    fn scorer_error_includes_method_name() {
        let error = make_error("mean");

        assert!(error.to_string().contains("mean score"));
    }
}
