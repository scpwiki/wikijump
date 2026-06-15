/*
 * includes/test.rs
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

use super::{DebugIncluder, PageRef, include};
use crate::layout::Layout;
use crate::settings::{WikitextMode, WikitextSettings};

#[test]
fn includes() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);

    macro_rules! test {
        ($text:expr, $expected:expr $(,)?) => {{
            let mut text = str!($text);
            let result = include(&mut text, &settings, DebugIncluder, || panic!());
            let (output, actual) = result.expect("Fetching pages failed");
            let expected = $expected;

            println!("Input:  '{}'", $text);
            println!("Output: '{}'", &output);
            println!("Pages (actual):   {:?}", &actual);
            println!("Pages (expected): {:?}", &expected);
            println!();

            assert_eq!(
                &actual, &expected,
                "Actual pages to include doesn't match expected"
            );
        }};
    }

    // Valid cases

    test!("", vec![]);
    test!("[[include page]]", vec![PageRef::page_only("page")]);
    test!("[[include page ]]", vec![PageRef::page_only("page")]);
    test!("[[include page ]]", vec![PageRef::page_only("page")]);
    test!("[[ include page ]]", vec![PageRef::page_only("page")]);
    test!("[[include page |]]", vec![PageRef::page_only("page")]);
    test!("[[include page | ]]", vec![PageRef::page_only("page")]);
    test!("[[include page ||]]", vec![PageRef::page_only("page")]);
    test!("[[include page || ]]", vec![PageRef::page_only("page")]);

    test!("[[include PAGE]]", vec![PageRef::page_only("PAGE")]);
    test!("[[include PAGE ]]", vec![PageRef::page_only("PAGE")]);
    test!("[[include PAGE ]]", vec![PageRef::page_only("PAGE")]);
    test!("[[ include PAGE ]]", vec![PageRef::page_only("PAGE")]);

    // Arguments
    test!("[[include apple a =1]]", vec![PageRef::page_only("apple")]);
    test!("[[include apple a= 1]]", vec![PageRef::page_only("apple")]);
    test!("[[include apple a = 1]]", vec![PageRef::page_only("apple")]);
    test!(
        "[[include apple a = 1 ]]",
        vec![PageRef::page_only("apple")],
    );
    test!(
        "[[include apple  a = 1 ]]",
        vec![PageRef::page_only("apple")],
    );

    test!("[[include banana a=1]]", vec![PageRef::page_only("banana")]);
    test!(
        "[[include banana a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1||]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1| |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1|||]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1| |  |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1 ||]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana a=1 | |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana |a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana ||a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana | |a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana | a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana || a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana |a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana ||a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana | a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana || a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana |a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana |a=1 ||]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana ||a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana ||a=1 ||]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana | a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana | a=1 ||]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana || a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include banana || a=1 ||]]",
        vec![PageRef::page_only("banana")],
    );

    test!(
        "[[include cherry a=1|b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1||b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1|b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1|b=2||]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1||b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1||b=2||]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1 |b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1 |b=2 ||]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry a=1 ||b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1|b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1||b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry ||a=1|b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry | a=1| b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry | a=1|| b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry || a=1| b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1|b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1|b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1|b=2||]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1||b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry ||a=1|b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry ||a=1||b=2||]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry | a=1| b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry |a=1 |b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include cherry | a=1 | b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );

    test!(
        "[[include durian a=1|b=2|C=**]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian a=1|b=2|C=**|]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian a=1 |b=2 |C=** |]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian |a=1|b=2|C=**]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian | a=1| b=2| C=**]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian |a=1|b=2|C=**|]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian | a=1| b=2| C=**|]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian |a=1 |b=2 |C=** |]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include durian | a=1 | b=2 | C=** ]]",
        vec![PageRef::page_only("durian")],
    );

    // Off-site includes
    test!(
        "[[include component:my-thing]]",
        vec![PageRef::page_only("component:my-thing")],
    );
    test!(
        "[[include :scp-wiki:main]]",
        vec![PageRef::page_and_site("scp-wiki", "main")],
    );
    test!(
        "[[include :scp-wiki:component:my-thing]]",
        vec![PageRef::page_and_site("scp-wiki", "component:my-thing")],
    );
    test!(
        "[[include :scp-wiki:deleted:protected:component:magic]]",
        vec![PageRef::page_and_site(
            "scp-wiki",
            "deleted:protected:component:magic"
        )],
    );

    // Multiple includes
    test!(
        "A\n[[include B]]\nC\n[[include D]]\nE\n[[include F]]\nG",
        vec![
            PageRef::page_only("B"),
            PageRef::page_only("D"),
            PageRef::page_only("F"),
        ],
    );
    test!(
        "[[include my-page]]\n[[include :scp-wiki:theme:black-highlighter-theme]]\n",
        vec![
            PageRef::page_only("my-page"),
            PageRef::page_and_site("scp-wiki", "theme:black-highlighter-theme"),
        ],
    );

    // Multi-line includes
    test!("[[include page\n]]", vec![PageRef::page_only("page")]);
    test!(
        "[[include component:multi-line | contents= \nSome content here \nMore stuff]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "[[include component:multi-line argument=x | contents= \nSome content here \nMore stuff \n|]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "[[include component:multi-line | contents= \nSome content here\nMore stuff\n]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "[[include component:multi-line | contents=\nSome content here\nMore stuff\n]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "My wonderful page!\n\n[[include component:info-ayers\n\tlang=en |\n\tpage=scp-xxxx |\n\tauthorPage=http://scpwiki.com/main |\n\tcomments=\n**SCP-XXXX:** My amazing skip \n**Author:** [[*user Username]] \n]]",
        vec![PageRef::page_only("component:info-ayers")],
    );
    test!(
        "My other wonderful page!\n\n[[include component:info-ayers\n\t|lang=en\n\t|page=scp-xxxx\n\t|authorPage=http://scpwiki.com/main\n\t|comments=\n**SCP-XXXX:** My amazing skip \n**Author:** [[*user Username]] \n]]",
        vec![PageRef::page_only("component:info-ayers")],
    );

    // Invalid cases

    test!("other text", vec![]);
    test!("include]]", vec![]);
    test!("[[include", vec![]);
    test!("[[include]]", vec![]);
    test!("[[include ]]", vec![]);
    test!("[[ include]]", vec![]);

    test!(
        "[[include component:multi-line | contents= \nSome content here \nMore stuff",
        vec![],
    );
}
