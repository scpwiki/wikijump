/*
 * services/render/literal_regions/common.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::ops::Range;

pub(super) fn collect_wikidot_tag_ranges(source: &str, ranges: &mut Vec<Range<usize>>) {
    let mut line_start = 0usize;
    for line in source.split_inclusive('\n') {
        let bytes = line.as_bytes();
        let mut cursor = 0usize;
        while let Some(offset) = line[cursor..].find("[[") {
            let relative_start = cursor + offset;
            let start = line_start + relative_start;
            let mut relative_end = relative_start + 2;
            let mut quote = None;
            while relative_end + 1 < bytes.len() {
                match (quote, bytes[relative_end]) {
                    (Some(expected), actual) if expected == actual => {
                        quote = None;
                        relative_end += 1;
                    }
                    (None, b'\'' | b'"') => {
                        quote = Some(bytes[relative_end]);
                        relative_end += 1;
                    }
                    (None, b']') if bytes[relative_end + 1] == b']' => {
                        relative_end += 2;
                        break;
                    }
                    _ => relative_end += 1,
                }
            }
            if relative_end + 1 >= bytes.len() {
                relative_end = line.len();
            }
            ranges.push(start..line_start + relative_end);
            cursor = relative_end;
        }
        line_start += line.len();
    }
}

#[derive(Clone, Copy)]
struct WikidotLiteralBlock {
    close: &'static str,
    quote_depth: usize,
    start: usize,
}

pub(super) fn collect_wikidot_block_ranges(source: &str, ranges: &mut Vec<Range<usize>>) {
    let mut offset = 0usize;
    let mut active: Option<WikidotLiteralBlock> = None;

    for line in source.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let (quote_depth, logical) = quote_depth_and_body(body);
        let logical_start = offset + body.len() - logical.len();
        let lower = logical.to_ascii_lowercase();

        if let Some(block) = active {
            if block.quote_depth > 0 && quote_depth < block.quote_depth {
                ranges.push(block.start..offset);
                active = None;
            } else {
                let close_depth_matches =
                    block.quote_depth == 0 || quote_depth == block.quote_depth;
                if close_depth_matches && let Some(close_start) = lower.find(block.close)
                {
                    ranges.push(
                        block.start..logical_start + close_start + block.close.len(),
                    );
                    active = None;
                }
                offset += line.len();
                continue;
            }
        }

        if let Some((close, opener_end)) = wikidot_literal_block(&lower) {
            let block = WikidotLiteralBlock {
                close,
                quote_depth,
                start: logical_start,
            };
            if let Some(relative_close) = lower[opener_end..].find(close) {
                ranges.push(
                    logical_start
                        ..logical_start + opener_end + relative_close + close.len(),
                );
            } else {
                active = Some(block);
            }
        }
        offset += line.len();
    }

    if let Some(block) = active {
        ranges.push(block.start..source.len());
    }
}

fn quote_depth_and_body(mut body: &str) -> (usize, &str) {
    let mut quote_depth = 0;
    body = body.trim_start_matches([' ', '\t']);
    while let Some(rest) = body.strip_prefix('>') {
        quote_depth += 1;
        body = rest.trim_start_matches([' ', '\t']);
    }
    (quote_depth, body)
}

fn wikidot_literal_block(lower: &str) -> Option<(&'static str, usize)> {
    let marker = lower.strip_prefix("[[")?.trim_start();
    let (head, _) = marker.split_once("]]")?;
    let opener_end = lower.find("]]")? + 2;
    let close = match head.trim_end().split_ascii_whitespace().next()? {
        "code" => "[[/code]]",
        "html" => "[[/html]]",
        "raw" => "[[/raw]]",
        _ => return None,
    };
    Some((close, opener_end))
}
