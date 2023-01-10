/*
 * data/score.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2023 Wikijump Team
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

/// Represents the score on a page.
///
/// This is a generic numeric value, either being an integer or a
/// floating-point number depending on the context.
///
/// Which type is used depends on the scoring algorithm used for this page,
/// something that is configurable.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ScoreValue {
    Integer(i64),
    Float(f64),
}

impl ScoreValue {
    #[inline]
    pub fn to_f64(self) -> f64 {
        match self {
            ScoreValue::Integer(value) => value as f64,
            ScoreValue::Float(value) => value,
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
