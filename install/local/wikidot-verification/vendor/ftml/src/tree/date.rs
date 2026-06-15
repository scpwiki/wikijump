/*
 * tree/date.rs
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

use icu_calendar::{Date as IcuDate, Gregorian, Iso};
use icu_datetime::DateTimeFormatter;
use icu_datetime::DateTimeFormatterPreferences;
use icu_datetime::fieldsets::enums::TimeFieldSet;
use icu_datetime::fieldsets::{E, M, T, YMD, YMDT};
use icu_datetime::input::{DateTime as IcuDateTime, Time as IcuTime};
use icu_datetime::options::{TimePrecision, YearStyle};
use icu_datetime::pattern::{
    DateTimePattern, DayPeriodNameLength, FixedCalendarDateTimeNames,
};
use icu_datetime::preferences::HourCycle;
use icu_decimal::input::Decimal;
use icu_experimental::relativetime::options::Numeric;
use icu_experimental::relativetime::{
    RelativeTimeFormatter, RelativeTimeFormatterOptions, RelativeTimeFormatterPreferences,
};
use icu_locale::Locale;
use std::fmt;
use std::io;
use std::sync::LazyLock;
use time::format_description::parse_strftime_borrowed;
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};
use writeable::TryWriteable;

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

    pub fn format(self, format: Option<&str>, language: &str) -> io::Result<String> {
        match format {
            Some(format) => self.format_strftime(format, language),
            None => self.format_default(language),
        }
    }

    pub fn format_or_default(self, format: Option<&str>, language: &str) -> String {
        match self.format(format, language) {
            Ok(datetime) => datetime,
            Err(first_error) => self.format(None, language).unwrap_or_else(|fallback_error| {
                error!(
                    "Error formatting date into string: initial error: {first_error}; fallback error: {fallback_error}"
                );
                str!("<ERROR>")
            }),
        }
    }

    fn format_strftime(self, format: &str, language: &str) -> io::Result<String> {
        let datetime = self.to_datetime_tz();
        let locale = locale_from_language(language);
        let mut rendered = String::new();
        let mut literal_start = 0;
        let mut chars = format.char_indices();

        while let Some((index, ch)) = chars.next() {
            if ch != '%' {
                continue;
            }

            rendered.push_str(&format[literal_start..index]);

            let Some((directive_index, mut directive)) = chars.next() else {
                return Err(invalid_strftime_error(
                    format,
                    "unexpected end of input after '%'",
                ));
            };

            let mut spec_end = directive_index + directive.len_utf8();
            if matches!(directive, '_' | '-' | '0') {
                let Some((component_index, component)) = chars.next() else {
                    return Err(invalid_strftime_error(
                        format,
                        "unexpected end of input after padding modifier",
                    ));
                };

                directive = component;
                spec_end = component_index + component.len_utf8();
            }

            let spec = &format[index..spec_end];
            render_directive(&mut rendered, &datetime, &locale, directive, spec)?;
            literal_start = spec_end;
        }

        rendered.push_str(&format[literal_start..]);

        Ok(rendered)
    }

    fn format_default(self, language: &str) -> io::Result<String> {
        let datetime = self.to_datetime_tz();
        let locale = locale_from_language(language);
        let mut rendered = String::new();

        append_localized_datetime_full_year(&mut rendered, &datetime, &locale)?;

        Ok(rendered)
    }
}

fn map_format_result(result: Result<String, time::error::Format>) -> io::Result<String> {
    use time::error::Format;

    result.map_err(|error| match error {
        Format::StdIo(io_error) => io_error,
        _ => io::Error::other(error),
    })
}

fn render_directive(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
    directive: char,
    spec: &str,
) -> io::Result<()> {
    match directive {
        '%' => rendered.push('%'),
        'a' => append_localized_weekday(rendered, datetime, locale, true)?,
        'A' => append_localized_weekday(rendered, datetime, locale, false)?,
        'b' => append_localized_month(rendered, datetime, locale, true)?,
        'B' => append_localized_month(rendered, datetime, locale, false)?,
        'c' => append_localized_datetime_full_year(rendered, datetime, locale)?,
        'O' => append_relative_time(rendered, *datetime, locale)?,
        'p' => append_localized_day_period(rendered, datetime, locale, true)?,
        'P' => append_localized_day_period(rendered, datetime, locale, false)?,
        'r' => append_localized_time_h12(rendered, datetime, locale)?,
        'X' => append_localized_time(rendered, datetime, locale)?,
        'x' => append_localized_date(rendered, datetime, locale)?,
        'Z' | 'z' => append_timezone_gmt(rendered, &datetime.offset()),
        _ => append_unlocalized_directive(rendered, datetime, spec)?,
    }

    Ok(())
}

/// %a/%A - localized weekday name.
/// EN: "Fri"/"Friday" | ES: "vie"/"viernes" | JA: "金"/"金曜日"
fn append_localized_weekday(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
    abbreviated: bool,
) -> io::Result<()> {
    let formatter = if abbreviated {
        DateTimeFormatter::try_new(locale.clone().into(), E::short())
    } else {
        DateTimeFormatter::try_new(locale.clone().into(), E::long())
    }
    .map_err(localization_error)?;
    let date = to_icu_date(*datetime)?;

    append_normalized_display(rendered, formatter.format(&date))
}

/// %b/%B - localized month name.
/// EN: "Dec"/"December" | ES: "dic"/"diciembre" | JA: "12月"/"12月"
fn append_localized_month(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
    abbreviated: bool,
) -> io::Result<()> {
    let formatter = if abbreviated {
        DateTimeFormatter::try_new(locale.clone().into(), M::medium())
    } else {
        DateTimeFormatter::try_new(locale.clone().into(), M::long())
    }
    .map_err(localization_error)?;
    let date = to_icu_date(*datetime)?;

    append_normalized_display(rendered, formatter.format(&date))
}

/// %x - localized short date.
/// EN: "12/25/09" | ES: "25/12/09" | JA: "2009/12/25"
fn append_localized_date(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
) -> io::Result<()> {
    let formatter = DateTimeFormatter::try_new(locale.clone().into(), YMD::short())
        .map_err(localization_error)?;
    let date = to_icu_date(*datetime)?;

    append_normalized_display(rendered, formatter.format(&date))
}

/// %c - localized date+time with full year.
/// EN: "12/25/2009, 8:18:05 AM" | ES: "25/12/2009, 08:18:05" | JA: "2009/12/25 8:18:05"
fn append_localized_datetime_full_year(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
) -> io::Result<()> {
    let formatter = DateTimeFormatter::try_new(
        locale.clone().into(),
        YMDT::short()
            .with_year_style(YearStyle::Full)
            .with_time_precision(TimePrecision::Second),
    )
    .map_err(localization_error)?;
    let datetime = to_icu_datetime(*datetime)?;

    append_normalized_display(rendered, formatter.format(&datetime))
}

/// %X - localized time (follows locale hour cycle).
/// EN: "8:18:05 AM" | ES: "08:18:05" | JA: "8:18:05"
fn append_localized_time(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
) -> io::Result<()> {
    let formatter = DateTimeFormatter::try_new(locale.clone().into(), T::medium())
        .map_err(localization_error)?;
    let time = to_icu_time(*datetime)?;

    append_normalized_display(rendered, formatter.format(&time))
}

/// %p/%P - localized day period via ICU pattern "a".
/// EN: "AM"/"am" | ES: "a. m."/"a. m." | JA: "午前"/"午前"
fn append_localized_day_period(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
    uppercase: bool,
) -> io::Result<()> {
    static DAY_PERIOD_PATTERN: LazyLock<DateTimePattern> =
        LazyLock::new(|| "a".parse().expect("valid day-period pattern"));

    let time = to_icu_time(*datetime)?;
    let mut names: FixedCalendarDateTimeNames<Gregorian, TimeFieldSet> =
        FixedCalendarDateTimeNames::try_new(locale.clone().into())
            .map_err(localization_error)?;
    names
        .include_day_period_names(DayPeriodNameLength::Abbreviated)
        .map_err(localization_error)?;

    let formatter = names
        .with_pattern_unchecked(&DAY_PERIOD_PATTERN)
        .format(&time);
    let raw = formatter
        .try_write_to_string()
        .map_err(|(error, _)| localization_error(error))?;

    if uppercase {
        append_normalized_display(rendered, raw.as_ref())
    } else {
        append_normalized_display(rendered, raw.as_ref().to_lowercase())
    }
}

/// %r - localized 12-hour time (forced H12 cycle).
/// EN: "8:18:05 AM" | ES: "8:18:05 a. m." | JA: "午前8:18:05"
fn append_localized_time_h12(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    locale: &Locale,
) -> io::Result<()> {
    let mut prefs: DateTimeFormatterPreferences = locale.clone().into();
    prefs.hour_cycle = Some(HourCycle::H12);

    let formatter =
        DateTimeFormatter::try_new(prefs, T::medium()).map_err(localization_error)?;
    let time = to_icu_time(*datetime)?;

    append_normalized_display(rendered, formatter.format(&time))
}

/// %O - localized relative time from now.
/// EN: "7 days ago" | ES: "hace 7 dias" | JA: "7 日前"
fn append_relative_time(
    rendered: &mut String,
    datetime: OffsetDateTime,
    locale: &Locale,
) -> io::Result<()> {
    let (value, unit) = relative_time_value(datetime);
    let prefs: RelativeTimeFormatterPreferences = locale.clone().into();
    let formatter = match unit {
        RelativeTimeUnit::Second => RelativeTimeFormatter::try_new_long_second(
            prefs,
            relative_time_fmt_options(Numeric::Always),
        ),
        RelativeTimeUnit::Minute => RelativeTimeFormatter::try_new_long_minute(
            prefs,
            relative_time_fmt_options(Numeric::Always),
        ),
        RelativeTimeUnit::Hour => RelativeTimeFormatter::try_new_long_hour(
            prefs,
            relative_time_fmt_options(Numeric::Always),
        ),
        RelativeTimeUnit::Day => RelativeTimeFormatter::try_new_long_day(
            prefs,
            relative_time_fmt_options(Numeric::Auto),
        ),
    }
    .map_err(localization_error)?;

    append_normalized_display(rendered, formatter.format(Decimal::from(value)))
}

/// Helper to create a `RelativeTimeFormatterOptions`.
/// Because the struct is non-exhaustive, we cannot use the struct syntax to
/// create new instances here.
fn relative_time_fmt_options(numeric: Numeric) -> RelativeTimeFormatterOptions {
    let mut options = RelativeTimeFormatterOptions::default();
    options.numeric = numeric;
    options
}

fn relative_time_value(datetime: OffsetDateTime) -> (i64, RelativeTimeUnit) {
    let delta_seconds = datetime.unix_timestamp() - now().timestamp();
    let abs_delta = delta_seconds.unsigned_abs();

    if abs_delta < 60 {
        (delta_seconds, RelativeTimeUnit::Second)
    } else if abs_delta < 3_600 {
        (delta_seconds / 60, RelativeTimeUnit::Minute)
    } else if abs_delta < 86_400 {
        (delta_seconds / 3_600, RelativeTimeUnit::Hour)
    } else {
        (delta_seconds / 86_400, RelativeTimeUnit::Day)
    }
}

#[derive(Copy, Clone)]
enum RelativeTimeUnit {
    Second,
    Minute,
    Hour,
    Day,
}

/// %Z - timezone as GMT offset.
/// "GMT+02" | "GMT-05:30"
fn append_timezone_gmt(rendered: &mut String, offset: &UtcOffset) {
    let total_seconds = offset.whole_seconds();
    let sign = if total_seconds < 0 { '-' } else { '+' };
    let absolute_seconds = total_seconds.abs();
    let hours = absolute_seconds / 3600;
    let minutes = (absolute_seconds % 3600) / 60;

    if minutes == 0 {
        str_write!(rendered, "GMT{sign}{hours:02}");
    } else {
        str_write!(rendered, "GMT{sign}{hours:02}:{minutes:02}");
    }
}

fn append_normalized_display(
    rendered: &mut String,
    value: impl fmt::Display,
) -> io::Result<()> {
    str_write!(&mut NormalizingWriter(rendered), "{value}");
    Ok(())
}

struct NormalizingWriter<'a>(&'a mut String);

impl fmt::Write for NormalizingWriter<'_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for ch in value.chars() {
            if matches!(ch, '\u{00A0}' | '\u{202F}') {
                self.0.push(' ');
            } else {
                self.0.push(ch);
            }
        }

        Ok(())
    }
}

/// %d, %H, %I, %m, %M, %R, %S, %T, %y, %Y, %z, etc. - non-localized strftime.
/// Delegates to the `time` crate's strftime formatter.
fn append_unlocalized_directive(
    rendered: &mut String,
    datetime: &OffsetDateTime,
    spec: &str,
) -> io::Result<()> {
    let items = parse_strftime_borrowed(spec).map_err(|error| {
        invalid_strftime_error(spec, &format!("failed to parse directive: {error}"))
    })?;

    rendered.push_str(&map_format_result(datetime.format(&items))?);
    Ok(())
}

fn to_icu_date(datetime: OffsetDateTime) -> io::Result<IcuDate<Iso>> {
    let date = datetime.date();

    IcuDate::try_new_iso(date.year(), date.month().into(), date.day())
        .map_err(localization_error)
}

fn to_icu_datetime(datetime: OffsetDateTime) -> io::Result<IcuDateTime<Iso>> {
    Ok(IcuDateTime {
        date: to_icu_date(datetime)?,
        time: to_icu_time(datetime)?,
    })
}

fn to_icu_time(datetime: OffsetDateTime) -> io::Result<IcuTime> {
    let time = datetime.time();

    IcuTime::try_new(time.hour(), time.minute(), time.second(), time.nanosecond())
        .map_err(localization_error)
}

fn locale_from_language(language: &str) -> Locale {
    Locale::try_from_str(language).unwrap_or_else(|error| {
        debug!(
            "Invalid date render locale '{language}', falling back to English: {error}"
        );
        Locale::try_from_str("en").expect("English locale should always parse")
    })
}

fn invalid_strftime_error(format: &str, message: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid strftime format string '{format}': {message}"),
    )
}

fn localization_error(error: impl ToString) -> io::Error {
    io::Error::other(error.to_string())
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

#[test]
fn date_format_supports_strftime() {
    let date = DateItem::from(time::macros::datetime!(2010-01-01 08:10:00 +00:00));

    assert_eq!(
        date.format(Some("%Y-%m-%d %H:%M:%S %z"), "en").unwrap(),
        "2010-01-01 08:10:00 GMT+00",
    );
}

#[test]
fn date_format_rejects_invalid_strftime() {
    let date = DateItem::from(time::macros::datetime!(2010-01-01 08:10:00 +00:00));

    date.format(Some("%Q"), "en")
        .expect_err("invalid strftime format unexpectedly succeeded");
}

#[test]
fn date_format_falls_back_to_default() {
    let date = DateItem::from(time::macros::datetime!(2010-01-01 08:10:00 +00:00));

    assert_eq!(
        date.format_or_default(Some("%Q"), "en"),
        date.format(None, "en").unwrap(),
    );
}

#[test]
fn date_format_defaults_to_localized_short_datetime() {
    let date = DateItem::from(time::macros::datetime!(2010-01-01 08:10:00 +00:00));

    assert_eq!(date.format(None, "en-US").unwrap(), "1/1/2010, 8:10:00 AM");
}

#[test]
fn date_format_uses_localized_relative_time_patterns() {
    let past_date = DateItem::from(time::macros::datetime!(2009-12-31 08:10:00 +00:00));
    let future_date = DateItem::from(time::macros::datetime!(2010-01-02 08:10:00 +00:00));

    assert_eq!(past_date.format(Some("%O"), "fr").unwrap(), "hier");
    assert_eq!(future_date.format(Some("%O"), "fr").unwrap(), "demain");
}

#[test]
fn date_format_supports_prefix_and_non_ascii_day_periods() {
    let date_am = DateItem::from(time::macros::datetime!(2009-12-25 08:18:05 +00:00));
    let date_pm = DateItem::from(time::macros::datetime!(2009-12-25 13:18:05 +00:00));

    // en-US: uppercase native, lowercase via %P
    assert_eq!(date_am.format(Some("%p"), "en-US").unwrap(), "AM");
    assert_eq!(date_am.format(Some("%P"), "en-US").unwrap(), "am");
    assert_eq!(date_pm.format(Some("%p"), "en-US").unwrap(), "PM");
    assert_eq!(date_pm.format(Some("%P"), "en-US").unwrap(), "pm");

    // other prefix/non-ASCII locales
    assert_eq!(date_am.format(Some("%p"), "ja").unwrap(), "午前");
    assert_eq!(date_am.format(Some("%P"), "ja").unwrap(), "午前");
    assert_eq!(date_pm.format(Some("%p"), "ja").unwrap(), "午後");
    assert_eq!(date_pm.format(Some("%P"), "ja").unwrap(), "午後");
    assert_eq!(date_am.format(Some("%p"), "zh-CN").unwrap(), "上午");
    assert_eq!(date_am.format(Some("%p"), "fa").unwrap(), "ق.ظ.");
}

#[test]
fn date_format_supports_prefix_day_periods_in_twelve_hour_time() {
    let date = DateItem::from(time::macros::datetime!(2009-12-25 08:18:05 +00:00));

    assert_eq!(date.format(Some("%r"), "zh-CN").unwrap(), "上午8:18:05");
    assert_eq!(date.format(Some("%r"), "ja").unwrap(), "午前8:18:05");
}

#[test]
fn date_format_supports_full_regression_matrix_in_spanish() {
    let date = DateItem::from(time::macros::datetime!(2009-12-25 08:18:05 +02:00));
    let format = "[a %a] [A %A] [b %b] [B %B] [c %c] [d %d] [D %D] [e %e] [H %H] [I %I] [m %m] [M %M] [O %O] [p %p] [P %P] [r %r] [R %R] [S %S] [T %T] [x %x] [X %X] [y %y] [Y %Y] [z %z] [Z %Z]";

    assert_eq!(
        date.format(Some(format), "es-ES").unwrap(),
        "[a vie] [A viernes] [b dic] [B diciembre] [c 25/12/2009, 08:18:05] [d 25] [D 12/25/09] [e 25] [H 08] [I 08] [m 12] [M 18] [O hace 7 d\u{00ED}as] [p a. m.] [P a. m.] [r 8:18:05 a. m.] [R 08:18] [S 05] [T 08:18:05] [x 25/12/09] [X 08:18:05] [y 09] [Y 2009] [z GMT+02] [Z GMT+02]"
    );
}

#[test]
fn date_format_supports_full_regression_matrix_in_english() {
    let date = DateItem::from(time::macros::datetime!(2009-12-25 08:18:05 +02:00));
    let format = "[a %a] [A %A] [b %b] [B %B] [c %c] [d %d] [D %D] [e %e] [H %H] [I %I] [m %m] [M %M] [O %O] [p %p] [P %P] [r %r] [R %R] [S %S] [T %T] [x %x] [X %X] [y %y] [Y %Y] [z %z] [Z %Z]";

    assert_eq!(
        date.format(Some(format), "en-US").unwrap(),
        "[a Fri] [A Friday] [b Dec] [B December] [c 12/25/2009, 8:18:05 AM] [d 25] [D 12/25/09] [e 25] [H 08] [I 08] [m 12] [M 18] [O 7 days ago] [p AM] [P am] [r 8:18:05 AM] [R 08:18] [S 05] [T 08:18:05] [x 12/25/09] [X 8:18:05 AM] [y 09] [Y 2009] [z GMT+02] [Z GMT+02]"
    );
}

#[test]
fn date_format_supports_full_regression_matrix_in_japanese() {
    let date = DateItem::from(time::macros::datetime!(2009-12-25 08:18:05 +02:00));
    let format = "[a %a] [A %A] [b %b] [B %B] [c %c] [d %d] [D %D] [e %e] [H %H] [I %I] [m %m] [M %M] [O %O] [p %p] [P %P] [r %r] [R %R] [S %S] [T %T] [x %x] [X %X] [y %y] [Y %Y] [z %z] [Z %Z]";

    assert_eq!(
        date.format(Some(format), "ja").unwrap(),
        "[a 金] [A 金曜日] [b 12月] [B 12月] [c 2009/12/25 8:18:05] [d 25] [D 12/25/09] [e 25] [H 08] [I 08] [m 12] [M 18] [O 7 日前] [p 午前] [P 午前] [r 午前8:18:05] [R 08:18] [S 05] [T 08:18:05] [x 2009/12/25] [X 8:18:05] [y 09] [Y 2009] [z GMT+02] [Z GMT+02]"
    );
}
