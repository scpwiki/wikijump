/*
 * parse/tree/mod.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

// FIXME to prevent compile spam
#![allow(dead_code)]

macro_rules! capture {
    ($capture:expr, $name:expr) => ( $capture.name($name).unwrap().as_str() )
}

mod line;
mod misc;
mod word;

mod prelude {
    lazy_static! {
        pub static ref ARGUMENT_NAME: Regex = Regex::new(r"(?P<name>\w+)=").unwrap();
    }

    pub use pest::iterators::{Pair, Pairs};
    pub use regex::{Regex, RegexBuilder};
    pub use super::{Line, LineInner, TableRow, Word};
    pub use super::super::Rule;
    pub use super::super::string::interp_str;
}

pub use self::line::{Line, LineInner};
pub use self::misc::TableRow;
pub use self::word::Word;

use self::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree<'a> {
    lines: Vec<Line<'a>>,
}

impl<'a> SyntaxTree<'a> {
    pub fn from_line_pairs(pairs: Pairs<'a, Rule>) -> Self {
        trace!("Converting pairs into a SyntaxTree...");

        let lines = pairs
            .into_iter()
            .filter(|pair| pair.as_rule() == Rule::line)
            .map(|pair| Line::from_pair(pair))
            .collect();

        SyntaxTree { lines }
    }
}
