/*
 * services/score/structs.rs
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

use std::collections::BTreeMap;

pub use crate::services::vote::VoteValue;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VoteType {
    UpsDowns,
    FiveStar,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScoreType {
    Null,
    Test,
    Sum,
    Mean,
    Median,
    Percent,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VoteMap {
    inner: BTreeMap<VoteValue, u64>,
}

impl VoteMap {
    #[inline]
    pub fn new() -> Self {
        VoteMap::default()
    }

    #[inline]
    pub fn insert(&mut self, vote: VoteValue, count: u64) {
        self.inner.insert(vote, count);
    }

    #[inline]
    pub fn get(&self, vote: VoteValue) -> u64 {
        self.inner.get(&vote).copied().unwrap_or(0)
    }

    /// Gets the number of votes in this map.
    pub fn count(&self) -> u64 {
        self.iter().fold(0, |sum, (_, count)| sum + count)
    }

    #[inline]
    #[allow(dead_code)] // TEMP
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Gets the sum of all the votes in this map.
    #[allow(dead_code)] // TEMP
    pub fn sum(&self) -> i64 {
        self.iter().fold(0, |sum, (value, count)| {
            let value = i64::from(value);
            let count = count as i64;

            sum + value * count
        })
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (VoteValue, u64)> + '_ {
        // We can't quite use .copied() here because we need to copy the tuple too
        self.inner.iter().map(|(&value, &count)| (value, count))
    }
}
