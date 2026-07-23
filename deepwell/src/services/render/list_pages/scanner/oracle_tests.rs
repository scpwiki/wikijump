/*
 * services/render/list_pages_scanner/oracle_tests.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use super::super::super::literal_regions::{
    ListPagesSourceProjection, TextTokenCursor, left_block_start_in_run,
    wikidot_right_bracket_token, wikidot_trimmed_name,
};
use ftml::parsing::Token;
use std::ops::Range;

const MAX_ORACLE_SOURCE_BYTES: usize = 256;

fn pinned_tokens(source: &str) -> Vec<(Token, Range<usize>)> {
    assert!(source.len() <= MAX_ORACLE_SOURCE_BYTES);
    let tokenization = ftml::tokenize(source);
    let tokens = tokenization
        .tokens()
        .iter()
        .map(|extracted| (extracted.token, extracted.span.clone()))
        .collect();
    drop(tokenization);
    tokens
}

fn token_covering(
    tokens: &[(Token, Range<usize>)],
    offset: usize,
) -> Option<(Token, &Range<usize>)> {
    tokens
        .iter()
        .find(|(_, span)| span.start <= offset && offset < span.end)
        .map(|(token, span)| (*token, span))
}

fn is_left_block_family(token: Token) -> bool {
    matches!(
        token,
        Token::LeftBlock
            | Token::LeftBlockEnd
            | Token::LeftBlockAnchor
            | Token::LeftBlockStar
            | Token::LeftMath
    )
}

#[test]
fn right_bracket_scanner_matches_pinned_tokens_for_short_runs_and_marker_ownership() {
    let mut sources = (1..=12).map(|run| "]".repeat(run)).collect::<Vec<_>>();
    sources.extend(
        [
            "$]",
            "$]]",
            "$]]]",
            "$]]]]",
            "--]",
            "--]]",
            "--]]]",
            "--]]]]",
            "---]",
            "---]]",
            "----]",
            "----]]",
            "https://e.test/a$]]",
            "https://e.test/a--]]",
            "foo@bar.example$]]",
            "foo@bar.example--]]",
        ]
        .into_iter()
        .map(str::to_owned),
    );

    for source in sources {
        let tokens = pinned_tokens(&source);
        let bytes = source.as_bytes();
        let mut text_tokens = TextTokenCursor::new(&source);
        let mut cursor = 0;
        while let Some(relative) = source[cursor..].find(']') {
            let start = cursor + relative;
            let (actual_is_right_block, actual_len) =
                wikidot_right_bracket_token(bytes, start, bytes.len(), &mut text_tokens);
            let (oracle_token, oracle_span) =
                token_covering(&tokens, start).expect("every bracket belongs to a token");
            assert!(
                oracle_span.start == start
                    || matches!(oracle_token, Token::RightMath | Token::RightComment),
                "the monotone caller must not call inside a bracket token: source={source:?}, start={start}, oracle={oracle_token:?} {oracle_span:?}",
            );
            let oracle_is_right_block =
                oracle_token == Token::RightBlock && oracle_span.start == start;
            let oracle_len = oracle_span.end - start;
            assert_eq!(
                (actual_is_right_block, actual_len),
                (oracle_is_right_block, oracle_len),
                "source={source:?}, start={start}, oracle={oracle_token:?} {oracle_span:?}",
            );
            cursor = start + actual_len;
        }
    }
}

#[test]
fn left_block_run_scanner_matches_pinned_block_family_starts() {
    for run in 2..=12 {
        for suffix in ["", "/", "*", "#", "$"] {
            let source = format!("{}{suffix}", "[".repeat(run));
            let tokens = pinned_tokens(&source);
            for candidate in 0..run - 1 {
                let (actual_start, actual_run_end) =
                    left_block_start_in_run(source.as_bytes(), candidate);
                let oracle_start = tokens
                    .iter()
                    .find(|(token, span)| {
                        is_left_block_family(*token)
                            && candidate <= span.start
                            && span.start < run
                    })
                    .map(|(_, span)| span.start);
                assert_eq!(
                    actual_run_end, run,
                    "source={source:?}, candidate={candidate}"
                );
                assert_eq!(
                    actual_start, oracle_start,
                    "source={source:?}, candidate={candidate}, tokens={tokens:?}",
                );
            }
        }
    }
}

#[test]
fn text_token_cursor_matches_pinned_url_and_email_spans() {
    let sources = [
        "foo@bar.example",
        "foo@bar.example=tail",
        "foo@bar.example\"tail",
        "foo@bar.example\\tail",
        "foo@bar.example$]]",
        "foo@bar.example--]]",
        "foo@bar.example\tend",
        "foo@bar.example end",
        "foo@bar.example\rend",
        "foo@bar.example\ntail",
        "foo@bar.example[tail",
        "foo@bar.example]tail",
        "foo@bar.example{tail",
        "foo@bar.example}tail",
        "foo@bar.example>@tail",
        "foo@bar.example@<tail",
        "foo@bar.example|tail",
        "foo@bar.example\u{00a0}tail",
        "foo@bar.example\0tail",
        "x foo@bar.example y baz@qux.example",
        "a%http://e.test/x foo@bar.example ftp://f.test/y",
        "http://e.test/x",
        "https://e.test/a",
        "ftp://e.test/a",
        "http://",
        "https://",
        "ftp://",
        "http://>@",
        "https://e.test/a=tail",
        "https://e.test/a'tail",
        "https://e.test/a\\tail",
        "https://e.test/a$]]",
        "https://e.test/a--]]",
        "https://e.test/a\tmore",
        "https://e.test/a more",
        "https://e.test/a\rmore",
        "https://e.test/a\nmore",
        "https://e.test/a\"more",
        "https://e.test/a|more",
        "https://e.test/a[more",
        "https://e.test/a]more",
        "https://e.test/a>@tail",
        "https://e.test/a\u{00a0}tail",
        "https://e.test/a\0tail",
        "HTTPS://e.test/a",
    ];

    for source in sources {
        let tokens = pinned_tokens(source);
        let mut cursor = TextTokenCursor::new(source);
        for offset in 0..source.len() {
            let oracle_contains = tokens.iter().any(|(token, span)| {
                matches!(token, Token::Url | Token::Email) && span.contains(&offset)
            });
            assert_eq!(
                cursor.contains(offset),
                oracle_contains,
                "source={source:?}, offset={offset}, tokens={tokens:?}",
            );
        }
    }
}

fn assert_single_content_token(source: &str, expected: Token) {
    let tokens = pinned_tokens(source)
        .into_iter()
        .filter(|(token, _)| !matches!(token, Token::InputStart | Token::InputEnd))
        .collect::<Vec<_>>();
    assert_eq!(
        tokens,
        vec![(expected, 0..source.len())],
        "source={source:?}"
    );
}

#[test]
fn pinned_whitespace_and_control_token_boundaries_are_explicit() {
    for source in [" ", "\t", " \t "] {
        assert_single_content_token(source, Token::Whitespace);
    }
    for source in ["\n", "\r", "\r\n"] {
        assert_single_content_token(source, Token::LineBreak);
    }
    for source in ["\n\n", "\r\r", "\r\n\r\n", "\r\n\r", "\r\n\n", "\n\r\n"] {
        assert_single_content_token(source, Token::ParagraphBreak);
    }
    for source in ["\u{00a0}", "\u{2007}", "\0", "\u{000b}", "\u{000c}"] {
        assert_single_content_token(source, Token::Other);
    }

    assert_eq!(
        pinned_tokens("\n \n")
            .into_iter()
            .filter(|(token, _)| !matches!(token, Token::InputStart | Token::InputEnd))
            .collect::<Vec<_>>(),
        vec![
            (Token::LineBreak, 0..1),
            (Token::Whitespace, 1..2),
            (Token::LineBreak, 2..3),
        ],
    );
}

fn pinned_whitespace_projection(source: &str) -> String {
    assert!(source.len() <= MAX_ORACLE_SOURCE_BYTES);
    let mut projected = source.to_owned();
    ftml::preproc::whitespace::substitute(&mut projected);
    projected
}

fn scanner_whitespace_projection(source: &str) -> String {
    assert!(source.len() <= MAX_ORACLE_SOURCE_BYTES);
    ListPagesSourceProjection::new(source).map_or_else(
        || source.to_owned(),
        |projection| projection.source().to_owned(),
    )
}

#[test]
fn list_pages_whitespace_projection_matches_pinned_ftml() {
    for source in [
        "alpha\r\nbeta\rgamma\n",
        "alpha\r\n \t\r\nbeta",
        "alpha\\\r\nbeta\\\rgamma\\\ndelta",
        " \t\r\n\u{2007}lead\r\ninside\u{2007}space\0\tend",
        "\u{00a0}lead\rnext\nmid\u{00a0}dle",
    ] {
        assert_eq!(
            scanner_whitespace_projection(source),
            pinned_whitespace_projection(source),
            "source={source:?}",
        );
    }
}

fn pinned_trimmed_name_matches(source: &str, expected: &[u8]) -> bool {
    let tokens = pinned_tokens(source);
    let raw_end = tokens
        .iter()
        .find(|(token, _)| {
            matches!(
                token,
                Token::LineBreak
                    | Token::ParagraphBreak
                    | Token::Whitespace
                    | Token::RightBlock
            )
        })
        .map_or(source.len(), |(_, span)| span.start);
    source[..raw_end]
        .trim()
        .as_bytes()
        .eq_ignore_ascii_case(expected)
}

#[test]
fn trimmed_name_separator_matches_pinned_token_collection_and_unicode_trim() {
    for source in [
        "user ",
        "\u{00a0}user ",
        "\u{2007}user ",
        "user\u{00a0} ",
        "user\u{2007} ",
        "us\u{00a0}er ",
        "us\u{2007}er ",
        "user\u{2007}\0 ",
        "\0user ",
    ] {
        let actual = wikidot_trimmed_name(source.as_bytes(), 0)
            .0
            .is_some_and(|name| name.eq_ignore_ascii_case(b"user"));
        assert_eq!(
            actual,
            pinned_trimmed_name_matches(source, b"user"),
            "source={source:?}",
        );
    }
}
