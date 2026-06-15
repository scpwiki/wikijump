/*
 * tokenizer.rs
 *
 * ftml - Library to parse Wikidot text
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

use crate::parsing::{ExtractedToken, Token};
use crate::text::FullText;

/// Struct that represents both a list of tokens and the text the tokens were generated from.
#[derive(Debug, Clone)]
pub struct Tokenization<'t> {
    tokens: Vec<ExtractedToken<'t>>,
    full_text: FullText<'t>,
}

impl<'t> Tokenization<'t> {
    #[inline]
    pub fn tokens<'r>(&'r self) -> &'r [ExtractedToken<'t>] {
        &self.tokens
    }

    #[inline]
    pub(crate) fn full_text(&self) -> FullText<'t> {
        self.full_text
    }
}

impl<'t> From<Tokenization<'t>> for Vec<ExtractedToken<'t>> {
    #[inline]
    fn from(tokenization: Tokenization<'t>) -> Vec<ExtractedToken<'t>> {
        tokenization.tokens
    }
}

/// Take an input string and produce a list of tokens for consumption by the parser.
pub fn tokenize(text: &str) -> Tokenization<'_> {
    info!(
        "Running lexer on text ({} bytes) to produce tokens",
        text.len(),
    );

    let tokens = Token::extract_all(text);
    let full_text = FullText::new(text);

    Tokenization { tokens, full_text }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4096))]

        #[test]
        #[ignore = "slow test"]
        fn tokenizer_prop(s in ".*") {
            let _ = tokenize(&s);
        }
    }
}
