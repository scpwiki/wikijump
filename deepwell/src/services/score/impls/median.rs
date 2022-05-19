/*
 * services/score/impls/median.rs
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
pub struct MedianScorer;

impl Scorer for MedianScorer {
    #[inline]
    fn score_type(&self) -> ScoreType {
        ScoreType::Median
    }

    fn score(&self, votes: &VoteMap) -> f64 {
        // Finds the median by iterating through elements
        // until it finds one which is before than the target
        // value, but stepping to the next count would pass it.
        //
        // Remember, a vote map is a collection of values and how
        // many instances of that value are present. It is not a list
        // of all votes, but rather can be thought of as being made
        // of "blocks" of various sizes.
        //
        // The target value here is the count divided by two, since that
        // is where the median would be, if we imagine the vote map
        // as a block of length N (where N is the sum) where each vote
        // is 1 unit wide, and each corresponding vote value has a size
        // of x units based on how many votes have that value.

        let half = votes.count_int() / 2;
        let mut progress = 0;

        for (vote, count) in votes.iter() {
            progress += count;

            if progress > half {
                return f64::from(vote);
            }
        }

        // Default case
        //
        // This should only happen if no votes have been cast,
        // that is "votes" is empty.

        0.0
    }
}

#[test]
fn median() {
    // Tests to ensure our median algorithm works, see above.

    macro_rules! votes {
        () => {
            VoteMap::new()
        };

        ($($key:expr => $value:expr,)+) => {
            votes!($($key => $value),+)
        };

        ($($key:expr => $value:expr),*) => {{
            let mut votes = VoteMap::new();

            $(
                // $key   -- Vote value
                // $value -- Count
                votes.insert($key, $value);
            )*

            votes
        }};
    }

    macro_rules! check {
        ($expected:expr, $votes:expr $(,)?) => {{
            let votes = $votes;
            let actual = MedianScorer.score(&votes);
            let expected = $expected as f64;

            assert_eq!(
                actual, expected,
                "Actual median score doesn't match expected",
            );
        }};
    }

    check!(0, votes! {});
    check!(0, votes! { 1 => 0 });
    check!(0, votes! { 0 => 0, 1 => 0 });

    check!(1, votes! { 1 => 1 });
    check!(1, votes! { 1 => 5 });
    check!(0, votes! { 0 => 5 });
    check!(-1, votes! { -1 => 5 });

    check!(0, votes! { -1 => 1, 0 => 1, 1 => 1 });
    check!(-1, votes! { -1 => 4, 0 => 1, 1 => 1 });
    check!(1, votes! { -1 => 1, 0 => 1, 1 => 4 });
}
