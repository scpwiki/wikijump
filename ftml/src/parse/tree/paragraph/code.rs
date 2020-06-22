/*
 * parse/tree/paragraph/code.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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

lazy_static! {
    static ref CODE_BLOCK: Regex = {
        RegexBuilder::new(
            r"(?x)
            \[\[\s*code[^\]]*\]\]\n
            (?P<contents>(?:.*\n)?)
            \[\[/\s*code\s*\]\]",
        )
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
    };
}

pub fn parse(pair: Pair<Rule>) -> Result<Paragraph> {
    let mut language = None;
    let contents = extract!(CODE_BLOCK, pair);

    // Parse arguments
    let pairs = pair
        .into_inner()
        .filter(|pair| pair.as_rule() == Rule::code_arg);

    for pair in pairs {
        let capture = ARGUMENT_NAME
            .captures(pair.as_str())
            .expect("Regular expression ARGUMENT_NAME didn't match");
        let key = capture!(capture, "name");
        let value_pair = get_first_pair!(pair);

        debug_assert_eq!(value_pair.as_rule(), Rule::string);

        let value = value_pair.as_str();

        #[derive(Debug, Copy, Clone)]
        enum Argument {
            Language,
        }

        fn get_argument(key: &str) -> Argument {
            const CODE_ARGUMENTS: [(&str, Argument); 3] = [
                ("type", Argument::Language),
                ("lang", Argument::Language),
                ("language", Argument::Language),
            ];

            for (name, argument) in &CODE_ARGUMENTS {
                if name.eq_ignore_ascii_case(key) {
                    return *argument;
                }
            }

            panic!("Unknown argument for [[code]]: {}", key);
        }

        match get_argument(key) {
            Argument::Language => language = interp_str(value).ok(),
        }
    }

    Ok(Paragraph::CodeBlock { language, contents })
}

#[test]
fn test_regexes() {
    let _ = &*CODE_BLOCK;
}
