/*
 * span_wrap.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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

//! Wrapper struct to allow formatting of `Range

use std::ops::Range;

#[derive(Debug, Clone)]
pub struct SpanWrap<'a> {
    inner: &'a Range<usize>,
}

impl<'a> slog::Value for SpanWrap<'a> {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, "[")?;
        serializer.emit_usize(key, self.inner.start)?;
        serializer.emit_str(key, ", ")?;
        serializer.emit_usize(key, self.inner.end)?;
        serializer.emit_str(key, "]")?;
        Ok(())
    }
}
