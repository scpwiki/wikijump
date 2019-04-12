/*
 * parse/test.rs
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

use pest::Parser;
use super::{Rule, WikidotParser};

#[test]
fn test_strings() {
    const INPUT_STRINGS: [&str; 22] = [
        "@@ apple @@ @@banana@@",
        "@@ [!-- literal comment @@ durian",
        "@@@@@@ at signs `````` tildes",
        "apple `` legacy raw @@ `` banana",
        "[!-- [[ footnote invalid formatting in here-- [[ eref --] test",
        "__**test** cherry {{ durian (?) }}__ ^^up!^^",
        "** [[date 0]] **",
        "__ [[  date 0  ]] [!-- comment here --]__",
        "[[span class = \"test\"]]//hello// world![[footnote]]actually country[[/footnote]][[/span]]",
        "--[[*user rounderhouse]] [[# test-anchor ]]-- [[ eref equation_id ]]",
        "[[ image tree.png link = \"https://example.com\" alt=\"A tree.\" class=\"image-block\"  ]]",
        "__**--^^,,{{super formatted}},,^^--**__",
        "//[[date -100]] number// [[footnote]]Content **cherry** [[*user aismallard]][[/footnote]] [[footnote]]Content **cherry** [[*user aismallard]][[/footnote]]",
        "apple\n[[module Rate]]\nbanana",
        "apple\n[[module CSS]]\n@import url('https://example.com/style.css');\ndiv.container { display: none; }\n[[/module]]\nbanana",
        "[[form]]\nform data here\nmore stuff\n[[/form]]",
        "[[note]]\nnote internal information here\napple\ndurian\nbanana\n[[/note]]",
        "apple\n[[note]]\ninternal\n[[/note]]\nbanana",
        "^^**alpha** beta ,,gama,,^^",
        "= {{apple}} banana",
        "++ header\n+++ apple __banana__\n++++ @@ RAW @@\ndurian",
        "internal [[# anchor-name]] [[date 1000]] **apple** _",
    ];

    for string in &INPUT_STRINGS[..] {
        println!("Testing parser: {:?}", string);
        if let Err(err) = WikidotParser::parse(Rule::page, string) {
            panic!("Failed to parse test string:\n{}\nProduced error: {}", string, err);
        }
    }
}
