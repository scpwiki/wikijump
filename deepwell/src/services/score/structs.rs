/*
 * services/score/structs.rs
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

use std::cmp::Ordering;
use std::collections::HashMap;

pub use crate::services::vote::VoteValue;

pub type VoteMap = HashMap<VoteValue, u64>;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(untagged)]
pub enum ScoreValue {
    Integer(i64),
    Float(f64),
}

impl ScoreValue {
    #[inline]
    pub fn as_f64(self) -> f64 {
        self.into()
    }
}

impl From<ScoreValue> for f64 {
    fn from(value: ScoreValue) -> f64 {
        match value {
            ScoreValue::Integer(n) => n as f64,
            ScoreValue::Float(n) => n,
        }
    }
}

impl From<i64> for ScoreValue {
    #[inline]
    fn from(value: i64) -> ScoreValue {
        ScoreValue::Integer(value)
    }
}

impl From<f64> for ScoreValue {
    #[inline]
    fn from(value: f64) -> ScoreValue {
        ScoreValue::Float(value)
    }
}

impl PartialEq for ScoreValue {
    #[inline]
    fn eq(&self, other: &ScoreValue) -> bool {
        self.as_f64() == other.as_f64()
    }
}

impl PartialOrd for ScoreValue {
    #[inline]
    fn partial_cmp(&self, other: &ScoreValue) -> Option<Ordering> {
        let x = self.as_f64();
        let y = other.as_f64();

        x.partial_cmp(&y)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScoreType {
    Null,
    Sum,
    Percent,
    Wilson,
}
