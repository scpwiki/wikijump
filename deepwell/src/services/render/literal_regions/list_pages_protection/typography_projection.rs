/*
 * services/render/literal_regions/list_pages_protection/typography_projection.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use regex::Regex;
use std::{ops::Range, sync::LazyLock};

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static UTF8_BYTES_VALIDATED: Cell<usize> = const { Cell::new(0) };
}

// The pinned tokenizer classifies `?` as Other, its URL and email scans accept it like the Unicode typography replacements, and no recognized multi-byte delimiter contains it, so it cannot combine with neighboring source bytes into syntax.
const INERT: u8 = b'?';

// Keep these expressions and their order in lockstep with FTML 4fc7df28's typography preprocessor. The equal-width inert bytes preserve original offsets while having the same scanner-relevant property as the Unicode replacements: they are not ASCII quotes, dots, whitespace, or Wikidot delimiters.
static DOUBLE_QUOTES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"``(.*?)''").unwrap());
static LOW_DOUBLE_QUOTES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r",,(.*?)''").unwrap());
static SINGLE_QUOTES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`(.*?)'").unwrap());
static HORIZONTAL_ELLIPSIS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^\.])(?<repl>(\.\.|\. \. )\.)(?:[^\.]|$)").unwrap()
});

pub(super) fn project_typography_in_place(source: &mut [u8]) -> bool {
    let mut changed = false;
    changed |= project_surround(source, &DOUBLE_QUOTES);
    changed |= project_surround(source, &LOW_DOUBLE_QUOTES);
    changed |= project_surround(source, &SINGLE_QUOTES);
    changed |= project_replacement(source, &HORIZONTAL_ELLIPSIS);
    changed
}

fn project_surround(source: &mut [u8], regex: &Regex) -> bool {
    let ranges = surround_matches(projected_str(source), regex);
    for (opening, closing) in ranges.iter().cloned() {
        source[opening].fill(INERT);
        source[closing].fill(INERT);
    }
    !ranges.is_empty()
}

fn surround_matches(source: &str, regex: &Regex) -> Vec<(Range<usize>, Range<usize>)> {
    let mut ranges = Vec::new();
    let mut offset = 0usize;
    while let Some(capture) = regex.captures_at(source, offset) {
        let full = capture.get(0).expect("typography regex has a full match");
        let content = capture
            .get(1)
            .expect("surround typography regex has a content group");
        ranges.push((full.start()..content.start(), content.end()..full.end()));
        offset = full.end();
    }
    ranges
}

fn project_replacement(source: &mut [u8], regex: &Regex) -> bool {
    let ranges = replacement_matches(projected_str(source), regex);
    for target in ranges.iter().cloned() {
        source[target].fill(INERT);
    }
    !ranges.is_empty()
}

fn replacement_matches(source: &str, regex: &Regex) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut offset = 0usize;
    while let Some(capture) = regex.captures_at(source, offset) {
        let target = capture
            .name("repl")
            .expect("replacement typography regex has a repl group");
        ranges.push(target.start()..target.end());
        offset = target.end();
    }
    ranges
}

fn projected_str(source: &[u8]) -> &str {
    #[cfg(test)]
    UTF8_BYTES_VALIDATED.with(|total| total.set(total.get() + source.len()));
    std::str::from_utf8(source).expect("ListPages source projection preserves UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::render::list_pages::scanner::find_list_pages_module_matches;
    use crate::services::render::literal_regions::token_boundaries::TextTokenCursor;

    fn inert_projection(source: &str) -> String {
        let mut projected = source.as_bytes().to_vec();
        assert!(project_typography_in_place(&mut projected));
        String::from_utf8(projected).unwrap()
    }

    fn ftml_typography(source: &str) -> String {
        let mut projected = source.to_owned();
        ftml::preproc::typography::substitute(&mut projected);
        projected
    }

    fn token_owns(source: &str, delimiter: &str) -> bool {
        let offset = source.find(delimiter).unwrap();
        TextTokenCursor::new(source).contains(offset)
    }

    #[test]
    fn projection_uses_the_pinned_typography_order_and_overlap_offsets() {
        let source = "``double'' ,,low'' `single' ... . . . ...";
        assert_eq!(
            inert_projection(source),
            "??double?? ??low?? ?single? ??? ????? ???",
        );
        assert_eq!(ftml_typography(source), "“double” „low” ‘single’ … … …",);

        for (source, expected) in [
            (". . .", "?????"),
            (". . . ", "????? "),
            ("... ", "??? "),
            ("... . . . ...", "??? ????? ???"),
            ("x... ...y. . . z", "x??? ???y????? z"),
        ] {
            assert_eq!(inert_projection(source), expected, "{source:?}");
        }
    }

    #[test]
    fn projection_does_not_synthesize_wikidot_delimiters() {
        const DELIMITERS: &[&str] = &[
            "@@", "@<", ">@", "[!--", "--]", "[[[[", "]]]]", "[[[*", "[[[", "[[$", "[[#",
            "[[*", "[[/", "[[", "[#", "[*", "((", "]]]", "$]]", "]]", "))", "**", "//",
            "__", "^^", ",,", "##", "{{", "}}", "||~", "||>", "||=", "||", "<<", "\\\"",
            "\\\\", "~~~", "---", "++",
        ];
        let comment_repro = r#"[`--x' [[module ListPages name="live"]]X[[/module]] --]"#;
        let projected = inert_projection(comment_repro);

        assert_eq!(
            projected,
            r#"[?--x? [[module ListPages name="live"]]X[[/module]] --]"#,
        );
        assert_eq!(
            ftml_typography(comment_repro),
            r#"[‘--x’ [[module ListPages name="live"]]X[[/module]] --]"#,
        );
        let modules = find_list_pages_module_matches(comment_repro);
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].head, r#"name="live""#);

        for source in [
            "[`--x'", "[...--", "@...@", "@. . .<", ">. . .@", "[...[", "]...]", "[...$",
            "$...]",
        ] {
            let projected = inert_projection(source);
            for &delimiter in DELIMITERS {
                assert!(
                    projected.matches(delimiter).count()
                        <= source.matches(delimiter).count(),
                    "{source:?} projected to {projected:?} and synthesized {delimiter:?}",
                );
            }
        }
    }

    #[test]
    fn dense_typography_projection_validates_each_pass_once() {
        const SEGMENTS: usize = 20_000;
        let source = "`x'... ".repeat(SEGMENTS);
        UTF8_BYTES_VALIDATED.with(|total| total.set(0));

        let projected = inert_projection(&source);
        let validated = UTF8_BYTES_VALIDATED.with(Cell::get);

        assert_eq!(projected, "?x???? ".repeat(SEGMENTS));
        assert_eq!(validated, source.len() * 4);
    }

    #[test]
    fn projected_email_ownership_matches_pinned_ellipsis_output() {
        for (source, delimiter, raw_owned, projected_owned) in [
            ("a@b...@@ tail", "@@", true, false),
            ("a@b...@< tail", "@<", true, false),
            ("a@b.c. . .@@ tail", "@@", false, true),
            ("a@b.c. . .@< tail", "@<", false, true),
            ("a@b.c. . . @@ tail", "@@", false, false),
            ("a@b... @@ tail", "@@", false, false),
            ("a@b... . . . ...@@ tail", "@@", false, false),
        ] {
            let inert = inert_projection(source);
            let actual = ftml_typography(source);
            assert_eq!(token_owns(source, delimiter), raw_owned, "raw {source:?}");
            assert_eq!(
                token_owns(&inert, delimiter),
                projected_owned,
                "inert {source:?}",
            );
            assert_eq!(
                token_owns(&actual, delimiter),
                projected_owned,
                "FTML {source:?} became {actual:?}",
            );
        }
    }

    #[test]
    fn projected_url_ownership_matches_pinned_typography_output() {
        for (source, delimiter, raw_owned, projected_owned) in [
            ("https://e.test/a...@@ tail", "@@", true, true),
            ("https://e.test/a...@< tail", "@<", true, true),
            ("https://e.test/a. . .@@ tail", "@@", false, true),
            ("https://e.test/a. . .@< tail", "@<", false, true),
        ] {
            let inert = inert_projection(source);
            let actual = ftml_typography(source);
            assert_eq!(token_owns(source, delimiter), raw_owned, "raw {source:?}");
            assert_eq!(
                token_owns(&inert, delimiter),
                projected_owned,
                "inert {source:?}",
            );
            assert_eq!(
                token_owns(&actual, delimiter),
                projected_owned,
                "FTML {source:?} became {actual:?}",
            );
        }
    }

    #[test]
    fn projected_ellipsis_ownership_controls_raw_literal_visibility() {
        for source in [
            r#"a@b...@@ [[module ListPages name="hidden"]]x[[/module]] @@"#,
            r#"a@b...@< [[module ListPages name="hidden"]]x[[/module]] >@"#,
        ] {
            assert!(
                find_list_pages_module_matches(source).is_empty(),
                "{source:?}"
            );
        }

        for source in [
            r#"a@b.c. . .@@ [[module ListPages name="visible"]]x[[/module]] @@"#,
            r#"a@b.c. . .@< [[module ListPages name="visible"]]x[[/module]] >@"#,
        ] {
            let modules = find_list_pages_module_matches(source);
            assert_eq!(modules.len(), 1, "{source:?}");
            assert_eq!(modules[0].head, r#"name="visible""#, "{source:?}");
        }
    }

    #[test]
    fn projected_quote_replacements_hide_only_directly_surviving_candidates() {
        for source in [
            r#"[[span title='before `x' [[module ListPages name="hidden"]]x[[/module]] after']]"#,
            r#"[[span title='before ``x\'' [[module ListPages name="hidden"]]x[[/module]] after']]"#,
            r#"[[span title='before ,,x\'' [[module ListPages name="hidden"]]x[[/module]] after']]"#,
            r#"[[span `x' [[module ListPages name="visible"]]x[[/module]] after']]"#,
            r#"[[span ``x\'' [[module ListPages name="visible"]]x[[/module]] after']]"#,
            r#"[[span ,,x\'' [[module ListPages name="visible"]]x[[/module]] after']]"#,
        ] {
            assert!(
                find_list_pages_module_matches(source).is_empty(),
                "{source:?}"
            );
        }
    }
}
