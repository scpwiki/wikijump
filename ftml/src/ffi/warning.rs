/*
 * ffi/warning.rs
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
use crate::parsing::ParseWarning;
use std::borrow::Cow;

/// Representation of an ftml parsing warning in C.
///
/// All of the strings here are statically allocated.
/// They are reused and must not be modified or freed.
#[repr(C)]
#[derive(Debug)]
pub struct ftml_warning {
    pub token: *const c_char,
    pub rule: *const c_char,
    pub span_start: usize,
    pub span_end: usize,
    pub kind: *const c_char,
}

impl From<&'_ ParseWarning> for ftml_warning {
    fn from(warning: &ParseWarning) -> ftml_warning {
        let rule_cstr = match warning.rule_raw() {
            Cow::Borrowed(s) => get_static_cstr(s),
            Cow::Owned(_) => panic!("Rule string not static, was serialized"),
        };

        ftml_warning {
            token: get_static_cstr(warning.token().name()),
            rule: rule_cstr,
            span_start: warning.span().start,
            span_end: warning.span().end,
            kind: get_static_cstr(warning.kind().name()),
        }
    }
}
