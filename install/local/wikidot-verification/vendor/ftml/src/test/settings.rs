/*
 * test/settings.rs
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

use crate::data::PageInfo;
use crate::layout::Layout;
use crate::render::{Render, html::HtmlRender};
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

    let page_info = PageInfo::dummy();

    macro_rules! test_individual {
        ($mode:expr, $input:expr, $substring:expr, $contains:expr) => {{
            let settings = WikitextSettings::from_mode($mode, Layout::Wikidot);
            let mut text = str!($input);
            crate::preprocess(&mut text);

            let tokens = crate::tokenize(&text);
            let result = crate::parse(&tokens, &page_info, &settings);
            let (tree, _errors) = result.into();
            let html_output = HtmlRender.render(&tree, &page_info, &settings);

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

    macro_rules! test {
        ($input:expr, $substring:expr, $contains:expr $(,)?) => {{
            for (&mode, &contains) in PAGE_MODES.iter().zip($contains.iter()) {
                test_individual!(mode, $input, $substring, contains);
            }
        }};
    }

    test!("++ H2", "toc0", [true, false, false, false, false]);
    test!("[[toc]]", "wj-toc", [true, false, false, false, false]);
    test!(
        "[[module Rate]]",
        "TODO: module Rate",
        [true, true, false, false, true],
    );
    test!(
        "[[include-elements page]]",
        "INCLUDED PAGE",
        [true, true, false, false, true],
    );
    test!(
        "[[image /local-file.png]]",
        "local-file.png",
        [true, true, false, false, true],
    );
    test!(
        "[[image /some-page/local-file.png]]",
        "local-file.png",
        [true, true, false, false, true],
    );
    test!(
        "[[image /my-site/some-page/local-file.png]]",
        "local-file.png",
        [true, true, false, false, true],
    );
}
