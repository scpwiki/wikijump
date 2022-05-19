/*
 * web/fetch_limit/de.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

use super::FetchLimit;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use std::fmt;

impl<'de> Deserialize<'de> for FetchLimit {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(FetchLimitVisitor)
    }
}

#[derive(Debug)]
struct FetchLimitVisitor;

impl FetchLimitVisitor {
    fn visit_unsigned<E>(&self, value: u128) -> Result<FetchLimit, E>
    where
        E: de::Error,
    {
        if value <= 100 {
            Ok(FetchLimit(value as u16))
        } else {
            Err(E::custom(format!("limit out of range: {}", value)))
        }
    }

    fn visit_signed<E>(&self, value: i128) -> Result<FetchLimit, E>
    where
        E: de::Error,
    {
        if value >= 0 {
            self.visit_unsigned(value as u128)
        } else {
            Err(E::custom(format!("limit must be positive: {}", value)))
        }
    }
}

macro_rules! impl_visit_unsigned {
    ($type:ty, $impl_method:ident) => {
        #[inline]
        fn $impl_method<E>(self, value: $type) -> Result<FetchLimit, E>
        where
            E: de::Error,
        {
            self.visit_unsigned(u128::from(value))
        }
    };
}

macro_rules! impl_visit_signed {
    ($type:ty, $impl_method:ident) => {
        #[inline]
        fn $impl_method<E>(self, value: $type) -> Result<FetchLimit, E>
        where
            E: de::Error,
        {
            self.visit_signed(i128::from(value))
        }
    };
}

impl<'de> Visitor<'de> for FetchLimitVisitor {
    type Value = FetchLimit;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 100")
    }

    impl_visit_signed!(i8, visit_i8);
    impl_visit_signed!(i16, visit_i16);
    impl_visit_signed!(i32, visit_i32);
    impl_visit_signed!(i64, visit_i64);
    impl_visit_signed!(i128, visit_i128);

    impl_visit_unsigned!(u8, visit_u8);
    impl_visit_unsigned!(u16, visit_u16);
    impl_visit_unsigned!(u32, visit_u32);
    impl_visit_unsigned!(u64, visit_u64);
    impl_visit_unsigned!(u128, visit_u128);
}

#[test]
fn fetch_limit_deserialize() {
    macro_rules! check_internal {
        ($value:expr => $expected:expr) => {{
            let actual: Option<FetchLimit> =
                serde_json::from_str(stringify!($value)).ok();

            assert_eq!(
                actual, $expected,
                "Actual item limit doesn't match expected",
            );
        }};
    }

    macro_rules! check_ok {
        ($value:expr $(,)?) => {
            check_internal!($value => Some(FetchLimit($value)))
        };
    }

    macro_rules! check_err {
        ($value:expr $(,)?) => {
            check_internal!($value => None)
        };
    }

    check_err!(-1000);
    check_err!(-100);
    check_err!(-10);
    check_err!(-1);

    check_ok!(0);
    check_ok!(1);
    check_ok!(5);
    check_ok!(10);
    check_ok!(20);
    check_ok!(50);
    check_ok!(100);

    check_err!(101);
    check_err!(200);
    check_err!(1000);
    check_err!(10000);
}
