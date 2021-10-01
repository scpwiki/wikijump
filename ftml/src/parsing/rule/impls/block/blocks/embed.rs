/*
 * parsing/rule/impls/block/blocks/embed.rs
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

use super::prelude::*;
use crate::tree::Embed;
use std::borrow::Cow;

type EmbedBuilderFn = for<'p, 't> fn(
    &'p Parser<'_, 't>,
    &'p mut Arguments<'t>,
) -> Result<Embed<'t>, ParseWarning>;

pub const BLOCK_EMBED: BlockRule = BlockRule {
    name: "block-embed",
    accepts_names: &["embed"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(
        log,
        "Parsing embed block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Embed doesn't allow star flag");
    assert!(!flag_score, "Embed doesn't allow star flag");
    assert_block_name(&BLOCK_EMBED, name);

    let (name, mut arguments) = parser.get_head_name_map(&BLOCK_EMBED, in_head)?;
    let embed = build_embed(parser, name, &mut arguments)?;

    todo!()
}

fn build_embed<'r, 't>(
    parser: &Parser<'r, 't>,
    name: &str,
    arguments: &mut Arguments<'t>,
) -> Result<Embed<'t>, ParseWarning>
where
    'r: 't,
{
    const EMBED_BUILDERS: &[(&str, EmbedBuilderFn)] = &[
        ("youtube", build_youtube),
        ("vimeo", build_vimeo),
        ("github-gist", build_github_gist),
        ("gitlab-snippet", build_gitlab_snippet),
    ];

    for &(embed_name, builder) in EMBED_BUILDERS {
        if embed_name.eq_ignore_ascii_case(name) {
            return builder(parser, arguments);
        }
    }

    Err(parser.make_warn(ParseWarningKind::NoSuchEmbed))
}

// Different embed builders

fn build_youtube<'p, 't>(
    parser: &'p Parser<'_, 't>,
    arguments: &'p mut Arguments<'t>,
) -> Result<Embed<'t>, ParseWarning> {
    let video_id = arguments
        .get("video")
        .ok_or_else(|| parser.make_warn(ParseWarningKind::BlockMissingArguments))?;

    let width = parse_num(parser, arguments.get("width"))?;
    let height = parse_num(parser, arguments.get("height"))?;

    Ok(Embed::YouTube {
        video_id,
        width,
        height,
    })
}

fn build_vimeo<'p, 't>(
    parser: &'p Parser<'_, 't>,
    arguments: &'p mut Arguments<'t>,
) -> Result<Embed<'t>, ParseWarning> {
    let video_id = arguments
        .get("video")
        .ok_or_else(|| parser.make_warn(ParseWarningKind::BlockMissingArguments))?;

    let width = parse_num(parser, arguments.get("width"))?;
    let height = parse_num(parser, arguments.get("height"))?;

    Ok(Embed::Vimeo {
        video_id,
        width,
        height,
    })
}

fn build_github_gist<'p, 't>(
    parser: &'p Parser<'_, 't>,
    arguments: &'p mut Arguments<'t>,
) -> Result<Embed<'t>, ParseWarning> {
    let username = arguments
        .get("username")
        .ok_or_else(|| parser.make_warn(ParseWarningKind::BlockMissingArguments))?;

    let hash = arguments
        .get("hash")
        .ok_or_else(|| parser.make_warn(ParseWarningKind::BlockMissingArguments))?;

    Ok(Embed::GithubGist { username, hash })
}

fn build_gitlab_snippet<'p, 't>(
    parser: &'p Parser<'_, 't>,
    arguments: &'p mut Arguments<'t>,
) -> Result<Embed<'t>, ParseWarning> {
    let snippet_id = arguments
        .get("id")
        .ok_or_else(|| parser.make_warn(ParseWarningKind::BlockMissingArguments))?;

    Ok(Embed::GitlabSnippet { snippet_id })
}

// Utilities

fn parse_num(
    parser: &Parser,
    value: Option<Cow<str>>,
) -> Result<Option<u32>, ParseWarning> {
    match value {
        None => Ok(None),
        Some(value) => match value.parse() {
            Ok(num) => Ok(Some(num)),
            Err(_) => Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments)),
        },
    }
}

#[test]
fn embed_builder_types() {
    let _: EmbedBuilderFn = build_youtube;
    let _: EmbedBuilderFn = build_vimeo;
    let _: EmbedBuilderFn = build_github_gist;
    let _: EmbedBuilderFn = build_gitlab_snippet;
}
