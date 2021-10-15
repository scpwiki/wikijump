/*
 * test/settings.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use crate::data::PageInfo;
use crate::render::{html::HtmlRender, Render};
use crate::settings::{WikitextMode, WikitextSettings};

#[test]
fn settings() {
    const PAGE_MODES: [WikitextMode; 5] = [
        WikitextMode::Page,
        WikitextMode::Draft,
        WikitextMode::ForumPost,
        WikitextMode::DirectMessage,
        WikitextMode::List,
    ];

    let log = &crate::build_logger();
    let page_info = PageInfo::dummy();

    macro_rules! check_individual {
        ($mode:expr, $input:expr, $substring:expr, $contains:expr) => {{
            let settings = WikitextSettings::from_mode($mode);
            let mut text = str!($input);
            crate::preprocess(log, &mut text);

            let tokens = crate::tokenize(log, &text);
            let result = crate::parse(log, &tokens, &page_info, &settings);
            let (tree, _warnings) = result.into();
            let html_output = HtmlRender.render(log, &tree, &page_info, &settings);

            println!();
            println!("Input:  {:?}", $input);
            println!("Output: {:?}", html_output.body);

            assert_eq!(
                html_output.body.contains($substring),
                $contains,
                "For {:?}, HTML expected {} the expected substring {:?}",
                $mode,
                if $contains {
                    "to contain"
                } else {
                    "to not contain"
                },
                $substring,
            );
        }};
    }

    macro_rules! check {
        ($input:expr, $substring:expr, $contains:expr $(,)?) => {{
            for (&mode, &contains) in PAGE_MODES.iter().zip($contains.iter()) {
                check_individual!(mode, $input, $substring, contains);
            }
        }};
    }

    check!("++ H2", "toc0", [true, true, false, false, false]);
    check!("[[toc]]", "wj-toc", [true, true, false, false, false]);
    check!(
        "[[module Rate]]",
        "TODO: module Rate",
        [true, true, false, false, true],
    );
    check!(
        "[[include-elements page]]",
        "INCLUDED PAGE",
        [true, true, false, false, true],
    );
}
