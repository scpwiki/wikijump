/*
 * tree/date.rs
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

use chrono::prelude::*;

/// The default format string for `[[date]]`, if none is specified.
pub const DEFAULT_DATETIME_FORMAT: &str = "%B %d, %Y %I:%M:%S %p %Z";

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Date {
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    DateTimeTz(DateTime<FixedOffset>),
}

impl Date {
    pub fn timestamp(self) -> i64 {
        match self {
            Date::Date(date) => date.and_hms(0, 0, 0).timestamp(),
            Date::DateTime(datetime) => datetime.timestamp(),
            Date::DateTimeTz(datetime_tz) => datetime_tz.timestamp(),
        }
    }

    pub fn time_since(self) -> i64 {
        self.timestamp() - Utc::now().timestamp()
    }

    pub fn to_datetime_tz(self) -> DateTime<FixedOffset> {
        match self {
            Date::Date(date) => to_datetime_tz(to_datetime(date)),
            Date::DateTime(datetime) => to_datetime_tz(datetime),
            Date::DateTimeTz(datetime_tz) => datetime_tz,
        }
    }

    pub fn to_rfc3339(self) -> String {
        self.to_datetime_tz().to_rfc3339()
    }

    pub fn format<S: AsRef<str>>(self, format_string: Option<S>) -> String {
        let format_string = match format_string {
            Some(s) => s.as_ref(),
            None => DEFAULT_DATETIME_FORMAT,
        };

        let result = match self {
            Date::Date(date) => date.format(format_string),
            Date::DateTime(datetime) => datetime.format(format_string),
            Date::DateTimeTz(datetime_tz) => datetime_tz.format(format_string),
        };

        str!(result)
    }
}

impl From<NaiveDate> for Date {
    #[inline]
    fn from(date: NaiveDate) -> Self {
        Date::Date(date)
    }
}

impl From<NaiveDateTime> for Date {
    #[inline]
    fn from(datetime: NaiveDateTime) -> Self {
        Date::DateTime(datetime)
    }
}

impl From<DateTime<FixedOffset>> for Date {
    #[inline]
    fn from(datetime_tz: DateTime<FixedOffset>) -> Self {
        Date::DateTimeTz(datetime_tz)
    }
}

#[inline]
fn to_datetime(date: NaiveDate) -> NaiveDateTime {
    date.and_hms(0, 0, 0)
}

#[inline]
fn to_datetime_tz(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
    DateTime::from_utc(datetime, FixedOffset::east(0))
}
