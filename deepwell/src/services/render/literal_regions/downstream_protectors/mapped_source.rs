/*
 * services/render/literal_regions/downstream_protectors/mapped_source.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::{AcceptedProtectorRange, record_collector_work};
use std::ops::Range;

pub(super) struct MappedSource {
    source: String,
    origin_pieces: Vec<OriginPiece>,
}

#[derive(Clone, Debug)]
struct OriginPiece {
    virtual_range: Range<usize>,
    original_range: Range<usize>,
    linear: bool,
}

pub(super) struct OriginRangeCursor<'a> {
    pieces: &'a [OriginPiece],
    index: usize,
}

struct VirtualRangeCursor<'a> {
    pieces: &'a [OriginPiece],
    index: usize,
}

impl MappedSource {
    pub(super) fn new(source: &str) -> Self {
        record_collector_work(source.len());
        Self {
            source: source.to_owned(),
            origin_pieces: (!source.is_empty())
                .then_some(OriginPiece {
                    virtual_range: 0..source.len(),
                    original_range: 0..source.len(),
                    linear: true,
                })
                .into_iter()
                .collect(),
        }
    }

    pub(super) fn source(&self) -> &str {
        &self.source
    }

    pub(super) fn origin_cursor(&self) -> OriginRangeCursor<'_> {
        OriginRangeCursor {
            pieces: &self.origin_pieces,
            index: 0,
        }
    }

    fn virtual_cursor(&self) -> VirtualRangeCursor<'_> {
        VirtualRangeCursor {
            pieces: &self.origin_pieces,
            index: 0,
        }
    }

    pub(super) fn remove_original_ranges(&mut self, ranges: &[Range<usize>]) {
        let mut virtual_ranges = self.virtual_cursor();
        let replacements = ranges
            .iter()
            .map(|range| (virtual_ranges.map_range(range.clone()), None))
            .collect::<Vec<_>>();
        self.rewrite(&replacements, None);
    }

    pub(super) fn replace_original_ranges_with_text(
        &mut self,
        ranges: &[Range<usize>],
        replacement: &str,
    ) {
        assert!(
            !replacement.is_empty(),
            "opaque replacements must be nonempty"
        );
        let mut virtual_ranges = self.virtual_cursor();
        let replacements = ranges
            .iter()
            .map(|range| (virtual_ranges.map_range(range.clone()), Some(range.clone())))
            .collect::<Vec<_>>();
        self.rewrite(&replacements, Some(replacement));
    }

    pub(super) fn replace_original_ranges_with_inert_markers(
        &mut self,
        ranges: &[Range<usize>],
    ) {
        self.replace_original_ranges_with_text(ranges, "x");
    }

    pub(super) fn replace_with_inert_markers(
        &mut self,
        ranges: &[AcceptedProtectorRange],
    ) {
        let replacements = ranges
            .iter()
            .map(|accepted| {
                (
                    accepted.virtual_range.clone(),
                    Some(accepted.protected.range.clone()),
                )
            })
            .collect::<Vec<_>>();
        self.rewrite(&replacements, Some("x"));
    }

    fn rewrite(
        &mut self,
        replacements: &[(Range<usize>, Option<Range<usize>>)],
        replacement: Option<&str>,
    ) {
        if replacements.is_empty() {
            return;
        }
        let mut rewritten = String::with_capacity(self.source.len());
        let mut rewritten_pieces = Vec::with_capacity(self.origin_pieces.len());
        let mut piece_cursor = 0usize;
        let mut cursor = 0usize;
        for (range, marker_origin) in replacements {
            debug_assert!(cursor <= range.start);
            rewritten.push_str(&self.source[cursor..range.start]);
            append_origin_slice(
                &self.origin_pieces,
                cursor..range.start,
                rewritten.len() - (range.start - cursor),
                &mut piece_cursor,
                &mut rewritten_pieces,
            );
            record_collector_work(range.start - cursor);
            if let Some(marker_origin) = marker_origin {
                // Issued registry markers contain no downstream syntax delimiter. Their length and nonce do not affect these regexes or guards, so one inert byte models them; non-registry replacements pass their exact text.
                let replacement =
                    replacement.expect("replacement text accompanies an origin");
                let marker_start = rewritten.len();
                rewritten.push_str(replacement);
                push_origin_piece(
                    &mut rewritten_pieces,
                    OriginPiece {
                        virtual_range: marker_start..marker_start + replacement.len(),
                        original_range: marker_origin.clone(),
                        linear: false,
                    },
                );
                record_collector_work(replacement.len());
            }
            cursor = range.end;
        }
        rewritten.push_str(&self.source[cursor..]);
        append_origin_slice(
            &self.origin_pieces,
            cursor..self.source.len(),
            rewritten.len() - (self.source.len() - cursor),
            &mut piece_cursor,
            &mut rewritten_pieces,
        );
        record_collector_work(self.source.len() - cursor);
        self.source = rewritten;
        self.origin_pieces = rewritten_pieces;
    }
}

impl OriginRangeCursor<'_> {
    pub(super) fn map_range(&mut self, range: Range<usize>) -> Range<usize> {
        debug_assert!(range.start < range.end);
        while self
            .pieces
            .get(self.index)
            .is_some_and(|piece| piece.virtual_range.end <= range.start)
        {
            self.index += 1;
            record_collector_work(1);
        }
        let first = self
            .pieces
            .get(self.index)
            .expect("mapped range start belongs to an origin piece");
        debug_assert!(first.virtual_range.start <= range.start);
        let original_start = map_piece_start(first, range.start);

        let mut last_index = self.index;
        while self.pieces[last_index].virtual_range.end < range.end {
            last_index += 1;
            record_collector_work(1);
        }
        let last = &self.pieces[last_index];
        debug_assert!(range.end <= last.virtual_range.end);
        self.index = last_index;
        original_start..map_piece_end(last, range.end)
    }
}

impl VirtualRangeCursor<'_> {
    fn map_range(&mut self, range: Range<usize>) -> Range<usize> {
        debug_assert!(range.start < range.end);
        while self
            .pieces
            .get(self.index)
            .is_some_and(|piece| piece.original_range.end <= range.start)
        {
            self.index += 1;
            record_collector_work(1);
        }
        let first = self
            .pieces
            .get(self.index)
            .expect("original range start belongs to a mapped piece");
        debug_assert!(first.original_range.start <= range.start);
        let virtual_start = map_original_piece_start(first, range.start);

        let mut last_index = self.index;
        while self.pieces[last_index].original_range.end < range.end {
            last_index += 1;
            record_collector_work(1);
        }
        let last = &self.pieces[last_index];
        debug_assert!(range.end <= last.original_range.end);
        self.index = last_index;
        virtual_start..map_original_piece_end(last, range.end)
    }
}

fn map_piece_start(piece: &OriginPiece, offset: usize) -> usize {
    if piece.linear {
        piece.original_range.start + offset - piece.virtual_range.start
    } else {
        debug_assert_eq!(offset, piece.virtual_range.start);
        piece.original_range.start
    }
}

fn map_piece_end(piece: &OriginPiece, offset: usize) -> usize {
    if piece.linear {
        piece.original_range.start + offset - piece.virtual_range.start
    } else {
        debug_assert_eq!(offset, piece.virtual_range.end);
        piece.original_range.end
    }
}

fn map_original_piece_start(piece: &OriginPiece, offset: usize) -> usize {
    if piece.linear {
        piece.virtual_range.start + offset - piece.original_range.start
    } else {
        debug_assert_eq!(offset, piece.original_range.start);
        piece.virtual_range.start
    }
}

fn map_original_piece_end(piece: &OriginPiece, offset: usize) -> usize {
    if piece.linear {
        piece.virtual_range.start + offset - piece.original_range.start
    } else {
        debug_assert_eq!(offset, piece.original_range.end);
        piece.virtual_range.end
    }
}

fn append_origin_slice(
    pieces: &[OriginPiece],
    slice: Range<usize>,
    output_start: usize,
    piece_cursor: &mut usize,
    output: &mut Vec<OriginPiece>,
) {
    if slice.is_empty() {
        return;
    }
    while pieces
        .get(*piece_cursor)
        .is_some_and(|piece| piece.virtual_range.end <= slice.start)
    {
        *piece_cursor += 1;
        record_collector_work(1);
    }

    let mut cursor = slice.start;
    while cursor < slice.end {
        let piece = &pieces[*piece_cursor];
        let end = piece.virtual_range.end.min(slice.end);
        let length = end - cursor;
        let original_range = map_piece_start(piece, cursor)..map_piece_end(piece, end);
        push_origin_piece(
            output,
            OriginPiece {
                virtual_range: output_start + cursor - slice.start
                    ..output_start + cursor - slice.start + length,
                original_range,
                linear: piece.linear,
            },
        );
        cursor = end;
        if cursor == piece.virtual_range.end {
            *piece_cursor += 1;
            record_collector_work(1);
        }
    }
}

fn push_origin_piece(output: &mut Vec<OriginPiece>, piece: OriginPiece) {
    if let Some(previous) = output.last_mut()
        && previous.linear
        && piece.linear
        && previous.virtual_range.end == piece.virtual_range.start
        && previous.original_range.end == piece.original_range.start
    {
        previous.virtual_range.end = piece.virtual_range.end;
        previous.original_range.end = piece.original_range.end;
        return;
    }
    output.push(piece);
}
