/*
 * tree/date.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2023 Wikijump Team
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

use std::io;
use time::format_description::well_known::Rfc2822;
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum DateItem {
    Date(Date),
    DateTime(PrimitiveDateTime),
    DateTimeTz(OffsetDateTime),
}

impl DateItem {
    pub fn add_timezone(self, offset: UtcOffset) -> Option<Self> {
        let datetime_tz = match self {
            DateItem::Date(date) => date.midnight().assume_offset(offset),
            DateItem::DateTime(datetime) => datetime.assume_offset(offset),
            DateItem::DateTimeTz(_) => return None,
        };

        Some(DateItem::DateTimeTz(datetime_tz))
    }

    pub fn timestamp(self) -> i64 {
        match self {
            DateItem::Date(date) => date.midnight().assume_utc().unix_timestamp(),
            DateItem::DateTime(datetime) => datetime.assume_utc().unix_timestamp(),
            DateItem::DateTimeTz(datetime_tz) => datetime_tz.unix_timestamp(),
        }
    }

    pub fn time_since(self) -> i64 {
        self.timestamp() - now().timestamp()
    }

    pub fn to_datetime_tz(self) -> OffsetDateTime {
        match self {
            DateItem::Date(date) => date.midnight().assume_utc(),
            DateItem::DateTime(datetime) => datetime.assume_utc(),
            DateItem::DateTimeTz(datetime_tz) => datetime_tz,
        }
    }

    pub fn format(self) -> io::Result<String> {
        use time::error::Format;

        let result = match self {
            DateItem::Date(date) => date.format(&Rfc2822),
            DateItem::DateTime(datetime) => datetime.format(&Rfc2822),
            DateItem::DateTimeTz(datetime_tz) => datetime_tz.format(&Rfc2822),
        };

        result.map_err(|error| match error {
            Format::StdIo(io_error) => io_error,
            _ => io::Error::new(io::ErrorKind::Other, error),
        })
    }
}

impl From<Date> for DateItem {
    #[inline]
    fn from(date: Date) -> Self {
        DateItem::Date(date)
    }
}

impl From<PrimitiveDateTime> for DateItem {
    #[inline]
    fn from(datetime: PrimitiveDateTime) -> Self {
        DateItem::DateTime(datetime)
    }
}

impl From<OffsetDateTime> for DateItem {
    #[inline]
    fn from(datetime_tz: OffsetDateTime) -> Self {
        DateItem::DateTimeTz(datetime_tz)
    }
}

cfg_if! {
    if #[cfg(test)] {
        /// Produces a fixed constant value as "now".
        ///
        /// We need a consistent date for render tests to not constantly expire.
        #[inline]
        fn now() -> DateItem {
            time::macros::datetime!(2010-01-01 08:10:00).into()
        }
    } else {
        /// Helper function to get the current date and time, UTC.
        #[inline]
        fn now() -> DateItem {
            OffsetDateTime::now_utc().into()
        }
    }
}
