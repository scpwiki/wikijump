/*
 * includes/test.rs
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

use super::{include, DebugIncluder, PageRef};

#[test]
fn includes() {
    let log = crate::build_logger();

    macro_rules! test {
        ($text:expr, $expected:expr $(,)?) => {{
            let mut text = str!($text);
            let result = include(&log, &mut text, DebugIncluder, || panic!());
            let (output, actual) = result.expect("Fetching pages failed");
            let expected = $expected;

            println!("Input:  '{}'", $text);
            println!("Output: '{}'", output);
            println!("Pages (actual):   {:?}", actual);
            println!("Pages (expected): {:?}", expected);
            println!();

            assert_eq!(
                &actual, &expected,
                "Actual pages to include doesn't match expected"
            );
        }};
    }

    // Valid cases

    test!("", vec![]);
    test!("[[include-messy page]]", vec![PageRef::page_only("page")]);
    test!("[[include-messy page ]]", vec![PageRef::page_only("page")]);
    test!("[[include-messy page ]]", vec![PageRef::page_only("page")]);
    test!("[[ include-messy page ]]", vec![PageRef::page_only("page")]);

    test!("[[include-messy PAGE]]", vec![PageRef::page_only("PAGE")]);
    test!("[[include-messy PAGE ]]", vec![PageRef::page_only("PAGE")]);
    test!("[[include-messy PAGE ]]", vec![PageRef::page_only("PAGE")]);
    test!("[[ include-messy PAGE ]]", vec![PageRef::page_only("PAGE")]);

    // Arguments
    test!("[[include-messy apple a =1]]", vec![PageRef::page_only("apple")]);
    test!("[[include-messy apple a= 1]]", vec![PageRef::page_only("apple")]);
    test!("[[include-messy apple a = 1]]", vec![PageRef::page_only("apple")]);
    test!(
        "[[include-messy apple a = 1 ]]",
        vec![PageRef::page_only("apple")]
    );
    test!(
        "[[include-messy apple  a = 1 ]]",
        vec![PageRef::page_only("apple")]
    );

    test!("[[include-messy banana a=1]]", vec![PageRef::page_only("banana")]);
    test!(
        "[[include-messy banana a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana |a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana | a=1]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana |a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana | a=1|]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana |a=1 |]]",
        vec![PageRef::page_only("banana")],
    );
    test!(
        "[[include-messy banana | a=1 |]]",
        vec![PageRef::page_only("banana")],
    );

    test!(
        "[[include-messy cherry a=1|b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry a=1|b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry a=1 |b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry |a=1|b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry | a=1| b=2]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry |a=1|b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry | a=1| b=2|]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry |a=1 |b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );
    test!(
        "[[include-messy cherry | a=1 | b=2 |]]",
        vec![PageRef::page_only("cherry")],
    );

    test!(
        "[[include-messy durian a=1|b=2|C=**]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian a=1|b=2|C=**|]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian a=1 |b=2 |C=** |]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian |a=1|b=2|C=**]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian | a=1| b=2| C=**]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian |a=1|b=2|C=**|]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian | a=1| b=2| C=**|]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian |a=1 |b=2 |C=** |]]",
        vec![PageRef::page_only("durian")],
    );
    test!(
        "[[include-messy durian | a=1 | b=2 | C=** ]]",
        vec![PageRef::page_only("durian")],
    );

    // Off-site includes
    test!(
        "[[include-messy component:my-thing]]",
        vec![PageRef::page_only("component:my-thing")],
    );
    test!(
        "[[include-messy :scp-wiki:main]]",
        vec![PageRef::page_and_site("scp-wiki", "main")],
    );
    test!(
        "[[include-messy :scp-wiki:component:my-thing]]",
        vec![PageRef::page_and_site("scp-wiki", "component:my-thing")],
    );
    test!(
        "[[include-messy :scp-wiki:deleted:protected:component:magic]]",
        vec![PageRef::page_and_site(
            "scp-wiki",
            "deleted:protected:component:magic"
        )],
    );

    // Multiple include-messys
    test!(
        "A\n[[include-messy B]]\nC\n[[include-messy D]]\nE\n[[include-messy F]]\nG",
        vec![
            PageRef::page_only("B"),
            PageRef::page_only("D"),
            PageRef::page_only("F"),
        ],
    );
    test!(
        "[[include-messy my-page]]\n[[include-messy :scp-wiki:theme:black-highlighter-theme]]\n",
        vec![
            PageRef::page_only("my-page"),
            PageRef::page_and_site("scp-wiki", "theme:black-highlighter-theme"),
        ],
    );

    // Multi-line includes
    test!("[[include-messy page\n]]", vec![PageRef::page_only("page")]);
    test!(
        "[[include-messy component:multi-line | contents= \nSome content here \nMore stuff]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "[[include-messy component:multi-line argument=x | contents= \nSome content here \nMore stuff \n|]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "[[include-messy component:multi-line | contents= \nSome content here\nMore stuff\n]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "[[include-messy component:multi-line | contents=\nSome content here\nMore stuff\n]]",
        vec![PageRef::page_only("component:multi-line")],
    );
    test!(
        "My wonderful page!\n\n[[include-messy component:info-ayers\n\tlang=en |\n\tpage=scp-xxxx |\n\tauthorPage=http://scpwiki.com/main |\n\tcomments=\n**SCP-XXXX:** My amazing skip \n**Author:** [[*user Username]] \n]]",
        vec![PageRef::page_only("component:info-ayers")],
    );
    test!(
        "My other wonderful page!\n\n[[include-messy component:info-ayers\n\t|lang=en\n\t|page=scp-xxxx\n\t|authorPage=http://scpwiki.com/main\n\t|comments=\n**SCP-XXXX:** My amazing skip \n**Author:** [[*user Username]] \n]]",
        vec![PageRef::page_only("component:info-ayers")],
    );

    // Invalid cases

    test!("other text", vec![]);
    test!("include-messy]]", vec![]);
    test!("[[include-messy", vec![]);
    test!("[[include-messy]]", vec![]);
    test!("[[include-messy ]]", vec![]);
    test!("[[ include-messy]]", vec![]);

    test!(
        "[[include-messy component:multi-line | contents= \nSome content here \nMore stuff",
        vec![],
    );
}
