/*
 * parsing/rule/impls/block/blocks/iframe.rs
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

use super::prelude::*;
use crate::url::is_url;

pub const BLOCK_IFRAME: BlockRule = BlockRule {
    name: "block-iframe",
    accepts_names: &["iframe"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing iframe block (in-head {in_head})");
    assert!(!flag_star, "iframe doesn't allow star flag");
    assert!(!flag_score, "iframe doesn't allow score flag");
    assert_block_name(&BLOCK_IFRAME, name);

    let (url, arguments) = parser.get_head_name_map(&BLOCK_IFRAME, in_head)?;
    if !is_url(url) {
        warn!("Iframe block references non-URL: {url}");
        return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments));
    }

    let element = Element::Iframe {
        url: cow!(url),
        attributes: arguments.to_attribute_map(parser.settings()),
    };

    ok!(element)
}
